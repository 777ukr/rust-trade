#!/usr/bin/env python3
"""
Импорт исторических данных с Gate.io API в rust-trade базу данных
Альтернатива premium data provider
"""

import os
import time
from datetime import datetime, timedelta, timezone
from typing import Dict, List

import psycopg2
import requests

# Настройки базы данных
DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://cryptotrader:cryptotrader@localhost/trading_core")

# Gate.io API
GATEIO_API_URL = "https://api.gateio.ws/api/v4"

def connect_db():
    """Подключение к базе данных"""
    return psycopg2.connect(DATABASE_URL)

def fetch_gateio_trades(symbol: str, start_time: datetime, end_time: datetime, limit: int = 1000) -> List[Dict]:
    """Загрузка исторических сделок с Gate.io"""
    # Конвертируем символ: ETHUSDT -> ETH_USDT
    gateio_symbol = symbol.replace("USDT", "_USDT")
    
    url = f"{GATEIO_API_URL}/futures/usdt/trades"
    params = {
        "contract": gateio_symbol,
        "limit": min(limit, 1000),  # Gate.io максимум 1000
        "from": int(start_time.timestamp()),
        "to": int(end_time.timestamp()),
    }
    
    print(f"📥 Загрузка данных с Gate.io для {symbol}...")
    print(f"   Период: {start_time} - {end_time}")
    
    try:
        response = requests.get(url, params=params, timeout=30)
        
        if response.status_code == 200:
            data = response.json()
            print(f"✅ Загружено {len(data)} сделок с Gate.io")
            return data
        else:
            print(f"❌ Ошибка Gate.io API: {response.status_code} - {response.text[:200]}")
            return []
    except Exception as e:
        print(f"❌ Ошибка при загрузке с Gate.io: {e}")
        return []

def convert_gateio_to_ticks(trades: List[Dict], symbol: str) -> List[Dict]:
    """Конвертирует сделки Gate.io в формат tick_data"""
    ticks = []
    
    for trade in trades:
        try:
            trade_id = str(trade.get("id", 0))
            create_time = trade.get("create_time", 0)
            price_str = trade.get("price", "0")
            size = abs(int(trade.get("size", 0)))  # Абсолютное значение
            role = trade.get("role", "maker")
            
            # Конвертируем timestamp (Gate.io использует секунды)
            timestamp = datetime.fromtimestamp(create_time, tz=timezone.utc)
            
            # Определяем side по role и size
            # В Gate.io size может быть отрицательным для продаж
            side = "SELL" if role == "taker" or trade.get("size", 0) < 0 else "BUY"
            
            tick = {
                'timestamp': timestamp,
                'symbol': symbol,  # Используем оригинальный формат ETHUSDT
                'price': float(price_str),
                'quantity': float(size),
                'side': side,
                'trade_id': trade_id,
                'is_buyer_maker': role == "maker"
            }
            ticks.append(tick)
        except Exception as e:
            print(f"⚠️  Ошибка при конвертации сделки: {e}")
            continue
    
    return ticks

def import_ticks_to_db(ticks: List[Dict], conn) -> int:
    """Импорт тиков в базу данных"""
    if not ticks:
        return 0
    
    cursor = conn.cursor()
    inserted = 0
    
    try:
        for tick in ticks:
            try:
                cursor.execute("""
                    INSERT INTO tick_data 
                    (timestamp, symbol, price, quantity, side, trade_id, is_buyer_maker)
                    VALUES (%s, %s, %s, %s, %s, %s, %s)
                    ON CONFLICT (symbol, trade_id, timestamp) DO NOTHING
                """, (
                    tick['timestamp'],
                    tick['symbol'],
                    tick['price'],
                    tick['quantity'],
                    tick['side'],
                    tick['trade_id'],
                    tick['is_buyer_maker']
                ))
                if cursor.rowcount > 0:
                    inserted += 1
            except Exception as e:
                # Rollback текущей транзакции при ошибке
                conn.rollback()
                print(f"⚠️  Ошибка при вставке тика: {e}")
                continue
        
        conn.commit()
    except Exception as e:
        conn.rollback()
        print(f"❌ Критическая ошибка при импорте: {e}")
    finally:
        cursor.close()
    
    return inserted

def download_and_import_eth(days: int = 30):
    """Скачать и импортировать данные ETH с Gate.io"""
    print(f"🚀 Начало загрузки и импорта ETH данных за последние {days} дней")
    
    conn = connect_db()
    
    try:
        symbol = "ETHUSDT"
        end_time = datetime.now(timezone.utc)
        start_time = end_time - timedelta(days=days)
        
        # Разбиваем на периоды по 1 дню для избежания лимитов
        current_start = start_time
        total_imported = 0
        
        while current_start < end_time:
            current_end = min(current_start + timedelta(days=1), end_time)
            
            print(f"\n📅 Загрузка периода: {current_start.date()} - {current_end.date()}")
            
            # Загружаем сделки
            trades = fetch_gateio_trades(symbol, current_start, current_end, limit=1000)
            
            if trades:
                # Конвертируем в тики
                ticks = convert_gateio_to_ticks(trades, symbol)
                
                # Импортируем в БД
                imported = import_ticks_to_db(ticks, conn)
                total_imported += imported
                
                print(f"✅ Импортировано {imported} новых тиков")
            
            # Небольшая задержка для избежания rate limits
            time.sleep(1)
            
            current_start = current_end
        
        print(f"\n🎉 Импорт завершен! Всего импортировано: {total_imported} тиков")
        
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()
    finally:
        conn.close()

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description='Импорт исторических данных с Gate.io для rust-trade')
    parser.add_argument('--days', type=int, default=30, help='Количество дней для загрузки (по умолчанию: 30)')
    parser.add_argument('--symbol', type=str, default='ETHUSDT', help='Символ (по умолчанию: ETHUSDT)')
    
    args = parser.parse_args()
    
    if args.symbol == 'ETHUSDT':
        download_and_import_eth(days=args.days)
    else:
        print("⚠️  Пока поддерживается только ETHUSDT")
        print("💡 Используйте: python3 import_gateio_data.py --symbol ETHUSDT --days 30")

