//! Расчет комиссий Gate.io с учетом 60% возврата на Futures
//! 
//! Gate.io возвращает 60% комиссии обратно на Futures контрактах
//! Maker: 0.015% → фактическая: 0.015% * 0.4 = 0.006%
//! Taker: 0.05% → фактическая: 0.05% * 0.4 = 0.02%

/// Рассчитать комиссию с учетом возврата
pub fn calculate_fee_with_rebate(amount: f64, is_maker: bool, use_rebate: bool) -> f64 {
    let base_maker_fee = 0.00015;  // 0.015%
    let base_taker_fee = 0.0005;   // 0.05%
    let rebate_rate = 0.6;          // 60% возврат
    
    let base_fee = if is_maker { base_maker_fee } else { base_taker_fee };
    
    if use_rebate {
        // Фактическая комиссия = базовая * (1 - rebate_rate)
        base_fee * amount * (1.0 - rebate_rate)
    } else {
        base_fee * amount
    }
}

/// Рассчитать возврат комиссии
pub fn calculate_rebate_amount(total_fee: f64) -> f64 {
    total_fee * 0.6
}

/// Рассчитать фактическую комиссию после возврата
pub fn calculate_net_fee(total_fee: f64, use_rebate: bool) -> f64 {
    if use_rebate {
        total_fee * 0.4  // Остается только 40%
    } else {
        total_fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maker_fee_with_rebate() {
        let amount = 1000.0;
        let fee = calculate_fee_with_rebate(amount, true, true);
        // 0.015% * 0.4 = 0.006% от 1000 = 0.06
        assert!((fee - 0.06).abs() < 0.001);
    }

    #[test]
    fn test_taker_fee_with_rebate() {
        let amount = 1000.0;
        let fee = calculate_fee_with_rebate(amount, false, true);
        // 0.05% * 0.4 = 0.02% от 1000 = 0.2
        assert!((fee - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_fee_without_rebate() {
        let amount = 1000.0;
        let fee = calculate_fee_with_rebate(amount, true, false);
        // 0.015% от 1000 = 0.15
        assert!((fee - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_rebate_calculation() {
        let total_fee = 100.0;
        let rebate = calculate_rebate_amount(total_fee);
        assert!((rebate - 60.0).abs() < 0.001);
    }
}



