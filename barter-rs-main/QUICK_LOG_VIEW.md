# 🚀 Быстрый просмотр логов и отчетов

## 📊 Пока стратегия работает

### Просмотр логов в реальном времени

```bash
# Найти последний лог и следить за ним
tail -f $(ls -t logs/eth_strategy_*.log 2>/dev/null | head -1)

# Или использовать скрипт
./view_logs.sh
```

### Поиск входов в позиции

```bash
grep "ENTERING LONG POSITION" logs/eth_strategy_*.log
```

### Просмотр всех событий стратегии

```bash
grep "eth_dip_strategy" logs/eth_strategy_*.log
```

## 📄 После остановки стратегии (Ctrl+C)

### Просмотр отчета

```bash
# Последний отчет
cat $(ls -t eth_strategy_summary_*.txt | head -1)

# Или все отчеты
ls -lh eth_strategy_summary_*.txt
```

## 🔍 Полезные команды

```bash
# Размер логов
du -sh logs/

# Количество входов в позиции
grep -c "ENTERING LONG POSITION" logs/eth_strategy_*.log

# Последние 50 строк лога
tail -50 $(ls -t logs/eth_strategy_*.log | head -1)

# Поиск ошибок
grep -i "error" logs/eth_strategy_*.log
```

## 📍 Где находятся файлы

- **Логи**: `logs/eth_strategy_YYYYMMDD_HHMMSS.log`
- **Отчеты**: `eth_strategy_summary_YYYYMMDD_HHMMSS.txt`

Полные пути:

- `/home/crypto/sites/cryptotrader.com/barter-rs-main/logs/`
- `/home/crypto/sites/cryptotrader.com/barter-rs-main/`
