use augustinus_tui::widgets::big_text::BigText;

#[test]
fn renders_fixed_height() {
    let t = BigText::new("LOCK IN");
    let lines = t.lines();
    assert_eq!(lines.len(), 5);
}

#[test]
fn supports_digits() {
    let t = BigText::new("7");
    assert!(t.lines().iter().any(|l| l.contains('#') || l.contains('█')));
}
