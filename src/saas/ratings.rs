//! Модуль рейтингов стратегий

use anyhow::Result;
use sqlx::{FromRow, PgPool};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StrategyRating {
    pub id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub strategy_id: i64,
    pub user_id: i64,
    pub rating: Decimal, // 1-10
    pub stars: i32,      // 0-5
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateStrategyRequest {
    pub rating: f64, // 1-10
    pub stars: Option<i32>, // 0-5
    pub comment: Option<String>,
}

pub struct RatingRepository {
    pool: PgPool,
}

impl RatingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Добавить/обновить рейтинг стратегии
    pub async fn rate_strategy(
        &self,
        strategy_id: i64,
        user_id: i64,
        req: RateStrategyRequest,
    ) -> Result<StrategyRating> {
        let rating = Decimal::try_from(req.rating).unwrap_or(Decimal::ZERO)
            .max(Decimal::ONE)
            .min(Decimal::from(10));
        let stars = req.stars.unwrap_or(0).max(0).min(5);
        
        let rating_row = sqlx::query_as::<_, StrategyRating>(
            r#"
            INSERT INTO strategy_ratings (strategy_id, user_id, rating, stars, comment)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (strategy_id, user_id)
            DO UPDATE SET
                rating = EXCLUDED.rating,
                stars = EXCLUDED.stars,
                comment = EXCLUDED.comment
            RETURNING id, created_at, strategy_id, user_id, rating, stars, comment
            "#
        )
        .bind(strategy_id)
        .bind(user_id)
        .bind(rating)
        .bind(stars)
        .bind(&req.comment)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(rating_row)
    }
    
    /// Получить рейтинги стратегии
    pub async fn get_strategy_ratings(&self, strategy_id: i64) -> Result<Vec<StrategyRating>> {
        let ratings = sqlx::query_as::<_, StrategyRating>(
            "SELECT * FROM strategy_ratings WHERE strategy_id = $1 ORDER BY created_at DESC"
        )
        .bind(strategy_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(ratings)
    }
}

