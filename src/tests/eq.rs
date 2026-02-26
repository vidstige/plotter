use crate::eq::linesearch;

#[test]
fn linesearch_uses_full_interval() {
    let root = 0.95_f32;
    let bracket = linesearch(|t| t - root, 0.1, 1.0, 9);
    assert!(bracket.is_some(), "expected root bracket in [0.1, 1.0]");

    let (lo, hi) = bracket.unwrap();
    assert!(lo >= 0.1 && hi <= 1.0, "bracket should stay within search interval");
    assert!(lo <= root && root <= hi, "bracket should contain the root");
}

#[test]
fn linesearch_with_zero_steps_returns_none() {
    let bracket = linesearch(|t| t - 1.0, 0.0, 2.0, 0);
    assert!(bracket.is_none(), "expected no bracket when steps == 0");
}
