// Scanner implementation
use crate::models::CryptoPair;
use crate::screener::Screener;

pub fn scan_markets(screener: &Screener, pairs: Vec<CryptoPair>) -> Vec<CryptoPair> {
    screener.scan(pairs)
}
