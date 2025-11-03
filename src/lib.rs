pub mod base_classes;
pub mod collectors;
pub mod exchanges;
pub mod utils;

#[cfg(feature = "gate_exec")]
pub mod execution;

#[cfg(feature = "gate_exec")]
pub mod strategy;

#[cfg(feature = "gate_exec")]
pub mod config;

#[cfg(feature = "gate_exec")]
pub mod logging;

// Analytics and testing
pub mod analytics;
pub mod tests;

// Models
pub mod models;

// Database (requires PostgreSQL feature)
#[cfg(feature = "database")]
pub mod database;

// Auth module (requires dashboard feature)
#[cfg(all(feature = "dashboard", feature = "database"))]
pub mod auth;

// SaaS module (requires dashboard and database)
#[cfg(all(feature = "dashboard", feature = "database"))]
pub mod saas;

// Backtest module (requires gate_exec)
#[cfg(feature = "gate_exec")]
pub mod backtest;
