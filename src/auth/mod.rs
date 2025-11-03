//! Модуль авторизации и аутентификации
//! JWT токены для работы по IP (без домена)

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    response::Response,
    RequestPartsExt,
};
use axum::middleware::Next;
use axum::extract::State;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Duration, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id или username
    pub role: String, // 'admin', 'client', 'trial'
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl AuthState {
    pub fn new(secret: Option<String>) -> Self {
        let secret = secret.unwrap_or_else(|| {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-key-change-in-production".to_string())
        });
        
        Self {
            jwt_secret: secret.clone(),
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }
}

pub fn create_token(user_id: i64, username: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-super-secret-key-change-in-production".to_string());
    
    let now = Utc::now();
    let exp = now + Duration::days(30); // Токен на 30 дней
    
    let claims = Claims {
        sub: format!("{}:{}", user_id, username),
        role: role.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn verify_token(token: &str, auth_state: &AuthState) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &auth_state.decoding_key,
        &Validation::default(),
    )
    .map(|data| data.claims)
}

/// Middleware для проверки JWT токена
pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Извлекаем токен из заголовка Authorization
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        });
    
    if let Some(token) = token {
        match verify_token(&token, &auth_state) {
            Ok(claims) => {
                // Добавляем claims в extensions для использования в handlers
                request.extensions_mut().insert(claims);
            }
            Err(_) => {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(next.run(request).await)
}

/// Middleware для проверки роли админа
pub async fn admin_middleware(
    State(auth_state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        });
    
    if let Some(token) = token {
        match verify_token(&token, &auth_state) {
            Ok(claims) => {
                if claims.role == "admin" {
                    request.extensions_mut().insert(claims);
                    return Ok(next.run(request).await);
                }
            }
            Err(_) => {}
        }
    }
    
    Err(StatusCode::FORBIDDEN)
}

/// Извлечение Claims из request extensions
pub fn extract_claims(request: &Request) -> Option<Claims> {
    request.extensions().get::<Claims>().cloned()
}

// Хеширование паролей
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2::Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::password_hash::PasswordVerifier;
    if let Ok(parsed_hash) = argon2::PasswordHash::new(hash) {
        argon2::Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

// Fallback на bcrypt если argon2 недоступен
#[cfg(feature = "bcrypt")]
pub fn hash_password_bcrypt(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

#[cfg(feature = "bcrypt")]
pub fn verify_password_bcrypt(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

pub mod users;

