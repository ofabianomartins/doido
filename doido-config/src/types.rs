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
