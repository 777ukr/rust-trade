// Stop loss implementation
use crate::models::Position;

pub fn check_stop_loss(position: &Position, current_price: f64) -> bool {
    match position.side.as_str() {
        "long" => current_price <= position.stop_loss,
        "short" => current_price >= position.stop_loss,
        _ => false,
    }
}

pub fn check_take_profit(position: &Position, current_price: f64) -> bool {
    match position.side.as_str() {
        "long" => current_price >= position.take_profit,
        "short" => current_price <= position.take_profit,
        _ => false,
    }
}
