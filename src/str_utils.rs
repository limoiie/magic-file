use regex::Regex;

/// Return `Some(s)` if each char of `s` is a `[[:alnum:]]` or is in `goodchars`.
///
/// `goodchars` should not contain any chars in `[[:alnum:]]`
pub(crate) fn ensure_goodchars(s: &str, goodchars: &str) -> Option<String> {
    Some(
        Regex::new(
            format!(r"^[0-9A-Za-z{}]*$", goodchars).as_str()
        ).unwrap()
            .captures(s)?
            .get(0)?
            .as_str()
            .to_string()
    )
}

#[test]
fn test_parse_extra() {
    let raw = "hello+-0";
    let goodchars = "+-";
    assert_eq!(
        ensure_goodchars(raw, goodchars)
            .unwrap().as_str(),
        raw
    )
}
