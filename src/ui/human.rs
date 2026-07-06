pub(crate) fn lines(items: &[(&str, String)]) -> Vec<String> {
    items
        .iter()
        .map(|(key, value)| format!("{key}: {value}"))
        .collect()
}
