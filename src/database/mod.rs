//! Database module for PostgreSQL integration
//! Provides repository pattern for data persistence and retrieval

pub mod repository;
pub mod types;

pub use repository::{DatabaseRepository, RepositoryError};
pub use types::*;

