# doido-view Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `doido-view` — the framework's template rendering system with a swappable `TemplateEngine` trait, a `TeraEngine` default implementation, layout support, and a `ViewResponse` builder.

**Architecture:** Four cooperating modules — `engine` defines the `TemplateEngine` trait; `tera_engine` wraps `tera::Tera` behind an `RwLock` for thread-safe rendering and hot-reload; `response` exposes a builder-pattern `ViewResponse` that carries template name, JSON context, HTTP status, and an optional layout override; `renderer` orchestrates the two-pass render (content then layout injection) using an `Arc<dyn TemplateEngine>`. The `Renderer` is constructed once at app boot and shared across request handlers.

**Tech Stack:** Rust, `doido-core 0.1`, `tera 1`, `serde_json 1`; dev: `tempfile 3`

---

## File Structure

| File | Purpose |
|------|---------|
| `doido-view/Cargo.toml` | Crate manifest with all dependencies |
| `doido-view/src/lib.rs` | Module declarations + public re-exports |
| `doido-view/src/engine.rs` | `TemplateEngine` trait |
| `doido-view/src/tera_engine.rs` | `TeraEngine` wrapping `tera::Tera` with `RwLock` |
| `doido-view/src/response.rs` | `ViewResponse` builder |
| `doido-view/src/renderer.rs` | `Renderer` struct — two-pass render with layout |
| `doido-view/tests/view_test.rs` | Integration tests |

---

### Task 1: Crate Scaffold

**Files:**
- Create: `doido-view/Cargo.toml`
- Modify: `Cargo.toml` (workspace root)
- Create: `doido-view/src/lib.rs`
- Create: `doido-view/src/engine.rs` (stub)
- Create: `doido-view/src/tera_engine.rs` (stub)
- Create: `doido-view/src/response.rs` (stub)
- Create: `doido-view/src/renderer.rs` (stub)

- [ ] **Step 1: Create `doido-view/Cargo.toml`**

```toml
[package]
name = "doido-view"
version = "0.1.0"
edition = "2021"

[dependencies]
doido-core = { path = "../doido-core" }
tera = "1"
serde_json = "1"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Add `doido-view` to the workspace**

Edit `Cargo.toml` at the workspace root:

```toml
[workspace]
resolver = "2"
members = [
    "doido-core",
    "doido-config",
    "doido-view",
]
```

- [ ] **Step 3: Create `doido-view/src/lib.rs`**

```rust
pub mod engine;
pub mod tera_engine;
pub mod response;
pub mod renderer;

pub use engine::TemplateEngine;
pub use tera_engine::TeraEngine;
pub use response::ViewResponse;
pub use renderer::Renderer;
```

- [ ] **Step 4: Create stub source files**

Create `doido-view/src/engine.rs`:
```rust
// filled in Task 2
```

Create `doido-view/src/tera_engine.rs`:
```rust
// filled in Task 3
```

Create `doido-view/src/response.rs`:
```rust
// filled in Task 5
```

Create `doido-view/src/renderer.rs`:
```rust
// filled in Task 6
```

- [ ] **Step 5: Verify crate is visible to the workspace**

Run: `cargo check -p doido-view`
Expected: errors about empty modules — confirms crate is found by Cargo.

- [ ] **Step 6: Commit**

```bash
git add doido-view/ Cargo.toml
git commit -m "feat(view): add doido-view crate scaffold"
```

---

### Task 2: `TemplateEngine` Trait

**Files:**
- Create: `doido-view/src/engine.rs`

- [ ] **Step 1: Write the failing inline test first**

Replace `doido-view/src/engine.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
    use super::TemplateEngine;
    use serde_json::json;

    struct FakeEngine;
    impl TemplateEngine for FakeEngine {
        fn render(&self, template: &str, _ctx: &serde_json::Value) -> doido_core::Result<String> {
            Ok(format!("rendered:{template}"))
        }
        fn reload(&self) -> doido_core::Result<()> { Ok(()) }
    }

    #[test]
    fn test_engine_trait_is_object_safe() {
        let engine: &dyn TemplateEngine = &FakeEngine;
        let result = engine.render("posts/index", &json!({})).unwrap();
        assert_eq!(result, "rendered:posts/index");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p doido-view`
Expected: compile error — `TemplateEngine` not defined.

- [ ] **Step 3: Implement `doido-view/src/engine.rs`**

```rust
/// A swappable template rendering backend.
///
/// Implementations must be `Send + Sync` so they can be shared across threads
/// behind an `Arc`.
pub trait TemplateEngine: Send + Sync {
    /// Render `template` (e.g. `"posts/index"`) with the given JSON `context`.
    fn render(&self, template: &str, context: &serde_json::Value) -> doido_core::Result<String>;

    /// Re-read templates from disk. Called in development on file-change events.
    fn reload(&self) -> doido_core::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::TemplateEngine;
    use serde_json::json;

    struct FakeEngine;
    impl TemplateEngine for FakeEngine {
        fn render(&self, template: &str, _ctx: &serde_json::Value) -> doido_core::Result<String> {
            Ok(format!("rendered:{template}"))
        }
        fn reload(&self) -> doido_core::Result<()> { Ok(()) }
    }

    #[test]
    fn test_engine_trait_is_object_safe() {
        let engine: &dyn TemplateEngine = &FakeEngine;
        let result = engine.render("posts/index", &json!({})).unwrap();
        assert_eq!(result, "rendered:posts/index");
    }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p doido-view`
Expected: PASS — 1 test.

- [ ] **Step 5: Commit**

```bash
git add doido-view/src/engine.rs
git commit -m "feat(view): add TemplateEngine trait"
```

---

### Task 3: `TeraEngine` Basic Render

**Files:**
- Create: `doido-view/src/tera_engine.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-view/src/tera_engine.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
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
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-view`
Expected: compile error — `TeraEngine` not defined.

- [ ] **Step 3: Implement `doido-view/src/tera_engine.rs`**

```rust
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
```

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p doido-view`
Expected: PASS — 6 tests (1 engine + 5 tera_engine).

- [ ] **Step 5: Commit**

```bash
git add doido-view/src/tera_engine.rs
git commit -m "feat(view): add TeraEngine with render, hot-reload, and template resolution"
```

---

### Task 4: `ViewResponse` Builder

**Files:**
- Create: `doido-view/src/response.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-view/src/response.rs` with just the test module:

```rust
#[cfg(test)]
mod tests {
    use super::ViewResponse;
    use serde_json::json;

    #[test]
    fn test_view_response_defaults() {
        let r = ViewResponse::new("posts/index", json!({"x": 1}));
        assert_eq!(r.template, "posts/index");
        assert_eq!(r.status, 200);
        assert!(r.layout.is_none());
    }

    #[test]
    fn test_view_response_status_builder() {
        let r = ViewResponse::new("posts/new", json!({})).status(422);
        assert_eq!(r.status, 422);
    }

    #[test]
    fn test_view_response_layout_builder() {
        let r = ViewResponse::new("posts/index", json!({})).layout("admin");
        assert_eq!(r.layout, Some("admin".to_string()));
    }

    #[test]
    fn test_view_response_no_layout_builder() {
        let r = ViewResponse::new("posts/index", json!({})).no_layout();
        assert_eq!(r.layout, Some("".to_string()));
    }
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-view`
Expected: compile error — `ViewResponse` not defined.

- [ ] **Step 3: Implement `doido-view/src/response.rs`**

```rust
use serde_json::Value;

pub struct ViewResponse {
    pub template: String,
    pub context: Value,
    pub status: u16,
    pub layout: Option<String>,
}

impl ViewResponse {
    pub fn new(template: impl Into<String>, context: Value) -> Self {
        Self { template: template.into(), context, status: 200, layout: None }
    }

    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }

    /// Override the layout for this response.
    pub fn layout(mut self, name: impl Into<String>) -> Self {
        self.layout = Some(name.into());
        self
    }

    /// Render without any layout.
    pub fn no_layout(mut self) -> Self {
        self.layout = Some(String::new());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ViewResponse;
    use serde_json::json;

    #[test]
    fn test_view_response_defaults() {
        let r = ViewResponse::new("posts/index", json!({"x": 1}));
        assert_eq!(r.template, "posts/index");
        assert_eq!(r.status, 200);
        assert!(r.layout.is_none());
    }

    #[test]
    fn test_view_response_status_builder() {
        let r = ViewResponse::new("posts/new", json!({})).status(422);
        assert_eq!(r.status, 422);
    }

    #[test]
    fn test_view_response_layout_builder() {
        let r = ViewResponse::new("posts/index", json!({})).layout("admin");
        assert_eq!(r.layout, Some("admin".to_string()));
    }

    #[test]
    fn test_view_response_no_layout_builder() {
        let r = ViewResponse::new("posts/index", json!({})).no_layout();
        assert_eq!(r.layout, Some("".to_string()));
    }
}
```

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p doido-view`
Expected: PASS — 10 tests (1 engine + 5 tera_engine + 4 response).

- [ ] **Step 5: Commit**

```bash
git add doido-view/src/response.rs
git commit -m "feat(view): add ViewResponse builder with status and layout overrides"
```

---

### Task 5: `Renderer` — Two-Pass Layout Rendering

**Files:**
- Create: `doido-view/src/renderer.rs`

- [ ] **Step 1: Write the failing inline tests first**

Replace `doido-view/src/renderer.rs` with just the test module:

```rust
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
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-view`
Expected: compile error — `Renderer` not defined.

- [ ] **Step 3: Implement `doido-view/src/renderer.rs`**

```rust
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
```

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p doido-view`
Expected: PASS — 13 tests (1 + 5 + 4 + 3).

- [ ] **Step 5: Commit**

```bash
git add doido-view/src/renderer.rs
git commit -m "feat(view): add Renderer with default layout, override, and no_layout"
```

---

### Task 6: Integration Tests

**Files:**
- Create: `doido-view/tests/view_test.rs`

- [ ] **Step 1: Write the failing integration tests**

Create `doido-view/tests/view_test.rs`:

```rust
use doido_view::{engine::TemplateEngine, renderer::Renderer, response::ViewResponse, tera_engine::TeraEngine};
use std::sync::Arc;
use tempfile::TempDir;
use std::fs;

fn write_tpl(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

struct UppercaseEngine;
impl TemplateEngine for UppercaseEngine {
    fn render(&self, template: &str, _ctx: &serde_json::Value) -> doido_core::Result<String> {
        Ok(template.to_uppercase())
    }
    fn reload(&self) -> doido_core::Result<()> { Ok(()) }
}

#[test]
fn test_full_render_pipeline() {
    let dir = TempDir::new().unwrap();
    write_tpl(&dir, "posts/show.html.tera", "<article>{{ post_title }}</article>");
    write_tpl(&dir, "layouts/application.html.tera", "<!DOCTYPE html><body>{{ content_for_layout }}</body>");
    let engine = Arc::new(TeraEngine::new(dir.path().to_str().unwrap()).unwrap());
    let renderer = Renderer::new(engine, "application");
    let resp = ViewResponse::new("posts/show", serde_json::json!({ "post_title": "Hello" }));
    let html = renderer.render(&resp).unwrap();
    assert_eq!(html, "<!DOCTYPE html><body><article>Hello</article></body>");
}

#[test]
fn test_custom_engine_drop_in() {
    let engine: Arc<dyn TemplateEngine> = Arc::new(UppercaseEngine);
    let renderer = Renderer::new(engine, "");
    let resp = ViewResponse::new("posts/index", serde_json::json!({})).no_layout();
    let html = renderer.render(&resp).unwrap();
    assert_eq!(html, "POSTS/INDEX");
}

#[test]
fn test_status_preserved_in_response() {
    let resp = ViewResponse::new("posts/new", serde_json::json!({})).status(422);
    assert_eq!(resp.status, 422);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-view --test view_test`
Expected: compile error — `doido_core` not in scope (it's a transitive dep; add as direct dev-dep if needed).

If `doido_core` is not resolved in the integration test, add to `[dev-dependencies]`:
```toml
doido-core = { path = "../doido-core" }
```

- [ ] **Step 3: Run integration tests to verify they pass**

Run: `cargo test -p doido-view --test view_test`
Expected: PASS — 3 integration tests.

- [ ] **Step 4: Run the full test suite**

Run: `cargo test -p doido-view`
Expected: PASS — 16 tests (1 engine + 5 tera_engine + 4 response + 3 renderer + 3 integration).

- [ ] **Step 5: Check for warnings**

Run: `cargo build -p doido-view 2>&1 | grep warning`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add doido-view/tests/view_test.rs doido-view/Cargo.toml
git commit -m "test(view): add integration tests for full render pipeline and custom engine"
```

---

## Self-Review

### Spec Coverage

| Spec requirement | Covered by |
|---|---|
| `TemplateEngine` trait with `render` + `reload`, `Send + Sync` | Task 2 |
| `TeraEngine` wraps `tera::Tera` with `RwLock` | Task 3 |
| Template key `"posts/index"` → `"posts/index.html.tera"` | Task 3 |
| `ViewResponse::new`, `.status()`, `.layout()`, `.no_layout()` | Task 4 |
| `Renderer` with `Arc<dyn TemplateEngine>` + default layout | Task 5 |
| Layout injection via `{{ content_for_layout }}` | Task 5 |
| `.no_layout()` skips layout | Task 5 |
| Layout override via `.layout("admin")` | Task 5 |
| Hot reload via `reload()` | Task 3 |
| Custom engine drop-in via `TemplateEngine` trait | Task 6 |
| JSON responses bypass template engine | Out of scope — handled at controller level |

### Placeholder Scan

No TODOs, no TBDs. Every step contains runnable code.

### Type Consistency

- `TemplateEngine::render(&self, template: &str, context: &serde_json::Value) -> doido_core::Result<String>` — identical in trait (Task 2), `TeraEngine` (Task 3), `UppercaseEngine` (Task 6). Consistent.
- `Renderer::new(engine: Arc<dyn TemplateEngine>, default_layout: impl Into<String>)` — matches all call sites in Tasks 5 and 6. Consistent.
- `ViewResponse::layout: Option<String>` — `no_layout()` sets `Some("")`; `Renderer` checks `l.is_empty()`. Consistent.
