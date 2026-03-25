pub fn normalize_username(value: &str) -> String {
    value.trim().trim_start_matches('@').to_string()
}
