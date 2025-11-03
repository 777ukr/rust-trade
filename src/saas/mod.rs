//! SaaS модуль для управления стратегиями пользователей
//! Визуальный редактор, рейтинги, ИИ рекомендации

#[cfg(feature = "database")]
pub mod strategies;

#[cfg(feature = "database")]
pub mod ratings;

#[cfg(feature = "database")]
pub mod ai_recommendations;

