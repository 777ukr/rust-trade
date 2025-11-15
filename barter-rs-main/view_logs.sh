#!/bin/bash
# Скрипт для просмотра логов стратегии

LOG_DIR="logs"
SUMMARY_DIR="."

echo "📊 ETHEREUM DIP STRATEGY - ПРОСМОТР ЛОГОВ И ОТЧЕТОВ"
echo "=================================================="
echo ""

# Проверка логов
echo "📁 ЛОГИ:"
if [ -d "$LOG_DIR" ] && [ -n "$(ls -A $LOG_DIR/eth_strategy_*.log 2>/dev/null)" ]; then
    LATEST_LOG=$(ls -t $LOG_DIR/eth_strategy_*.log 2>/dev/null | head -1)
    if [ -n "$LATEST_LOG" ]; then
        echo "   ✅ Последний лог: $LATEST_LOG"
        echo "   📊 Размер: $(du -h "$LATEST_LOG" | cut -f1)"
        echo "   📅 Создан: $(stat -c %y "$LATEST_LOG" | cut -d. -f1)"
        echo ""
        echo "   Последние 20 строк:"
        echo "   ----------------------------------------"
        tail -20 "$LATEST_LOG" | sed 's/^/   /'
    fi
else
    echo "   ⚠️  Логи еще не созданы (стратегия только запустилась)"
fi

echo ""
echo "📄 ОТЧЕТЫ:"
if [ -n "$(ls -A $SUMMARY_DIR/eth_strategy_summary_*.txt 2>/dev/null)" ]; then
    LATEST_REPORT=$(ls -t $SUMMARY_DIR/eth_strategy_summary_*.txt 2>/dev/null | head -1)
    if [ -n "$LATEST_REPORT" ]; then
        echo "   ✅ Последний отчет: $LATEST_REPORT"
        echo "   📊 Размер: $(du -h "$LATEST_REPORT" | cut -f1)"
        echo "   📅 Создан: $(stat -c %y "$LATEST_REPORT" | cut -d. -f1)"
    fi
else
    echo "   ⚠️  Отчеты появятся после остановки стратегии (Ctrl+C)"
fi

echo ""
echo "💡 Полезные команды:"
echo "   tail -f $LOG_DIR/eth_strategy_*.log  # Просмотр в реальном времени"
echo "   grep 'ENTERING' $LOG_DIR/eth_strategy_*.log  # Поиск входов в позиции"

