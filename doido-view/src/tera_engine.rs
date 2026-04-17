use std::sync::RwLock;
use crate::engine::TemplateEngine;
use doido_core::{Result, anyhow::Context as _};

pub struct TeraEngine {
    tera: RwLock<tera::Tera>,
    templates_dir: String,
}

impl TeraEngine {
    pub fn new(templates_dir: &str) -> Result<Self> {
        let pattern = format!("{}/**/*.tera", templates_dir);
        let tera = tera::Tera::new(&pattern)
            .with_context(|| format!("failed to load templates from {templates_dir}"))?;
        Ok(Self {
            tera: RwLock::new(tera),
            templates_dir: templates_dir.to_string(),
        })
    }
}

impl TemplateEngine for TeraEngine {
    fn render(&self, template: &str, context: &serde_json::Value) -> Result<String> {
        let template_name = format!("{}.html.tera", template);
        let ctx = tera::Context::from_value(context.clone())
            .map_err(|e| doido_core::anyhow::anyhow!("invalid template context: {e}"))?;
        self.tera
            .read()
            .unwrap()
            .render(&template_name, &ctx)
            .map_err(|e| doido_core::anyhow::anyhow!("template '{}' render failed: {e}", template))
    }

    fn reload(&self) -> Result<()> {
        let pattern = format!("{}/**/*.tera", self.templates_dir);
        let tera = tera::Tera::new(&pattern)
            .with_context(|| format!("reload failed for {}", self.templates_dir))?;
        *self.tera.write().unwrap() = tera;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::TemplateEngine;
    use tempfile::TempDir;
    use std::fs;

    fn write_tpl(dir: &TempDir, rel: &str, content: &str) {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_tera_engine_renders_template_with_context() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "<h1>{{ title }}</h1>");
        let engine = super::TeraEngine::new(dir.path().to_str().unwrap()).unwrap();
        let ctx = serde_json::json!({ "title": "Hello World" });
        let html = engine.render("posts/index", &ctx).unwrap();
        assert_eq!(html, "<h1>Hello World</h1>");
    }

    #[test]
    fn test_unknown_template_returns_error() {
        let dir = TempDir::new().unwrap();
        let engine = super::TeraEngine::new(dir.path().to_str().unwrap()).unwrap();
        let result = engine.render("nonexistent/template", &serde_json::json!({}));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.to_lowercase().contains("template"), "got: {msg}");
    }

    #[test]
    fn test_template_key_resolves_to_html_tera_extension() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "resolved");
        let engine = super::TeraEngine::new(dir.path().to_str().unwrap()).unwrap();
        let result = engine.render("posts/index", &serde_json::json!({})).unwrap();
        assert_eq!(result, "resolved");
    }

    #[test]
    fn test_nested_controller_path_resolves_correctly() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "admin/users/index.html.tera", "admin-users");
        let engine = super::TeraEngine::new(dir.path().to_str().unwrap()).unwrap();
        let result = engine.render("admin/users/index", &serde_json::json!({})).unwrap();
        assert_eq!(result, "admin-users");
    }

    #[test]
    fn test_hot_reload_picks_up_template_changes() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "version1");
        let engine = super::TeraEngine::new(dir.path().to_str().unwrap()).unwrap();
        let first = engine.render("posts/index", &serde_json::json!({})).unwrap();
        assert_eq!(first, "version1");
        write_tpl(&dir, "posts/index.html.tera", "version2");
        engine.reload().unwrap();
        let second = engine.render("posts/index", &serde_json::json!({})).unwrap();
        assert_eq!(second, "version2");
    }
}
