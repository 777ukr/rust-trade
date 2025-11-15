#!/usr/bin/env python3
"""
Импорт исторических данных из Freqtrade формата в rust-trade базу данных
Поддерживает импорт OHLCV данных (1m, 5m и т.д.) и конвертацию в tick data
"""

import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List

import psycopg2

# Добавляем путь к freqtrade скрипту
sys.path.insert(0, '/home/crypto/sites/cryptotrader.com/freqtrade')

try:
    from premium_data_provider import PremiumDataProvider
except ImportError:
    print("⚠️  Не удалось импортировать PremiumDataProvider")
    PremiumDataProvider = None

# Настройки базы данных
# По умолчанию использует пользователя cryptotrader (найден в системе)
DATABASE_URL = os.getenv("DATABASE_URL", "postgresql://cryptotrader:cryptotrader@localhost/trading_core")

def connect_db():
    """Подключение к базе данных"""
    return psycopg2.connect(DATABASE_URL)

def convert_ohlcv_to_ticks(ohlcv_data: List, symbol: str, timeframe_minutes: int = 1) -> List[Dict]:
    """
    Конвертирует OHLCV данные в tick data
    Для каждой свечи создаем несколько тиков (open, high, low, close)
    """
    ticks = []
    
    for candle in ohlcv_data:
        if len(candle) < 6:
            continue
            
        timestamp_seconds = int(candle[0])
        timestamp = datetime.fromtimestamp(timestamp_seconds, tz=datetime.timezone.utc)
        
        open_price = float(candle[1])
        high_price = float(candle[2])
        low_price = float(candle[3])
        close_price = float(candle[4])
        volume = float(candle[5])
        
        # Создаем тики для каждой свечи
        # Open tick
        ticks.append({
            'timestamp': timestamp,
            'symbol': symbol,
            'price': open_price,
            'quantity': volume / 4,  # Распределяем объем
            'side': 'BUY',
            'trade_id': f"{symbol}_{timestamp_seconds}_open",
            'is_buyer_maker': False
        })
        
        # High tick
        ticks.append({
            'timestamp': timestamp + timedelta(seconds=15 * timeframe_minutes),
            'symbol': symbol,
            'price': high_price,
            'quantity': volume / 4,
            'side': 'BUY',
            'trade_id': f"{symbol}_{timestamp_seconds}_high",
            'is_buyer_maker': False
        })
        
        # Low tick
        ticks.append({
            'timestamp': timestamp + timedelta(seconds=30 * timeframe_minutes),
            'symbol': symbol,
            'price': low_price,
            'quantity': volume / 4,
            'side': 'SELL',
            'trade_id': f"{symbol}_{timestamp_seconds}_low",
            'is_buyer_maker': True
        })
        
        # Close tick
        ticks.append({
            'timestamp': timestamp + timedelta(seconds=45 * timeframe_minutes),
            'symbol': symbol,
            'price': close_price,
            'quantity': volume / 4,
            'side': 'SELL',
            'trade_id': f"{symbol}_{timestamp_seconds}_close",
            'is_buyer_maker': True
        })
    
    return ticks

def import_from_file(file_path: Path, symbol: str, conn):
    """Импорт данных из JSON файла Freqtrade формата"""
    print(f"📂 Чтение файла: {file_path}")
    
    with open(file_path, 'r') as f:
        data = json.load(f)
    
    if not data:
        print(f"⚠️  Файл пуст: {file_path}")
        return 0
    
    # Определяем timeframe из имени файла
    timeframe = "1m"  # по умолчанию
    if "5m" in str(file_path):
        timeframe = "5m"
    elif "15m" in str(file_path):
        timeframe = "15m"
    elif "1h" in str(file_path):
        timeframe = "1h"
    
    timeframe_minutes = int(timeframe.replace('m', '').replace('h', '00').replace('d', '000'))
    if 'h' in timeframe:
        timeframe_minutes = int(timeframe.replace('h', '')) * 60
    
    print(f"📊 Конвертация {len(data)} свечей в tick data (timeframe: {timeframe})...")
    ticks = convert_ohlcv_to_ticks(data, symbol, timeframe_minutes)
    
    print(f"✅ Создано {len(ticks)} тиков")
    
    # Вставка в базу данных
    cursor = conn.cursor()
    inserted = 0
    
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
            print(f"⚠️  Ошибка при вставке тика: {e}")
            continue
    
    conn.commit()
    cursor.close()
    
    print(f"✅ Импортировано {inserted} новых тиков из {len(data)} свечей")
    return inserted

def download_and_import_eth_1m(days: int = 30):
    """Скачать и импортировать данные ETH 1m"""
    print(f"🚀 Начало загрузки и импорта ETH 1m данных за последние {days} дней")
    
    # Подключение к БД
    conn = connect_db()
    
    try:
        # Используем PremiumDataProvider если доступен
        if PremiumDataProvider:
            provider = PremiumDataProvider()
            source, file_path = provider.download_and_save(
                "ETH/USDT", 
                "1m", 
                days=days, 
                exchange="kaiko"
            )
            
            if file_path and Path(file_path).exists():
                print(f"📥 Данные загружены из {source}: {file_path}")
                imported = import_from_file(Path(file_path), "ETHUSDT", conn)
                print(f"✅ Импорт завершен: {imported} тиков")
            else:
                print("⚠️  Не удалось загрузить данные из premium источника")
        else:
            print("⚠️  PremiumDataProvider недоступен")
            print("💡 Используйте прямой импорт из файла или загрузите данные через Gate.io API")
            
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()
    finally:
        conn.close()

def import_from_freqtrade_file(file_path: str, symbol: str = "ETHUSDT"):
    """Импорт из существующего файла Freqtrade"""
    conn = connect_db()
    
    try:
        path = Path(file_path)
        if not path.exists():
            print(f"❌ Файл не найден: {file_path}")
            return
        
        imported = import_from_file(path, symbol, conn)
        print(f"✅ Импорт завершен: {imported} тиков")
        
    except Exception as e:
        print(f"❌ Ошибка: {e}")
        import traceback
        traceback.print_exc()
    finally:
        conn.close()

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description='Импорт исторических данных для rust-trade')
    parser.add_argument('--download', action='store_true', help='Скачать данные через PremiumDataProvider')
    parser.add_argument('--file', type=str, help='Путь к файлу Freqtrade JSON')
    parser.add_argument('--symbol', type=str, default='ETHUSDT', help='Символ (по умолчанию: ETHUSDT)')
    parser.add_argument('--days', type=int, default=30, help='Количество дней для загрузки (по умолчанию: 30)')
    
    args = parser.parse_args()
    
    if args.download:
        download_and_import_eth_1m(days=args.days)
    elif args.file:
        import_from_freqtrade_file(args.file, args.symbol)
    else:
        print("Использование:")
        print("  # Скачать и импортировать ETH 1m за 30 дней:")
        print("  python import_freqtrade_data.py --download --days 30")
        print("")
        print("  # Импортировать из файла:")
        print("  python import_freqtrade_data.py --file /path/to/data.json --symbol ETHUSDT")

