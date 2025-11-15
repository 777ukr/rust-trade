# ⚡ Быстрый старт - Frontend

## 🌐 Веб-интерфейс (Самый простой способ)

### Запуск

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/frontend
npm install  # Только при первой установке
npm run dev
```

### Откройте в браузере

```
http://localhost:3000
```

---

## 🖥️ Desktop приложение (Tauri)

### Запуск

```bash
cd /home/crypto/sites/cryptotrader.com/rust-trade/frontend
npm install  # Только при первой установке

# Вариант 1: Через npx (рекомендуется)
npx tauri dev

# Вариант 2: Через cargo (если tauri-cli установлен глобально)
cd ../src-tauri
cargo tauri dev
```

**Примечание:** Для Tauri нужны системные зависимости. См. [VISUAL_INTERFACE_GUIDE.md](../VISUAL_INTERFACE_GUIDE.md)

---

## 📋 Доступные команды

```bash
npm run dev      # Запуск веб-интерфейса (Next.js)
npm run build    # Сборка для production
npm run start    # Запуск production версии
npm run lint     # Проверка кода
```

---

## ⚠️ Решение проблем

### Проблема: "npm install" завершается с ошибками

```bash
# Удалите node_modules и переустановите
rm -rf node_modules package-lock.json
npm install
```

### Проблема: "npm run tauri dev" не работает

Используйте вместо этого:

```bash
npx tauri dev
```

Или для веб-интерфейса:

```bash
npm run dev
```

---

## 📚 Дополнительная информация

- [VISUAL_INTERFACE_GUIDE.md](../VISUAL_INTERFACE_GUIDE.md) - полное руководство
- [QUICK_START_VISUAL.md](../QUICK_START_VISUAL.md) - быстрый старт
