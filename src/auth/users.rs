//! Модуль управления пользователями

use crate::database::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub username: String,
    pub email: Option<String>,
    pub role: String, // 'admin', 'client', 'trial'
    pub is_active: bool,
    pub last_login: Option<chrono::DateTime<Utc>>,
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Создание нового пользователя
    pub async fn create_user(
        &self,
        req: CreateUserRequest,
        password_hash: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, role, full_name, phone)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, created_at, updated_at, username, email, role, is_active, 
                     last_login, full_name, phone, metadata
            "#
        )
        .bind(&req.username)
        .bind(&req.email)
        .bind(password_hash)
        .bind(req.role.unwrap_or_else(|| "client".to_string()))
        .bind(&req.full_name)
        .bind(&req.phone)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create user")?;
        
        Ok(user)
    }
    
    /// Получение пользователя по username
    pub async fn get_by_username(&self, username: &str) -> Result<Option<(User, String)>> {
        #[derive(FromRow)]
        struct UserWithPassword {
            id: i64,
            created_at: chrono::DateTime<Utc>,
            updated_at: chrono::DateTime<Utc>,
            username: String,
            email: Option<String>,
            password_hash: String,
            role: String,
            is_active: bool,
            last_login: Option<chrono::DateTime<Utc>>,
            full_name: Option<String>,
            phone: Option<String>,
            metadata: serde_json::Value,
        }
        
        let result = sqlx::query_as::<_, UserWithPassword>(
            "SELECT id, created_at, updated_at, username, email, password_hash, role, 
                    is_active, last_login, full_name, phone, metadata
             FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user")?;
        
        if let Some(user_with_pass) = result {
            let user = User {
                id: user_with_pass.id,
                created_at: user_with_pass.created_at,
                updated_at: user_with_pass.updated_at,
                username: user_with_pass.username,
                email: user_with_pass.email,
                role: user_with_pass.role,
                is_active: user_with_pass.is_active,
                last_login: user_with_pass.last_login,
                full_name: user_with_pass.full_name,
                phone: user_with_pass.phone,
                metadata: user_with_pass.metadata,
            };
            Ok(Some((user, user_with_pass.password_hash)))
        } else {
            Ok(None)
        }
    }
    
    /// Получение пользователя по ID
    pub async fn get_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, created_at, updated_at, username, email, role, is_active, 
                    last_login, full_name, phone, metadata
             FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user")?;
        
        Ok(user)
    }
    
    /// Обновление last_login
    pub async fn update_last_login(&self, user_id: i64) -> Result<()> {
        sqlx::query("UPDATE users SET last_login = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(user_id)
            .execute(&self.pool)
            .await
            .context("Failed to update last_login")?;
        
        Ok(())
    }
}

