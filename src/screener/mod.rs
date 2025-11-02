pub mod filters;
pub mod scanner;

use crate::models::CryptoPair;
use crate::indicators::TechnicalIndicator;

pub struct Screener {
    filters: Vec<Box<dyn Filter>>,
}

pub trait Filter {
    fn check(&self, pair: &CryptoPair) -> bool;
}

impl Screener {
    pub fn new() -> Self {
        Screener {
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }

    pub fn scan(&self, pairs: Vec<CryptoPair>) -> Vec<CryptoPair> {
        pairs.into_iter()
            .filter(|pair| self.filters.iter().all(|f| f.check(pair)))
            .collect()
    }
}
