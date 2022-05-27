use crate::check_pairs;

#[test]
fn check_valid_pairs() {
    let signle = check_pairs("btc_usdt");
    assert!(signle);
    let multiple = check_pairs("btc_usdt,eth_usdt");
    assert!(multiple);
}

#[test]
fn check_invalid_pairs_() {
    let single = check_pairs("btcusdt");
    assert!(!single);
    let multiple = check_pairs("btcusdt,eth_usdt");
    assert!(!multiple);
}