use axum::Router;
use std::sync::Arc;
use crate::data::repository::TickDataRepository;

use super::handlers;

pub fn create_router(repository: Arc<TickDataRepository>) -> Router {
    Router::new()
        .route("/api/strategies", axum::routing::get(handlers::get_strategies))
        .route("/api/data/info", axum::routing::get(handlers::get_data_info))
        .route("/api/backtest/validate", axum::routing::get(handlers::validate_config))
        .with_state(repository)
}

