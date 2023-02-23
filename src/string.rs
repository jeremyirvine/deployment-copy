pub fn truncate(s: impl Into<String>, max_chars: usize) -> String {
    let s: String = s.into();
    let s = s.as_str();
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => s[..idx].to_string(),
    }
}