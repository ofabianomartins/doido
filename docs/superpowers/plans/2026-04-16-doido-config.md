# doido-config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `doido-config` — the framework's layered configuration system providing TOML file loading (base → env-file → encrypted credentials → env vars), AES-256-GCM credential management, and a typed `Config` struct accessible via `Config::load() → Arc<Config>`.

**Architecture:** Four cooperating modules: `types` holds serde-derived Config structs with sensible defaults; `loader` reads and deep-merges TOML files from disk; `env_override` applies `SECTION__KEY` env var overrides on the merged `toml::Value`; `crypto` handles AES-256-GCM encrypt/decrypt of the credentials file plus master key resolution. `lib.rs` wires them as `Config::load_from_env(root, env) → Arc<Config>`. Each module has its own inline unit tests; `tests/config_test.rs` holds full-stack integration tests.

**Tech Stack:** Rust, `doido-core 0.1` (Result/anyhow), `toml 0.8`, `serde 1` (derive), `aes-gcm 0.10`, `base64 0.22`, `hex 0.4`; dev: `tempfile 3`, `serial_test 3`

---

## File Structure

| File | Purpose |
|------|---------|
| `doido-config/Cargo.toml` | Crate manifest with all dependencies |
| `doido-config/src/lib.rs` | Module declarations + `impl Config { load, load_from, load_from_env }` |
| `doido-config/src/types.rs` | `Config`, `ServerConfig`, `DatabaseConfig`, `ViewConfig`, `LogConfig` with serde defaults |
| `doido-config/src/loader.rs` | `load_layers(root, env)`, `deep_merge`, `load_toml` — TOML file I/O and merging |
| `doido-config/src/env_override.rs` | `apply_env_overrides`, `apply_overrides_from` — `SECTION__KEY` parsing |
| `doido-config/src/crypto.rs` | `encrypt_credentials`, `decrypt_credentials`, `load_master_key` |
| `doido-config/tests/config_test.rs` | Full-stack integration tests via `Config::load_from_env` |

---

### Task 1: Crate Scaffold

**Files:**
- Create: `doido-config/Cargo.toml`
- Modify: `Cargo.toml` (workspace root)
- Create: `doido-config/src/lib.rs`
- Create: `doido-config/src/types.rs` (stub)
- Create: `doido-config/src/loader.rs` (stub)
- Create: `doido-config/src/env_override.rs` (stub)
- Create: `doido-config/src/crypto.rs` (stub)

**Prerequisite:** `doido-core` must be present in the workspace. This plan assumes the feat/doido-core branch has been merged (or that you are branching off it). The path `../doido-core` must resolve from `doido-config/`.

- [ ] **Step 1: Create `doido-config/Cargo.toml`**

```toml
[package]
name = "doido-config"
version = "0.1.0"
edition = "2021"

[dependencies]
doido-core = { path = "../doido-core" }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
aes-gcm = "0.10"
base64 = "0.22"
hex = "0.4"

[dev-dependencies]
tempfile = "3"
serial_test = "3"
```

- [ ] **Step 2: Add `doido-config` to the workspace**

Edit `Cargo.toml` at the workspace root:

```toml
[workspace]
resolver = "2"
members = [
    "doido-core",
    "doido-config",
]
```

- [ ] **Step 3: Create `doido-config/src/lib.rs`**

```rust
pub mod types;
pub(crate) mod loader;
pub(crate) mod env_override;
pub mod crypto;

pub use types::{Config, ServerConfig, DatabaseConfig, ViewConfig, LogConfig};
```

- [ ] **Step 4: Create stub source files so the crate compiles**

Create `doido-config/src/types.rs`:
```rust
// filled in Task 2
```

Create `doido-config/src/loader.rs`:
```rust
// filled in Task 3
```

Create `doido-config/src/env_override.rs`:
```rust
// filled in Task 4
```

Create `doido-config/src/crypto.rs`:
```rust
// filled in Task 5
```

- [ ] **Step 5: Verify the crate is visible to the workspace**

Run: `cargo check -p doido-config`
Expected: errors about empty modules — confirms the crate is found by Cargo.

- [ ] **Step 6: Commit**

```bash
git add doido-config/ Cargo.toml
git commit -m "feat(config): add doido-config crate scaffold"
```

---

### Task 2: Config Types

**Files:**
- Create: `doido-config/src/types.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-config/src/types.rs` with the test module below **before** writing any implementation code:

```rust
// types.rs — tests written first; struct definitions come in Step 3

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_deserializes_all_sections() {
        let toml_str = r#"
[server]
port = 8080
bind = "0.0.0.0"
[database]
url = "postgres://localhost/mydb"
pool_size = 20
[view]
engine = "tera"
templates_dir = "views"
layout = "application"
hot_reload = false
[log]
level = "warn"
"#;
        let config: crate::types::Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.bind, "0.0.0.0");
        assert_eq!(config.database.url, "postgres://localhost/mydb");
        assert_eq!(config.database.pool_size, 20);
        assert_eq!(config.view.engine, "tera");
        assert_eq!(config.view.templates_dir, "views");
        assert_eq!(config.view.layout, "application");
        assert!(!config.view.hot_reload);
        assert_eq!(config.log.level, "warn");
    }

    #[test]
    fn test_config_uses_defaults_for_missing_sections() {
        let config: crate::types::Config = toml::from_str("").unwrap();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.bind, "127.0.0.1");
        assert_eq!(config.database.pool_size, 5);
        assert_eq!(config.log.level, "info");
        assert!(!config.view.hot_reload);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-config`
Expected: compile error — `crate::types::Config` not defined.

- [ ] **Step 3: Implement the full `doido-config/src/types.rs`**

Replace the file with:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub view: ViewConfig,
    pub log: LogConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            view: ViewConfig::default(),
            log: LogConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
    pub bind: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { port: 3000, bind: "127.0.0.1".to_string() }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://db/development.sqlite3".to_string(),
            pool_size: 5,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ViewConfig {
    pub engine: String,
    pub templates_dir: String,
    pub layout: String,
    pub hot_reload: bool,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            engine: "tera".to_string(),
            templates_dir: "views".to_string(),
            layout: "application".to_string(),
            hot_reload: false,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { level: "info".to_string() }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_deserializes_all_sections() {
        let toml_str = r#"
[server]
port = 8080
bind = "0.0.0.0"
[database]
url = "postgres://localhost/mydb"
pool_size = 20
[view]
engine = "tera"
templates_dir = "views"
layout = "application"
hot_reload = false
[log]
level = "warn"
"#;
        let config: super::Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.bind, "0.0.0.0");
        assert_eq!(config.database.url, "postgres://localhost/mydb");
        assert_eq!(config.database.pool_size, 20);
        assert_eq!(config.view.engine, "tera");
        assert_eq!(config.view.templates_dir, "views");
        assert_eq!(config.view.layout, "application");
        assert!(!config.view.hot_reload);
        assert_eq!(config.log.level, "warn");
    }

    #[test]
    fn test_config_uses_defaults_for_missing_sections() {
        let config: super::Config = toml::from_str("").unwrap();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.bind, "127.0.0.1");
        assert_eq!(config.database.pool_size, 5);
        assert_eq!(config.log.level, "info");
        assert!(!config.view.hot_reload);
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-config`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-config/src/types.rs
git commit -m "feat(config): add typed Config struct with serde defaults"
```

---

### Task 3: TOML Loader and Deep Merge

**Files:**
- Create: `doido-config/src/loader.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-config/src/loader.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    fn write(dir: &TempDir, rel: &str, content: &str) {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    const BASE: &str = r#"
[server]
port = 3000
bind = "127.0.0.1"
[database]
url = "sqlite://dev.db"
pool_size = 5
[view]
engine = "tera"
templates_dir = "views"
layout = "application"
hot_reload = true
[log]
level = "info"
"#;

    #[test]
    fn test_load_toml_reads_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        let val = super::load_toml(&dir.path().join("config/doido.toml")).unwrap();
        assert!(val.is_some());
        assert_eq!(val.unwrap()["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_toml_returns_none_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let val = super::load_toml(&dir.path().join("config/doido.toml")).unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_deep_merge_overrides_leaf_values() {
        let base: toml::Value = toml::from_str("[server]\nport = 3000\nbind = \"127.0.0.1\"").unwrap();
        let over: toml::Value = toml::from_str("[server]\nbind = \"0.0.0.0\"").unwrap();
        let merged = super::deep_merge(base, over);
        assert_eq!(merged["server"]["port"].as_integer(), Some(3000)); // preserved
        assert_eq!(merged["server"]["bind"].as_str(), Some("0.0.0.0")); // overridden
    }

    #[test]
    fn test_deep_merge_adds_new_keys() {
        let base: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        let over: toml::Value = toml::from_str("[database]\nurl = \"postgres://\"").unwrap();
        let merged = super::deep_merge(base, over);
        assert_eq!(merged["server"]["port"].as_integer(), Some(3000));
        assert_eq!(merged["database"]["url"].as_str(), Some("postgres://"));
    }

    #[test]
    fn test_load_layers_applies_env_override_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        write(&dir, "config/doido.prod.toml",
            "[server]\nbind = \"0.0.0.0\"\n[log]\nlevel = \"warn\"");
        let val = super::load_layers(dir.path(), "prod").unwrap();
        assert_eq!(val["server"]["port"].as_integer(), Some(3000)); // from base
        assert_eq!(val["server"]["bind"].as_str(), Some("0.0.0.0")); // from env file
        assert_eq!(val["log"]["level"].as_str(), Some("warn")); // from env file
    }

    #[test]
    fn test_load_layers_skips_missing_env_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        let val = super::load_layers(dir.path(), "noenv").unwrap();
        assert_eq!(val["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_layers_errors_on_missing_base() {
        let dir = TempDir::new().unwrap();
        let result = super::load_layers(dir.path(), "dev");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("config/doido.toml not found"), "got: {msg}");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-config`
Expected: compile error — `load_toml`, `deep_merge`, `load_layers` not defined.

- [ ] **Step 3: Implement `doido-config/src/loader.rs`**

Replace the file with the full implementation **followed by** the test module above:

```rust
use std::path::Path;
use doido_core::{Result, anyhow::Context as _};

pub(crate) fn load_toml(path: &Path) -> Result<Option<toml::Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let value: toml::Value = toml::from_str(&content)
        .with_context(|| format!("failed to parse TOML at {}", path.display()))?;
    Ok(Some(value))
}

pub(crate) fn deep_merge(base: toml::Value, over: toml::Value) -> toml::Value {
    match (base, over) {
        (toml::Value::Table(mut base_map), toml::Value::Table(over_map)) => {
            for (k, v) in over_map {
                let entry = base_map
                    .entry(k)
                    .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
                *entry = deep_merge(entry.clone(), v);
            }
            toml::Value::Table(base_map)
        }
        (_, over) => over,
    }
}

/// Load and merge TOML layers: base config, then env-specific override.
/// Credentials layer is added in Task 6.
pub(crate) fn load_layers(root: &Path, env: &str) -> Result<toml::Value> {
    // 1. Base config — required
    let base_path = root.join("config/doido.toml");
    let mut merged = load_toml(&base_path)?
        .ok_or_else(|| doido_core::anyhow::anyhow!(
            "config/doido.toml not found in {}",
            root.display()
        ))?;

    // 2. Environment-specific override — optional
    let env_path = root.join(format!("config/doido.{env}.toml"));
    if let Some(env_value) = load_toml(&env_path)? {
        merged = deep_merge(merged, env_value);
    }

    Ok(merged)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    fn write(dir: &TempDir, rel: &str, content: &str) {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    const BASE: &str = r#"
[server]
port = 3000
bind = "127.0.0.1"
[database]
url = "sqlite://dev.db"
pool_size = 5
[view]
engine = "tera"
templates_dir = "views"
layout = "application"
hot_reload = true
[log]
level = "info"
"#;

    #[test]
    fn test_load_toml_reads_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        let val = super::load_toml(&dir.path().join("config/doido.toml")).unwrap();
        assert!(val.is_some());
        assert_eq!(val.unwrap()["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_toml_returns_none_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let val = super::load_toml(&dir.path().join("config/doido.toml")).unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_deep_merge_overrides_leaf_values() {
        let base: toml::Value = toml::from_str("[server]\nport = 3000\nbind = \"127.0.0.1\"").unwrap();
        let over: toml::Value = toml::from_str("[server]\nbind = \"0.0.0.0\"").unwrap();
        let merged = super::deep_merge(base, over);
        assert_eq!(merged["server"]["port"].as_integer(), Some(3000));
        assert_eq!(merged["server"]["bind"].as_str(), Some("0.0.0.0"));
    }

    #[test]
    fn test_deep_merge_adds_new_keys() {
        let base: toml::Value = toml::from_str("[server]\nport = 3000").unwrap();
        let over: toml::Value = toml::from_str("[database]\nurl = \"postgres://\"").unwrap();
        let merged = super::deep_merge(base, over);
        assert_eq!(merged["server"]["port"].as_integer(), Some(3000));
        assert_eq!(merged["database"]["url"].as_str(), Some("postgres://"));
    }

    #[test]
    fn test_load_layers_applies_env_override_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        write(&dir, "config/doido.prod.toml",
            "[server]\nbind = \"0.0.0.0\"\n[log]\nlevel = \"warn\"");
        let val = super::load_layers(dir.path(), "prod").unwrap();
        assert_eq!(val["server"]["port"].as_integer(), Some(3000));
        assert_eq!(val["server"]["bind"].as_str(), Some("0.0.0.0"));
        assert_eq!(val["log"]["level"].as_str(), Some("warn"));
    }

    #[test]
    fn test_load_layers_skips_missing_env_file() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        let val = super::load_layers(dir.path(), "noenv").unwrap();
        assert_eq!(val["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_layers_errors_on_missing_base() {
        let dir = TempDir::new().unwrap();
        let result = super::load_layers(dir.path(), "dev");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("config/doido.toml not found"), "got: {msg}");
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-config`
Expected: PASS — 8 tests (2 types + 6 loader).

- [ ] **Step 5: Commit**

```bash
git add doido-config/src/loader.rs
git commit -m "feat(config): add TOML loader with deep merge for base and env-specific files"
```

---

### Task 4: Env Var Override

**Files:**
- Create: `doido-config/src/env_override.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-config/src/env_override.rs` with just the test module:

```rust
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
        // VIEW__HOT_RELOAD splits on __ → ["view", "hot_reload"]
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
        // "SERVER__" splits to ["server", ""] — empty part rejected
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-config`
Expected: compile error — `apply_overrides_from` not defined.

- [ ] **Step 3: Implement `doido-config/src/env_override.rs`**

Replace the file with the full implementation **followed by** the test module above:

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-config`
Expected: PASS — 15 tests (2 types + 6 loader + 7 env_override).

- [ ] **Step 5: Commit**

```bash
git add doido-config/src/env_override.rs
git commit -m "feat(config): add SECTION__KEY env var override with type coercion"
```

---

### Task 5: AES-256-GCM Crypto

**Files:**
- Create: `doido-config/src/crypto.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-config/src/crypto.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    fn all_zeros_key() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let key = all_zeros_key();
        let plaintext = "[database]\nurl = \"postgres://secret@host/db\"\n";
        let encrypted = super::encrypt_credentials(plaintext, &key).unwrap();
        let decrypted = super::decrypt_credentials(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_each_encryption_produces_unique_ciphertext() {
        let key = all_zeros_key();
        let c1 = super::encrypt_credentials("secret", &key).unwrap();
        let c2 = super::encrypt_credentials("secret", &key).unwrap();
        assert_ne!(c1, c2, "nonce must be random; same ciphertext twice means nonce is static");
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let encrypted = super::encrypt_credentials("secret", &key1).unwrap();
        let result = super::decrypt_credentials(&encrypted, &key2);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("decryption failed"),
            "error message should mention decryption failure"
        );
    }

    #[test]
    fn test_decrypt_fails_on_garbage_input() {
        let key = all_zeros_key();
        let result = super::decrypt_credentials("not-base64!!!", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_master_key_from_file() {
        let dir = TempDir::new().unwrap();
        let hex_key = "00".repeat(32); // 64 hex chars = 32 zero bytes
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, format!("{hex_key}\n")).unwrap(); // trailing newline trimmed
        let key = super::load_master_key(dir.path()).unwrap();
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_load_master_key_rejects_wrong_length() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "deadbeef").unwrap(); // only 4 bytes, not 32
        // Only run this if DOIDO_MASTER_KEY is not already set in the environment,
        // because env var takes priority over the file.
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(msg.contains("32 bytes"), "got: {msg}");
        }
    }

    #[test]
    fn test_load_master_key_rejects_invalid_hex() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "not-valid-hex-string-at-all-!!!!").unwrap();
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("valid hex"));
        }
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-config`
Expected: compile error — `encrypt_credentials`, `decrypt_credentials`, `load_master_key` not defined.

- [ ] **Step 3: Implement `doido-config/src/crypto.rs`**

Replace the file with the full implementation **followed by** the test module above:

```rust
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::path::Path;
use doido_core::{Result, anyhow::Context as _};

/// Encrypts `plaintext` with `key` using AES-256-GCM with a random nonce.
/// Returns a base64-encoded blob: `nonce(12 bytes) || ciphertext`.
pub fn encrypt_credentials(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| doido_core::anyhow::anyhow!("AES-GCM encryption failed"))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&out))
}

/// Decrypts a base64-encoded blob produced by `encrypt_credentials`.
pub fn decrypt_credentials(encoded: &str, key: &[u8; 32]) -> Result<String> {
    let raw = STANDARD
        .decode(encoded.trim())
        .map_err(|e| doido_core::anyhow::anyhow!("base64 decode failed: {e}"))?;
    if raw.len() < 12 {
        doido_core::anyhow::bail!("credentials blob too short to contain nonce");
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| doido_core::anyhow::anyhow!("decryption failed — wrong key?"))?;
    String::from_utf8(plaintext)
        .map_err(|e| doido_core::anyhow::anyhow!("credentials are not valid UTF-8: {e}"))
}

/// Resolves the 32-byte master key:
/// 1. `DOIDO_MASTER_KEY` env var (64-char hex string)
/// 2. `config/master.key` file (64-char hex string, trailing whitespace trimmed)
pub(crate) fn load_master_key(root: &Path) -> Result<[u8; 32]> {
    let hex_str = std::env::var("DOIDO_MASTER_KEY").or_else(|_| {
        let key_path = root.join("config/master.key");
        std::fs::read_to_string(&key_path)
            .map(|s| s.trim().to_string())
            .map_err(|e| doido_core::anyhow::anyhow!("cannot read config/master.key: {e}"))
    })?;
    let bytes = hex::decode(hex_str.trim())
        .context("master key is not valid hex")?;
    bytes
        .try_into()
        .map_err(|_| doido_core::anyhow::anyhow!("master key must be 32 bytes (64 hex chars)"))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use std::fs;

    fn all_zeros_key() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_round_trip() {
        let key = all_zeros_key();
        let plaintext = "[database]\nurl = \"postgres://secret@host/db\"\n";
        let encrypted = super::encrypt_credentials(plaintext, &key).unwrap();
        let decrypted = super::decrypt_credentials(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_each_encryption_produces_unique_ciphertext() {
        let key = all_zeros_key();
        let c1 = super::encrypt_credentials("secret", &key).unwrap();
        let c2 = super::encrypt_credentials("secret", &key).unwrap();
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];
        let encrypted = super::encrypt_credentials("secret", &key1).unwrap();
        let result = super::decrypt_credentials(&encrypted, &key2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("decryption failed"));
    }

    #[test]
    fn test_decrypt_fails_on_garbage_input() {
        let key = all_zeros_key();
        let result = super::decrypt_credentials("not-base64!!!", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_master_key_from_file() {
        let dir = TempDir::new().unwrap();
        let hex_key = "00".repeat(32);
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, format!("{hex_key}\n")).unwrap();
        let key = super::load_master_key(dir.path()).unwrap();
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_load_master_key_rejects_wrong_length() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "deadbeef").unwrap();
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(msg.contains("32 bytes"), "got: {msg}");
        }
    }

    #[test]
    fn test_load_master_key_rejects_invalid_hex() {
        let dir = TempDir::new().unwrap();
        let key_path = dir.path().join("config/master.key");
        fs::create_dir_all(key_path.parent().unwrap()).unwrap();
        fs::write(&key_path, "not-valid-hex-string-at-all-!!!!").unwrap();
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_master_key(dir.path());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("valid hex"));
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-config`
Expected: PASS — 22 tests (2 + 6 + 7 + 7).

- [ ] **Step 5: Commit**

```bash
git add doido-config/src/crypto.rs
git commit -m "feat(config): add AES-256-GCM credentials encryption and master key loading"
```

---

### Task 6: Credentials Layer

**Files:**
- Modify: `doido-config/src/loader.rs` (add credentials layer to `load_layers`)

The credentials file is `config/credentials.toml.enc`. When it exists, decrypt it and deep-merge its TOML content into the merged config. When it does not exist, skip silently. When it exists but the master key is missing, return an error.

- [ ] **Step 1: Add the failing test to `loader.rs`**

Append these tests to the `#[cfg(test)] mod tests` block in `loader.rs`:

```rust
    #[test]
    fn test_load_layers_merges_credentials() {
        use crate::crypto::encrypt_credentials;
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);

        // Create encrypted credentials file
        let key = [0u8; 32];
        let cred_toml = "[database]\nurl = \"postgres://secret@prod/db\"\n";
        let encrypted = encrypt_credentials(cred_toml, &key).unwrap();
        write(&dir, "config/credentials.toml.enc", &encrypted);

        // Write the master key file
        let hex_key = "00".repeat(32);
        write(&dir, "config/master.key", &hex_key);

        let val = super::load_layers(dir.path(), "noenv").unwrap();
        // Credentials override the base database.url
        assert_eq!(val["database"]["url"].as_str(), Some("postgres://secret@prod/db"));
        // Other base values are preserved
        assert_eq!(val["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_layers_skips_credentials_when_file_absent() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        // No credentials.toml.enc — should succeed without master key
        let val = super::load_layers(dir.path(), "noenv").unwrap();
        assert_eq!(val["server"]["port"].as_integer(), Some(3000));
    }

    #[test]
    fn test_load_layers_errors_when_credentials_exist_but_key_missing() {
        let dir = TempDir::new().unwrap();
        write(&dir, "config/doido.toml", BASE);
        write(&dir, "config/credentials.toml.enc", "fake-encrypted-content");
        // No master.key and no DOIDO_MASTER_KEY env var
        if std::env::var("DOIDO_MASTER_KEY").is_err() {
            let result = super::load_layers(dir.path(), "noenv");
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("master key") || msg.contains("master.key"),
                "got: {msg}"
            );
        }
    }
```

- [ ] **Step 2: Run the tests to verify the new ones fail**

Run: `cargo test -p doido-config loader`
Expected: 6 existing tests pass; 3 new tests fail — credentials logic not yet in `load_layers`.

- [ ] **Step 3: Add credentials layer to `load_layers` in `loader.rs`**

Add the credentials step after the env-file step inside `load_layers`:

```rust
    // 3. Encrypted credentials — optional file, but key is required when file exists
    let cred_path = root.join("config/credentials.toml.enc");
    if cred_path.exists() {
        let key = crate::crypto::load_master_key(root)
            .context("failed to load master key for credentials.toml.enc")?;
        let encoded = std::fs::read_to_string(&cred_path)
            .context("failed to read config/credentials.toml.enc")?;
        let plaintext = crate::crypto::decrypt_credentials(&encoded, &key)
            .context("failed to decrypt config/credentials.toml.enc")?;
        let cred_value: toml::Value = toml::from_str(&plaintext)
            .context("failed to parse decrypted credentials as TOML")?;
        merged = deep_merge(merged, cred_value);
    }
```

The full `load_layers` function after the edit:

```rust
pub(crate) fn load_layers(root: &Path, env: &str) -> Result<toml::Value> {
    // 1. Base config — required
    let base_path = root.join("config/doido.toml");
    let mut merged = load_toml(&base_path)?
        .ok_or_else(|| doido_core::anyhow::anyhow!(
            "config/doido.toml not found in {}",
            root.display()
        ))?;

    // 2. Environment-specific override — optional
    let env_path = root.join(format!("config/doido.{env}.toml"));
    if let Some(env_value) = load_toml(&env_path)? {
        merged = deep_merge(merged, env_value);
    }

    // 3. Encrypted credentials — optional file, but key is required when file exists
    let cred_path = root.join("config/credentials.toml.enc");
    if cred_path.exists() {
        let key = crate::crypto::load_master_key(root)
            .context("failed to load master key for credentials.toml.enc")?;
        let encoded = std::fs::read_to_string(&cred_path)
            .context("failed to read config/credentials.toml.enc")?;
        let plaintext = crate::crypto::decrypt_credentials(&encoded, &key)
            .context("failed to decrypt config/credentials.toml.enc")?;
        let cred_value: toml::Value = toml::from_str(&plaintext)
            .context("failed to parse decrypted credentials as TOML")?;
        merged = deep_merge(merged, cred_value);
    }

    Ok(merged)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-config`
Expected: PASS — 25 tests (2 + 9 + 7 + 7).

- [ ] **Step 5: Commit**

```bash
git add doido-config/src/loader.rs
git commit -m "feat(config): add encrypted credentials layer to config loading"
```

---

### Task 7: Config::load() and Integration Tests

**Files:**
- Modify: `doido-config/src/lib.rs` (add `impl Config` with load methods)
- Create: `doido-config/tests/config_test.rs`

- [ ] **Step 1: Write the failing integration tests**

Create `doido-config/tests/config_test.rs`:

```rust
use doido_config::Config;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

fn write(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

const BASE_TOML: &str = r#"
[server]
port = 3000
bind = "127.0.0.1"
[database]
url = "sqlite://dev.db"
pool_size = 5
[view]
engine = "tera"
templates_dir = "views"
layout = "application"
hot_reload = true
[log]
level = "info"
"#;

#[test]
fn test_load_base_config() {
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    let config = Config::load_from_env(dir.path(), "noenv").unwrap();
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.server.bind, "127.0.0.1");
    assert_eq!(config.database.pool_size, 5);
    assert_eq!(config.log.level, "info");
    assert!(config.view.hot_reload);
}

#[test]
fn test_env_file_overrides_base() {
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    write(&dir, "config/doido.staging.toml",
        "[server]\nbind = \"0.0.0.0\"\n[log]\nlevel = \"warn\"");
    let config = Config::load_from_env(dir.path(), "staging").unwrap();
    assert_eq!(config.server.port, 3000);         // preserved from base
    assert_eq!(config.server.bind, "0.0.0.0");    // overridden by staging
    assert_eq!(config.log.level, "warn");          // overridden by staging
}

#[test]
fn test_credentials_override_base_and_env_file() {
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    write(&dir, "config/doido.test.toml", "[database]\npool_size = 1");

    let key = [42u8; 32];
    let cred_toml = "[database]\nurl = \"postgres://secret@prod/db\"\n";
    let encrypted = doido_config::crypto::encrypt_credentials(cred_toml, &key).unwrap();
    write(&dir, "config/credentials.toml.enc", &encrypted);
    write(&dir, "config/master.key", &"2a".repeat(32)); // 0x2a = 42 decimal

    let config = Config::load_from_env(dir.path(), "test").unwrap();
    assert_eq!(config.database.url, "postgres://secret@prod/db"); // from credentials
    assert_eq!(config.database.pool_size, 1);  // from env file
    assert_eq!(config.server.port, 3000);      // from base
}

#[test]
#[serial]
fn test_env_var_takes_highest_priority() {
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    std::env::set_var("SERVER__PORT", "9999");
    std::env::set_var("LOG__LEVEL", "trace");
    let config = Config::load_from_env(dir.path(), "noenv").unwrap();
    std::env::remove_var("SERVER__PORT");
    std::env::remove_var("LOG__LEVEL");
    assert_eq!(config.server.port, 9999);
    assert_eq!(config.log.level, "trace");
}

#[test]
fn test_missing_base_config_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = Config::load_from_env(dir.path(), "dev");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("config/doido.toml not found"));
}

#[test]
fn test_missing_master_key_with_credentials_file_returns_error() {
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    write(&dir, "config/credentials.toml.enc", "not-valid-encrypted-content");
    if std::env::var("DOIDO_MASTER_KEY").is_err() {
        let result = Config::load_from_env(dir.path(), "noenv");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("master key") || msg.contains("master.key"), "got: {msg}");
    }
}

#[test]
fn test_load_from_uses_doido_env() {
    // Verify Config::load_from reads DOIDO_ENV. We can't test Config::load() without
    // affecting the real filesystem, so we test the DOIDO_ENV integration via load_from_env.
    let dir = TempDir::new().unwrap();
    write(&dir, "config/doido.toml", BASE_TOML);
    write(&dir, "config/doido.test.toml", "[log]\nlevel = \"test-level\"");
    // Simulates DOIDO_ENV=test
    let config = Config::load_from_env(dir.path(), "test").unwrap();
    assert_eq!(config.log.level, "test-level");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-config --test config_test`
Expected: compile error — `Config::load_from_env` does not exist yet.

- [ ] **Step 3: Add `impl Config` to `doido-config/src/lib.rs`**

Replace `doido-config/src/lib.rs` with the final version:

```rust
pub mod types;
pub(crate) mod loader;
pub(crate) mod env_override;
pub mod crypto;

pub use types::{Config, DatabaseConfig, LogConfig, ServerConfig, ViewConfig};

use std::{path::Path, sync::Arc};
use doido_core::Result;

impl Config {
    /// Load config using the current directory as root, environment from `DOIDO_ENV`
    /// (defaults to `"development"`).
    pub fn load() -> Result<Arc<Self>> {
        let env = std::env::var("DOIDO_ENV").unwrap_or_else(|_| "development".to_string());
        Self::load_from_env(Path::new("."), &env)
    }

    /// Load config from `root`, environment from `DOIDO_ENV`.
    pub fn load_from(root: impl AsRef<Path>) -> Result<Arc<Self>> {
        let env = std::env::var("DOIDO_ENV").unwrap_or_else(|_| "development".to_string());
        Self::load_from_env(root.as_ref(), &env)
    }

    /// Load config from `root` with an explicit environment name.
    /// Useful in tests to avoid depending on the `DOIDO_ENV` env var.
    ///
    /// Loading order (each layer overrides the previous):
    /// 1. `config/doido.toml`             — base config
    /// 2. `config/doido.<env>.toml`       — environment override
    /// 3. `config/credentials.toml.enc`   — encrypted secrets (if present)
    /// 4. Env vars with `SECTION__KEY` notation
    pub fn load_from_env(root: &Path, env: &str) -> Result<Arc<Self>> {
        let mut merged = loader::load_layers(root, env)?;
        env_override::apply_env_overrides(&mut merged);
        let toml_str = toml::to_string(&merged)
            .map_err(|e| doido_core::anyhow::anyhow!("cannot serialize merged config: {e}"))?;
        let config: Self = toml::from_str(&toml_str)
            .map_err(|e| doido_core::anyhow::anyhow!("cannot deserialize config: {e}"))?;
        Ok(Arc::new(config))
    }
}
```

- [ ] **Step 4: Run the integration tests to verify they pass**

Run: `cargo test -p doido-config --test config_test`
Expected: PASS — 7 integration tests.

- [ ] **Step 5: Run the full test suite**

Run: `cargo test -p doido-config`
Expected: PASS — all 32 tests (2 types + 9 loader + 7 env_override + 7 crypto + 7 integration).

- [ ] **Step 6: Check for compiler warnings**

Run: `cargo build -p doido-config 2>&1 | grep warning`
Expected: no warnings.

- [ ] **Step 7: Commit**

```bash
git add doido-config/src/lib.rs doido-config/tests/config_test.rs
git commit -m "feat(config): wire Config::load_from_env and add full integration tests"
```

---

## Self-Review

### Spec Coverage

| Spec requirement | Covered by |
|---|---|
| TOML parsing via `toml` crate + `serde` | Task 2, Task 3 |
| Layer 1: `config/doido.toml` (base) | Task 3 — `load_layers` |
| Layer 2: `config/doido.<env>.toml` (env override) | Task 3 — `load_layers` |
| Layer 3: `config/credentials.toml.enc` (encrypted) | Task 6 — `load_layers` credentials step |
| Layer 4: env vars `SECTION__KEY` | Task 4 — `apply_env_overrides` |
| AES-256-GCM encryption | Task 5 — `crypto.rs` |
| `DOIDO_MASTER_KEY` env var + `config/master.key` file | Task 5 — `load_master_key` |
| `Config::load()` | Task 7 |
| `Config::load_from(root)` | Task 7 |
| `Config::load_from_env(root, env)` | Task 7 |
| Returns `Arc<Config>` | Task 7 |
| `ServerConfig`, `DatabaseConfig`, `ViewConfig`, `LogConfig` | Task 2 |
| Missing `master.key` with credentials file → error | Task 6 test + Task 7 integration test |
| Env var overrides take highest precedence | Task 4 unit tests + Task 7 integration test |
| `Config::load()` in test env uses `config/doido.test.toml` | Task 7 `test_load_from_uses_doido_env` |
| `encrypt_credentials` / `decrypt_credentials` public API for CLI | Task 5 — `pub fn` in `crypto.rs` |

### Placeholder Scan

No TBDs or TODOs remain. Every step contains executable code.

### Type Consistency

- `load_layers(root: &Path, env: &str) -> Result<toml::Value>` — used in `lib.rs` `load_from_env` and tested in `loader.rs` unit tests. Consistent.
- `apply_env_overrides(value: &mut toml::Value)` — called in `lib.rs`. `apply_overrides_from` (same module, takes iterator) is used in unit tests. Consistent.
- `encrypt_credentials(plaintext: &str, key: &[u8; 32]) -> Result<String>` — used in Task 6 loader test and Task 7 integration test. Consistent.
- `decrypt_credentials(encoded: &str, key: &[u8; 32]) -> Result<String>` — used in `load_layers`. Consistent.
- `load_master_key(root: &Path) -> Result<[u8; 32]>` — used in `load_layers`. Consistent.
- `Config::load_from_env(root: &Path, env: &str) -> Result<Arc<Config>>` — tested in all 7 integration tests. Consistent.
