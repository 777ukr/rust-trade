// Filter implementations for screener
use crate::models::CryptoPair;
use crate::screener::Filter;

pub struct VolumeFilter {
    min_volume: f64,
}

impl VolumeFilter {
    pub fn new(min_volume: f64) -> Self {
        VolumeFilter { min_volume }
    }
}

impl Filter for VolumeFilter {
    fn check(&self, pair: &CryptoPair) -> bool {
        pair.volume_24h >= self.min_volume
    }
}
