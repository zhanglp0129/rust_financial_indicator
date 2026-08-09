use rust_decimal::Decimal;

/// 相对强弱指数RSI。
///
/// RSI 通过比较价格变动的平均涨幅和平均跌幅，衡量价格动量，取值范围为 `[0, 100]`：
///
/// - `RS = average_gain / average_loss`
/// - `RSI = 100 - 100 / (1 + RS)`
///
/// 样本数不足一个周期时，使用已有的所有价格变动计算平均值，分母为实际变动数，
/// 而不是 `period`。样本数超过一个周期后，使用 Wilder 平滑公式：
///
/// `average = (previous_average * (period - 1) + current_change) / period`
///
/// 返回值与 `prices` 等长。第一个价格没有可比较的前值，RSI 记为中性值 `50`。
/// 当平均涨幅和平均跌幅同时为零时，RSI 同样记为 `50`。
/// 当 `prices` 为空或 `period == 0` 时返回空数组。
///
/// # 参数
///
/// - `prices`：按时间从旧到新排列的价格，一般为收盘价。
/// - `period`：平均涨跌幅的平滑周期，通常为 14。
pub fn relative_strength_index(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    if prices.is_empty() || period == 0 {
        return vec![];
    }

    let mut rsi = Vec::with_capacity(prices.len());
    rsi.push(Decimal::from(50));

    let period_decimal = Decimal::from(period);
    let period_minus_one = Decimal::from(period - 1);
    let hundred = Decimal::from(100);
    let mut gain_sum = Decimal::ZERO;
    let mut loss_sum = Decimal::ZERO;
    let mut avg_gain = Decimal::ZERO;
    let mut avg_loss = Decimal::ZERO;

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        let gain = Decimal::ZERO.max(change);
        let loss = Decimal::ZERO.max(-change);

        if i <= period {
            gain_sum += gain;
            loss_sum += loss;
            let sample_count = Decimal::from(i);
            avg_gain = gain_sum / sample_count;
            avg_loss = loss_sum / sample_count;
        } else {
            avg_gain = (avg_gain * period_minus_one + gain) / period_decimal;
            avg_loss = (avg_loss * period_minus_one + loss) / period_decimal;
        }

        let value = if avg_loss == Decimal::ZERO {
            if avg_gain == Decimal::ZERO {
                Decimal::from(50)
            } else {
                hundred
            }
        } else {
            let relative_strength = avg_gain / avg_loss;
            hundred - hundred / (Decimal::ONE + relative_strength)
        };
        
        rsi.push(value);
    }

    rsi
}

/// 使用默认周期 14 计算 RSI。
pub fn relative_strength_index_default(prices: &[Decimal]) -> Vec<Decimal> {
    relative_strength_index(prices, 14)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimals(values: &[i64]) -> Vec<Decimal> {
        values.iter().copied().map(Decimal::from).collect()
    }

    #[test]
    fn calculates_wilder_rsi_with_available_samples_during_warmup() {
        let prices = decimals(&[10, 12, 10, 12, 10]);

        let actual = relative_strength_index(&prices, 2);

        assert_eq!(
            actual,
            vec![
                Decimal::from(50),
                Decimal::from(100),
                Decimal::from(50),
                Decimal::from(75),
                Decimal::new(375, 1),
            ]
        );
    }

    #[test]
    fn handles_rising_falling_and_unchanged_prices() {
        assert_eq!(
            relative_strength_index(&decimals(&[1, 2, 3]), 14),
            decimals(&[50, 100, 100])
        );
        assert_eq!(
            relative_strength_index(&decimals(&[3, 2, 1]), 14),
            decimals(&[50, 0, 0])
        );
        assert_eq!(
            relative_strength_index(&decimals(&[1, 1, 1]), 14),
            decimals(&[50, 50, 50])
        );
    }

    #[test]
    fn default_function_uses_period_14() {
        let prices = decimals(&[1, 2, 1, 3]);

        assert_eq!(
            relative_strength_index_default(&prices),
            relative_strength_index(&prices, 14)
        );
    }

    #[test]
    fn empty_prices_or_zero_period_return_an_empty_result() {
        let prices = decimals(&[1, 2, 3]);

        assert!(relative_strength_index(&[], 14).is_empty());
        assert!(relative_strength_index(&prices, 0).is_empty());
        assert!(relative_strength_index_default(&[]).is_empty());
    }
}
