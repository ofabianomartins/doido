use std::sync::Arc;
use crate::{engine::TemplateEngine, response::ViewResponse};
use doido_core::Result;

pub struct Renderer {
    engine: Arc<dyn TemplateEngine>,
    default_layout: String,
}

impl Renderer {
    pub fn new(engine: Arc<dyn TemplateEngine>, default_layout: impl Into<String>) -> Self {
        Self { engine, default_layout: default_layout.into() }
    }

    pub fn render(&self, response: &ViewResponse) -> Result<String> {
        let content = self.engine.render(&response.template, &response.context)?;

        let layout = match &response.layout {
            Some(l) if l.is_empty() => return Ok(content),
            Some(l) => l.clone(),
            None => self.default_layout.clone(),
        };

        if layout.is_empty() {
            return Ok(content);
        }

        let mut layout_ctx = response.context.clone();
        if let Some(obj) = layout_ctx.as_object_mut() {
            obj.insert(
                "content_for_layout".to_string(),
                serde_json::Value::String(content),
            );
        }
        self.engine.render(&format!("layouts/{}", layout), &layout_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::Renderer;
    use crate::response::ViewResponse;
    use std::sync::Arc;
    use tempfile::TempDir;
    use std::fs;

    fn write_tpl(dir: &TempDir, rel: &str, content: &str) {
        let path = dir.path().join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_renderer_uses_default_layout() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "<main>content</main>");
        write_tpl(&dir, "layouts/application.html.tera", "<html>{{ content_for_layout }}</html>");
        let engine = Arc::new(crate::tera_engine::TeraEngine::new(dir.path().to_str().unwrap()).unwrap());
        let renderer = Renderer::new(engine, "application");
        let resp = ViewResponse::new("posts/index", serde_json::json!({}));
        let html = renderer.render(&resp).unwrap();
        assert_eq!(html, "<html><main>content</main></html>");
    }

    #[test]
    fn test_renderer_no_layout_skips_layout() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "<main>bare</main>");
        let engine = Arc::new(crate::tera_engine::TeraEngine::new(dir.path().to_str().unwrap()).unwrap());
        let renderer = Renderer::new(engine, "application");
        let resp = ViewResponse::new("posts/index", serde_json::json!({})).no_layout();
        let html = renderer.render(&resp).unwrap();
        assert_eq!(html, "<main>bare</main>");
    }

    #[test]
    fn test_renderer_custom_layout_override() {
        let dir = TempDir::new().unwrap();
        write_tpl(&dir, "posts/index.html.tera", "body");
        write_tpl(&dir, "layouts/admin.html.tera", "<admin>{{ content_for_layout }}</admin>");
        let engine = Arc::new(crate::tera_engine::TeraEngine::new(dir.path().to_str().unwrap()).unwrap());
        let renderer = Renderer::new(engine, "application");
        let resp = ViewResponse::new("posts/index", serde_json::json!({})).layout("admin");
        let html = renderer.render(&resp).unwrap();
        assert_eq!(html, "<admin>body</admin>");
    }
}
