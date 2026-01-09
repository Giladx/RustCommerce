/// Shared helpers for lightweight string munging.
pub fn slugify(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}
