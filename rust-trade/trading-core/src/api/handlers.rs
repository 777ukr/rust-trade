use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::data::repository::TickDataRepository;

#[derive(Debug, Serialize)]
pub struct StrategyInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub records_count: u64,
    pub earliest_time: Option<String>,
    pub latest_time: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DataInfoResponse {
    pub total_records: u64,
    pub symbols_count: u64,
    pub earliest_time: Option<String>,
    pub latest_time: Option<String>,
    pub symbol_info: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateQuery {
    pub symbol: String,
    pub data_count: i64,
}

pub async fn get_strategies() -> Result<Json<Vec<StrategyInfo>>, StatusCode> {
    let strategies = crate::backtest::strategy::list_strategies();
    let response: Vec<StrategyInfo> = strategies.into_iter().map(|s| StrategyInfo {
        id: s.id,
        name: s.name,
        description: s.description,
    }).collect();
    
    Ok(Json(response))
}

pub async fn get_data_info(
    State(repository): State<Arc<TickDataRepository>>,
) -> Result<Json<DataInfoResponse>, StatusCode> {
    let data_info = repository
        .get_backtest_data_info()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = DataInfoResponse {
        total_records: data_info.total_records,
        symbols_count: data_info.symbols_count,
        earliest_time: data_info.earliest_time.map(|t| t.to_rfc3339()),
        latest_time: data_info.latest_time.map(|t| t.to_rfc3339()),
        symbol_info: data_info.symbol_info.into_iter().map(|info| SymbolInfo {
            symbol: info.symbol,
            records_count: info.records_count,
            earliest_time: info.earliest_time.map(|t| t.to_rfc3339()),
            latest_time: info.latest_time.map(|t| t.to_rfc3339()),
            min_price: info.min_price.map(|p| p.to_string()),
            max_price: info.max_price.map(|p| p.to_string()),
        }).collect(),
    };

    Ok(Json(response))
}

pub async fn validate_config(
    State(repository): State<Arc<TickDataRepository>>,
    Query(params): Query<ValidateQuery>,
) -> Result<Json<bool>, StatusCode> {
    let data_info = repository
        .get_backtest_data_info()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let is_valid = data_info.has_sufficient_data(&params.symbol, params.data_count as u64);
    
    Ok(Json(is_valid))
}

