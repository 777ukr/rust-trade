# 📋 ОДНОСТРОЧНАЯ КОМАНДА

Скопируйте и выполните:

```bash
PROJECT_DIR="/home/crypto/sites/cryptotrader.com" && mkdir -p "$PROJECT_DIR" && cd "$PROJECT_DIR" && bash /home/crypto/sites/cryptotrader.com/setup.sh
```

Или если скрипт еще не создан, используйте встроенную версию:

```bash
PROJECT_DIR="/home/crypto/sites/cryptotrader.com" && mkdir -p "$PROJECT_DIR" && cd "$PROJECT_DIR" && (command -v cargo &> /dev/null || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source "$HOME/.cargo/env")) && cargo init --name crypto_trader 2>/dev/null; mkdir -p src/{api,indicators,screener,parser,strategy,models,utils} tests config data/logs && echo '{"folders":[{"name":"crypto_trader","path":"/home/crypto/sites/cryptotrader.com"}],"settings":{"files.exclude":{"**/node_modules":true,"**/dist":true,"**/.git":true,"**/target":true}}}' > crypto_trader.code-workspace && echo "✅ Проект создан! Откройте: code $PROJECT_DIR/crypto_trader.code-workspace"
```

