pub(crate) fn apply_env_overrides(value: &mut toml::Value) {
    apply_overrides_from(value, std::env::vars());
}

pub(crate) fn apply_overrides_from(
    value: &mut toml::Value,
    vars: impl Iterator<Item = (String, String)>,
) {
    for (key, val_str) in vars {
        if let Some(path) = parse_env_key(&key) {
            set_nested(value, &path, coerce_value(val_str));
        }
    }
}

/// Converts `SECTION__KEY` or `A__B__C` into `["section", "key"]` / `["a", "b", "c"]`.
/// Returns `None` if the key has no `__` or has empty segments.
fn parse_env_key(key: &str) -> Option<Vec<String>> {
    if !key.contains("__") {
        return None;
    }
    let parts: Vec<String> = key.split("__").map(|s| s.to_lowercase()).collect();
    if parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    Some(parts)
}

/// Tries to parse the string as i64, then f64, then bool; falls back to String.
fn coerce_value(s: String) -> toml::Value {
    if let Ok(n) = s.parse::<i64>() {
        return toml::Value::Integer(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return toml::Value::Float(f);
    }
    match s.to_lowercase().as_str() {
        "true" => return toml::Value::Boolean(true),
        "false" => return toml::Value::Boolean(false),
        _ => {}
    }
    toml::Value::String(s)
}

fn set_nested(value: &mut toml::Value, path: &[String], val: toml::Value) {
    if let toml::Value::Table(map) = value {
        if path.len() == 1 {
            map.insert(path[0].clone(), val);
        } else {
            let child = map
                .entry(path[0].clone())
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            set_nested(child, &path[1..], val);
        }
    }
}

#[cfg(test)]
mod tests {
    fn empty_table() -> toml::Value {
        toml::Value::Table(toml::map::Map::new())
    }

    #[test]
    fn test_sets_integer_value() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![("SERVER__PORT".to_string(), "9090".to_string())].into_iter(),
        );
        assert_eq!(v["server"]["port"].as_integer(), Some(9090));
    }

    #[test]
    fn test_sets_string_value() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![("LOG__LEVEL".to_string(), "debug".to_string())].into_iter(),
        );
        assert_eq!(v["log"]["level"].as_str(), Some("debug"));
    }

    #[test]
    fn test_sets_boolean_value() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![("VIEW__HOT_RELOAD".to_string(), "false".to_string())].into_iter(),
        );
        assert_eq!(v["view"]["hot_reload"].as_bool(), Some(false));
    }

    #[test]
    fn test_ignores_single_underscore_vars() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![
                ("DOIDO_ENV".to_string(), "test".to_string()),
                ("PATH".to_string(), "/usr/bin".to_string()),
            ].into_iter(),
        );
        assert!(v.as_table().unwrap().is_empty());
    }

    #[test]
    fn test_ignores_empty_segment_from_trailing_double_underscore() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![("SERVER__".to_string(), "foo".to_string())].into_iter(),
        );
        assert!(v.as_table().unwrap().is_empty());
    }

    #[test]
    fn test_supports_three_level_nesting() {
        let mut v = empty_table();
        super::apply_overrides_from(
            &mut v,
            vec![("A__B__C".to_string(), "42".to_string())].into_iter(),
        );
        assert_eq!(v["a"]["b"]["c"].as_integer(), Some(42));
    }

    #[test]
    fn test_overrides_existing_value() {
        let mut v: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        super::apply_overrides_from(
            &mut v,
            vec![("SERVER__PORT".to_string(), "8080".to_string())].into_iter(),
        );
        assert_eq!(v["server"]["port"].as_integer(), Some(8080));
    }
}
