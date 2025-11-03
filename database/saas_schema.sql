-- =================================================================
-- SaaS Schema Extension for Multi-User Strategy Management
-- =================================================================

-- =================================================================
-- Table 1: Users (клиенты системы)
-- =================================================================
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Авторизация
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255) NOT NULL, -- bcrypt hash
    
    -- Роль: 'admin', 'client', 'trial'
    role VARCHAR(20) DEFAULT 'client' CHECK (role IN ('admin', 'client', 'trial')),
    
    -- Статус
    is_active BOOLEAN DEFAULT true,
    last_login TIMESTAMP WITH TIME ZONE,
    
    -- Контактная информация
    full_name VARCHAR(255),
    phone VARCHAR(50),
    
    -- Метаданные
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- =================================================================
-- Table 2: User Strategies (стратегии клиентов)
-- =================================================================
CREATE TABLE IF NOT EXISTS user_strategies (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Владелец
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Основная информация
    strategy_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Конфигурация стратегии (JSON из парсера)
    config_text TEXT NOT NULL, -- Полный текст ##Begin_Strategy ... ##End_Strategy
    config_json JSONB NOT NULL, -- Парсированная версия
    
    -- Статус
    is_active BOOLEAN DEFAULT false,
    is_public BOOLEAN DEFAULT false, -- Можно ли показывать другим
    
    -- Депозит и настройки для этого клиента
    initial_balance DECIMAL(20, 8) DEFAULT 1000.0,
    leverage INTEGER DEFAULT 10,
    
    -- Рейтинг и метрики
    rating DECIMAL(3, 1) DEFAULT 0.0 CHECK (rating >= 0 AND rating <= 10),
    stars INTEGER DEFAULT 0 CHECK (stars >= 0 AND stars <= 5),
    
    -- Бэктест результаты (лучший результат)
    best_roi DECIMAL(10, 4),
    best_profit_factor DECIMAL(10, 4),
    best_win_rate DECIMAL(10, 4),
    best_backtest_id BIGINT, -- Ссылка на backtest_results
    
    -- Теги и категории
    tags TEXT[], -- ['ema', 'scalping', 'long', 'short']
    category VARCHAR(50), -- 'momentum', 'reversal', 'market_making', 'custom'
    
    -- ИИ рекомендации
    ai_suggestions JSONB DEFAULT '[]'::jsonb, -- [{timestamp, suggestion, accepted}]
    
    -- Версионирование
    version INTEGER DEFAULT 1,
    parent_strategy_id BIGINT REFERENCES user_strategies(id), -- Для форков
    
    -- Метаданные
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_user_strategies_user_id ON user_strategies(user_id);
CREATE INDEX IF NOT EXISTS idx_user_strategies_rating ON user_strategies(rating DESC);
CREATE INDEX IF NOT EXISTS idx_user_strategies_stars ON user_strategies(stars DESC);
CREATE INDEX IF NOT EXISTS idx_user_strategies_best_roi ON user_strategies(best_roi DESC NULLS LAST);
CREATE INDEX IF NOT EXISTS idx_user_strategies_active ON user_strategies(is_active, is_public);

-- =================================================================
-- Table 3: API Keys (API ключи клиентов для live торговли)
-- =================================================================
CREATE TABLE IF NOT EXISTS api_keys (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Exchange информация
    exchange VARCHAR(20) NOT NULL DEFAULT 'gateio',
    
    -- API ключи (зашифрованы)
    api_key_encrypted TEXT NOT NULL,
    api_secret_encrypted TEXT NOT NULL,
    
    -- Настройки
    is_active BOOLEAN DEFAULT true,
    is_testnet BOOLEAN DEFAULT false,
    
    -- Лимиты и разрешения
    max_leverage INTEGER DEFAULT 100,
    allowed_symbols TEXT[], -- Если пусто - все доступны
    restricted_symbols TEXT[], -- Запрещенные символы
    
    -- Использование
    last_used_at TIMESTAMP WITH TIME ZONE,
    usage_count BIGINT DEFAULT 0,
    
    -- Метаданные
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active);

-- =================================================================
-- Table 4: Client Requests (заявки клиентов к админу)
-- =================================================================
CREATE TABLE IF NOT EXISTS client_requests (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Заявитель
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Тип запроса
    request_type VARCHAR(50) NOT NULL, -- 'strategy_customization', 'api_integration', 'support', 'feature_request'
    
    -- Содержимое
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    
    -- Связанная стратегия (если есть)
    strategy_id BIGINT REFERENCES user_strategies(id) ON DELETE SET NULL,
    
    -- Статус
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'rejected', 'cancelled')),
    
    -- Ответ админа
    admin_response TEXT,
    admin_id BIGINT REFERENCES users(id), -- Кто обработал
    resolved_at TIMESTAMP WITH TIME ZONE,
    
    -- Приоритет
    priority VARCHAR(10) DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high', 'urgent')),
    
    -- Метаданные
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_client_requests_user_id ON client_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_client_requests_status ON client_requests(status);
CREATE INDEX IF NOT EXISTS idx_client_requests_strategy_id ON client_requests(strategy_id);

-- =================================================================
-- Table 5: Strategy Ratings (отдельная таблица для рейтингов)
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_ratings (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    strategy_id BIGINT NOT NULL REFERENCES user_strategies(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Рейтинг (1-10)
    rating DECIMAL(3, 1) NOT NULL CHECK (rating >= 1 AND rating <= 10),
    
    -- Звезды (0-5)
    stars INTEGER DEFAULT 0 CHECK (stars >= 0 AND stars <= 5),
    
    -- Комментарий
    comment TEXT,
    
    -- Уникальность: один пользователь - один рейтинг на стратегию
    UNIQUE(strategy_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_strategy_ratings_strategy_id ON strategy_ratings(strategy_id);
CREATE INDEX IF NOT EXISTS idx_strategy_ratings_user_id ON strategy_ratings(user_id);

-- =================================================================
-- Table 6: AI Recommendations (ИИ рекомендации для стратегий)
-- =================================================================
CREATE TABLE IF NOT EXISTS ai_recommendations (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    strategy_id BIGINT NOT NULL REFERENCES user_strategies(id) ON DELETE CASCADE,
    
    -- Тип рекомендации
    recommendation_type VARCHAR(50) NOT NULL, -- 'parameter_tuning', 'risk_reduction', 'profit_optimization', 'stop_loss_adjustment'
    
    -- Содержимое
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    suggested_config JSONB, -- Изменения конфигурации
    expected_improvement JSONB, -- Ожидаемое улучшение метрик
    
    -- Статус
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'applied')),
    
    -- Кто применил
    applied_by_user_id BIGINT REFERENCES users(id),
    applied_at TIMESTAMP WITH TIME ZONE,
    
    -- Результат после применения
    actual_improvement JSONB, -- Реальные результаты после бэктеста
    backtest_result_id BIGINT REFERENCES backtest_results(id),
    
    -- Метаданные
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_ai_recommendations_strategy_id ON ai_recommendations(strategy_id);
CREATE INDEX IF NOT EXISTS idx_ai_recommendations_status ON ai_recommendations(status);

-- =================================================================
-- Table 7: Strategy Backtest Links (связь стратегий с бэктестами)
-- =================================================================
CREATE TABLE IF NOT EXISTS strategy_backtest_links (
    id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    strategy_id BIGINT NOT NULL REFERENCES user_strategies(id) ON DELETE CASCADE,
    backtest_id BIGINT NOT NULL REFERENCES backtest_results(id) ON DELETE CASCADE,
    
    -- Контекст запуска
    initial_balance DECIMAL(20, 8) NOT NULL,
    leverage INTEGER NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    
    -- Результат был успешным?
    is_best_result BOOLEAN DEFAULT false,
    
    UNIQUE(strategy_id, backtest_id)
);

CREATE INDEX IF NOT EXISTS idx_strategy_backtest_links_strategy_id ON strategy_backtest_links(strategy_id);
CREATE INDEX IF NOT EXISTS idx_strategy_backtest_links_backtest_id ON strategy_backtest_links(backtest_id);
CREATE INDEX IF NOT EXISTS idx_strategy_backtest_links_best ON strategy_backtest_links(is_best_result);

-- =================================================================
-- Function: Update strategy rating from ratings table
-- =================================================================
CREATE OR REPLACE FUNCTION update_strategy_rating()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE user_strategies
    SET 
        rating = (
            SELECT COALESCE(AVG(rating), 0)
            FROM strategy_ratings
            WHERE strategy_id = NEW.strategy_id
        ),
        stars = (
            SELECT CASE 
                WHEN AVG(rating) >= 9 THEN 5
                WHEN AVG(rating) >= 8 THEN 4
                WHEN AVG(rating) >= 7 THEN 3
                WHEN AVG(rating) >= 6 THEN 2
                WHEN AVG(rating) >= 5 THEN 1
                ELSE 0
            END
            FROM strategy_ratings
            WHERE strategy_id = NEW.strategy_id
        )
    WHERE id = NEW.strategy_id;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_strategy_rating
AFTER INSERT OR UPDATE OR DELETE ON strategy_ratings
FOR EACH ROW
EXECUTE FUNCTION update_strategy_rating();

-- =================================================================
-- Function: Update updated_at timestamp
-- =================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_update_user_strategies_updated_at
BEFORE UPDATE ON user_strategies
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_update_api_keys_updated_at
BEFORE UPDATE ON api_keys
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trigger_update_client_requests_updated_at
BEFORE UPDATE ON client_requests
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- =================================================================
-- Comments
-- =================================================================
COMMENT ON TABLE users IS 'Клиенты и администраторы системы';
COMMENT ON TABLE user_strategies IS 'Стратегии пользователей с конфигурациями';
COMMENT ON TABLE api_keys IS 'API ключи клиентов для live торговли';
COMMENT ON TABLE client_requests IS 'Заявки клиентов к админу на кастомизацию';
COMMENT ON TABLE strategy_ratings IS 'Рейтинги и оценки стратегий';
COMMENT ON TABLE ai_recommendations IS 'ИИ рекомендации для улучшения стратегий';
COMMENT ON TABLE strategy_backtest_links IS 'Связь стратегий с результатами бэктестов';


