use rust_decimal::Decimal;

/// 简单移动平均线SMA。
///
/// 返回值长度为 `prices.len()`，与 `prices` 一一对应。前 `period - 1` 个元素为前方所有价格的平均值。
///
/// `period == 0` 则返回空数组。
pub fn simple_moving_average(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    if period == 0 || prices.is_empty() {
        return vec![];
    }
    let mut sma = Vec::with_capacity(prices.len());
    let mut sum = Decimal::ZERO;
    let period_decimal = Decimal::from(period);
    for (i, &price) in prices.iter().enumerate() {
        sum += price;
        if i + 1 < period {
            sma.push(sum / Decimal::from(i + 1));
        } else {
            sma.push(sum / period_decimal);
            sum -= prices[i + 1 - period];
        }
    }
    sma
}

/// 加权移动平均线WMA。
///
/// 返回值长度为 `prices.len()`，与 `prices` 一一对应。`weights` 按从旧到新的顺序与价格对应。
/// 样本数不足一个周期时，使用 `weights` 后方数量相等的元素。
///
/// `weights` 为空时返回空数组。
///
/// # Panics
///
/// 当任一计算窗口的权重和为零时 panic。
pub fn weighted_moving_average(prices: &[Decimal], weights: &[Decimal]) -> Vec<Decimal> {
    let period = weights.len();
    if period == 0 || prices.is_empty() {
        return vec![];
    }

    let mut wma = Vec::with_capacity(prices.len());
    for i in 0..prices.len() {
        let window_len = (i + 1).min(period);
        let prices = &prices[i + 1 - window_len..=i];
        let weights = &weights[period - window_len..];
        let weight_sum: Decimal = weights.iter().copied().sum();
        assert_ne!(weight_sum, Decimal::ZERO, "WMA 计算窗口的权重和不能为零");

        let weighted_sum: Decimal = prices
            .iter()
            .zip(weights)
            .map(|(&price, &weight)| price * weight)
            .sum();
        wma.push(weighted_sum / weight_sum);
    }
    wma
}

/// 指数移动平均线EMA。
///
/// 返回值长度为 `prices.len()`，与 `prices` 一一对应，并以第一个价格作为初始值。
///
/// `period == 0` 则返回空数组。
pub fn exponential_moving_average(prices: &[Decimal], period: usize) -> Vec<Decimal> {
    if period == 0 || prices.is_empty() {
        return vec![];
    }
    let period_decimal = Decimal::from(period);
    // alpha=2/(period+1)
    let alpha = Decimal::TWO / (period_decimal + Decimal::ONE);
    let mut ema = Vec::with_capacity(prices.len());
    let mut prev = prices[0];
    for &price in prices {
        prev = alpha * price + (Decimal::ONE - alpha) * prev;
        ema.push(prev);
    }
    ema
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimals(values: &[i64]) -> Vec<Decimal> {
        values.iter().copied().map(Decimal::from).collect()
    }

    #[test]
    fn sma_uses_available_samples_during_warmup() {
        let prices = decimals(&[1, 2, 3, 4, 5]);

        let actual = simple_moving_average(&prices, 3);

        assert_eq!(
            actual,
            vec![
                Decimal::ONE,
                Decimal::new(15, 1),
                Decimal::from(2),
                Decimal::from(3),
                Decimal::from(4),
            ]
        );
    }

    #[test]
    fn wma_aligns_recent_price_with_last_weight() {
        let prices = decimals(&[2, 2, 6, 10]);
        let weights = decimals(&[1, 1, 2]);

        let actual = weighted_moving_average(&prices, &weights);

        assert_eq!(actual, decimals(&[2, 2, 4, 7]));
    }

    #[test]
    fn ema_uses_first_price_as_seed() {
        let prices = decimals(&[2, 4, 8, 4]);

        let actual = exponential_moving_average(&prices, 3);

        assert_eq!(
            actual,
            vec![
                Decimal::from(2),
                Decimal::from(3),
                Decimal::new(55, 1),
                Decimal::new(475, 2),
            ]
        );
    }

    #[test]
    fn empty_or_zero_period_inputs_return_empty_results() {
        let prices = decimals(&[1, 2, 3]);

        assert!(simple_moving_average(&prices, 0).is_empty());
        assert!(weighted_moving_average(&prices, &[]).is_empty());
        assert!(exponential_moving_average(&prices, 0).is_empty());
        assert!(simple_moving_average(&[], 3).is_empty());
        assert!(weighted_moving_average(&[], &decimals(&[1, 2, 3])).is_empty());
        assert!(exponential_moving_average(&[], 3).is_empty());
    }

    #[test]
    #[should_panic(expected = "WMA 计算窗口的权重和不能为零")]
    fn wma_rejects_a_zero_weight_sum() {
        weighted_moving_average(&decimals(&[1]), &[Decimal::ZERO]);
    }
}
