use rust_decimal::Decimal;

use crate::ma::exponential_moving_average;

/// 单个时间点的 MACD 计算结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MacdItem {
    /// 差离值：快速 EMA - 慢速 EMA。
    pub dif: Decimal,
    /// 信号线：DIF 的 EMA。
    pub dea: Decimal,
    /// 柱状图：DIF - DEA。
    pub histogram: Decimal,
}

/// 指数平滑异同移动平均线MACD。
///
/// 使用以下公式：
///
/// - `DIF = EMA(prices, fast_period) - EMA(prices, slow_period)`
/// - `DEA = EMA(DIF, signal_period)`
/// - `histogram = DIF - DEA`
///
/// EMA 以第一个输入值作为初始值，因此返回值与 `prices` 等长且从第一个价格开始。
/// 柱状图未乘以 2；部分行情软件采用 `2 * (DIF - DEA)`，使用时需留意定义差异。
/// 任一周期为 0 或 `prices` 为空时返回空数组。
///
/// # 参数
///
/// - `prices`：按时间从旧到新排列的价格，一般为收盘价。
/// - `fast_period`：快速 EMA 周期，通常为 12。
/// - `slow_period`：慢速 EMA 周期，通常为 26。
/// - `signal_period`：信号线 EMA 周期，通常为 9。
pub fn moving_average_convergence_divergence(
    prices: &[Decimal],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Vec<MacdItem> {
    if prices.is_empty() || fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return vec![];
    }

    let fast_ema = exponential_moving_average(prices, fast_period);
    let slow_ema = exponential_moving_average(prices, slow_period);
    let dif: Vec<_> = fast_ema
        .iter()
        .zip(&slow_ema)
        .map(|(&fast, &slow)| fast - slow)
        .collect();

    let dea = exponential_moving_average(&dif, signal_period);

    dif.iter()
        .zip(&dea)
        .map(|(&dif, &dea)| MacdItem {
            dif,
            dea,
            histogram: dif - dea,
        })
        .collect()
}

/// 使用默认参数 `(12, 26, 9)` 计算 MACD。
pub fn moving_average_convergence_divergence_default(prices: &[Decimal]) -> Vec<MacdItem> {
    moving_average_convergence_divergence(prices, 12, 26, 9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimals(values: &[i64]) -> Vec<Decimal> {
        values.iter().copied().map(Decimal::from).collect()
    }

    #[test]
    fn calculates_dif_dea_and_histogram() {
        let prices = decimals(&[2, 4, 8, 4]);

        let actual = moving_average_convergence_divergence(&prices, 1, 3, 3);

        assert_eq!(
            actual,
            vec![
                MacdItem {
                    dif: Decimal::ZERO,
                    dea: Decimal::ZERO,
                    histogram: Decimal::ZERO,
                },
                MacdItem {
                    dif: Decimal::ONE,
                    dea: Decimal::new(5, 1),
                    histogram: Decimal::new(5, 1),
                },
                MacdItem {
                    dif: Decimal::new(25, 1),
                    dea: Decimal::new(15, 1),
                    histogram: Decimal::ONE,
                },
                MacdItem {
                    dif: Decimal::new(-75, 2),
                    dea: Decimal::new(375, 3),
                    histogram: Decimal::new(-1125, 3),
                },
            ]
        );
    }

    #[test]
    fn constant_prices_produce_zero_macd() {
        let prices = decimals(&[10, 10, 10, 10]);

        let actual = moving_average_convergence_divergence_default(&prices);

        assert_eq!(actual.len(), prices.len());
        assert!(actual.iter().all(|item| {
            item.dif == Decimal::ZERO
                && item.dea == Decimal::ZERO
                && item.histogram == Decimal::ZERO
        }));
    }

    #[test]
    fn default_function_uses_12_26_9_periods() {
        let prices = decimals(&[1, 2, 3, 4, 5]);

        assert_eq!(
            moving_average_convergence_divergence_default(&prices),
            moving_average_convergence_divergence(&prices, 12, 26, 9)
        );
    }

    #[test]
    fn empty_prices_or_zero_period_return_an_empty_result() {
        let prices = decimals(&[1, 2, 3]);

        assert!(moving_average_convergence_divergence(&[], 12, 26, 9).is_empty());
        assert!(moving_average_convergence_divergence(&prices, 0, 26, 9).is_empty());
        assert!(moving_average_convergence_divergence(&prices, 12, 0, 9).is_empty());
        assert!(moving_average_convergence_divergence(&prices, 12, 26, 0).is_empty());
    }
}
