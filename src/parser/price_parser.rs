// Price parser implementation
pub fn parse_price(price_str: &str) -> Result<f64, String> {
    price_str.parse::<f64>().map_err(|e| format!("Failed to parse price: {}", e))
}
