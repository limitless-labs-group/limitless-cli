/// VWAP (Volume-Weighted Average Price) computation at dollar depths.
///
/// For a given target dollar amount, walk the orderbook levels and compute
/// the average fill price if you were to execute a market order of that size.

/// Result of VWAP computation at a specific dollar depth.
#[derive(Debug, Clone)]
pub struct VwapResult {
    /// Target dollar depth (e.g. 10, 50, 100, 200)
    pub depth_usd: f64,
    /// Average fill price if buying (walking asks ascending)
    pub vwap_buy: Option<f64>,
    /// Average fill price if selling (walking bids descending)
    pub vwap_sell: Option<f64>,
    /// Total shares that would be filled on buy side
    pub buy_shares: f64,
    /// Total shares that would be filled on sell side
    pub sell_shares: f64,
}

/// USDC has 6 decimals — sizes from the API are in raw units.
const USDC_SCALE: f64 = 1_000_000.0;

/// Compute VWAP for the buy side (walking asks in ascending order).
///
/// `asks` should be sorted ascending by price (best ask first).
/// Each entry is `(price, raw_size)` where raw_size is in USDC atomic units.
fn compute_vwap_one_side(levels: &[(f64, f64)], target_usd: f64) -> (Option<f64>, f64) {
    if levels.is_empty() || target_usd <= 0.0 {
        return (None, 0.0);
    }

    let mut remaining_usd = target_usd;
    let mut total_cost = 0.0;
    let mut total_shares = 0.0;

    for &(price, raw_size) in levels {
        if remaining_usd <= 0.0 || price <= 0.0 {
            break;
        }

        let scaled_size = raw_size / USDC_SCALE;
        let available_usd = scaled_size * price;
        let take_usd = remaining_usd.min(available_usd);
        let take_shares = take_usd / price;

        total_cost += take_usd;
        total_shares += take_shares;
        remaining_usd -= take_usd;
    }

    if total_shares > 0.0 {
        (Some(total_cost / total_shares), total_shares)
    } else {
        (None, 0.0)
    }
}

/// Compute VWAPs at multiple dollar depths.
///
/// `bids` and `asks` are `(price, raw_size)` tuples.
/// `bids` should be sorted descending (best bid first).
/// `asks` should be sorted ascending (best ask first).
pub fn compute_vwaps(
    bids: &[(f64, f64)],
    asks: &[(f64, f64)],
    depths: &[f64],
) -> Vec<VwapResult> {
    depths
        .iter()
        .map(|&depth_usd| {
            let (vwap_buy, buy_shares) = compute_vwap_one_side(asks, depth_usd);
            let (vwap_sell, sell_shares) = compute_vwap_one_side(bids, depth_usd);

            VwapResult {
                depth_usd,
                vwap_buy,
                vwap_sell,
                buy_shares,
                sell_shares,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_empty_book() {
        let results = compute_vwaps(&[], &[], &[10.0, 50.0]);
        assert_eq!(results.len(), 2);
        assert!(results[0].vwap_buy.is_none());
        assert!(results[0].vwap_sell.is_none());
    }

    #[test]
    fn test_vwap_single_level() {
        // Ask at $0.65 with 100 USDC worth of size (100 * 1e6 raw)
        let asks = vec![(0.65, 100.0 * USDC_SCALE)];
        let bids = vec![(0.60, 100.0 * USDC_SCALE)];

        let results = compute_vwaps(&bids, &asks, &[10.0]);
        assert_eq!(results.len(), 1);

        // Buying $10 at $0.65 → VWAP = $0.65
        let buy_vwap = results[0].vwap_buy.unwrap();
        assert!((buy_vwap - 0.65).abs() < 0.0001);

        // Selling $10 at $0.60 → VWAP = $0.60
        let sell_vwap = results[0].vwap_sell.unwrap();
        assert!((sell_vwap - 0.60).abs() < 0.0001);
    }

    #[test]
    fn test_vwap_multiple_levels() {
        // Asks: $0.65 (50 shares), $0.70 (50 shares)
        let asks = vec![
            (0.65, 50.0 * USDC_SCALE),
            (0.70, 50.0 * USDC_SCALE),
        ];

        // Buy $50 worth: fills 50*0.65 = $32.50 at first level, remainder at $0.70
        let results = compute_vwaps(&[], &asks, &[50.0]);
        let vwap = results[0].vwap_buy.unwrap();
        // Should be between 0.65 and 0.70
        assert!(vwap > 0.65 && vwap < 0.70);
    }

    #[test]
    fn test_vwap_insufficient_depth() {
        // Only $5 worth of liquidity at $0.65
        let asks = vec![(0.65, 5.0 * USDC_SCALE / 0.65)];

        let results = compute_vwaps(&[], &asks, &[100.0]);
        // Should still return a VWAP (partial fill)
        assert!(results[0].vwap_buy.is_some());
    }
}
