# doido-controller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `doido-controller` — a proc-macro-driven Action Controller for the Doido framework that wraps axum handlers, provides a typed `Context` request object, and supports `#[before_action]` / `#[after_action]` filter chains.

**Architecture:** Two crates: `doido-controller-macros` is a `proc-macro` crate with the `#[controller]`, `#[before_action]`, and `#[after_action]` attribute macros. `doido-controller` is the library crate exporting `Context`, `Response`, the `Controller` trait, and re-exporting the macros. The `#[controller]` macro rewrites each action method into an axum-compatible handler that builds a `Context`, runs the before-filter chain, calls the action, then runs the after-filter chain. `Context` owns the request parts and delegates rendering to `doido-view`.

**Tech Stack:** Rust, `axum 0.7`, `tokio 1`, `serde` + `serde_json 1`; proc-macro crate: `syn 2`, `quote 1`, `proc-macro2 1`; dev: `tower`, `http`, `http-body-util`, `tokio`

---

## File Structure

| File | Purpose |
|------|---------|
| `doido-controller-macros/Cargo.toml` | proc-macro crate manifest |
| `doido-controller-macros/src/lib.rs` | `#[controller]`, `#[before_action]`, `#[after_action]` entry points |
| `doido-controller-macros/src/controller.rs` | `#[controller]` macro implementation — rewrites the impl block |
| `doido-controller/Cargo.toml` | library crate manifest |
| `doido-controller/src/lib.rs` | Re-exports `Context`, `Response`, `Controller` trait, macros |
| `doido-controller/src/context.rs` | `Context` struct + response helpers (`render`, `json`, `redirect_to`, `status`) |
| `doido-controller/src/response.rs` | `Response` type alias + `IntoResponse` bridge |
| `doido-controller/tests/controller_test.rs` | Integration tests: direct action calls, filter chain, full HTTP |

---

### Task 1: Crate Scaffolds

**Files:**
- Create: `doido-controller-macros/Cargo.toml`
- Create: `doido-controller-macros/src/lib.rs` (stub)
- Create: `doido-controller/Cargo.toml`
- Create: `doido-controller/src/lib.rs` (stub)
- Create: `doido-controller/src/context.rs` (stub)
- Create: `doido-controller/src/response.rs` (stub)
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create `doido-controller-macros/Cargo.toml`**

```toml
[package]
name = "doido-controller-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

- [ ] **Step 2: Create `doido-controller-macros/src/lib.rs` stub**

```rust
use proc_macro::TokenStream;

/// Marks an impl block as a controller. Rewrites action methods into
/// axum-compatible handlers with filter chain support.
#[proc_macro_attribute]
pub fn controller(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Registers a before-action filter on the following action method.
/// Usage: `#[before_action(fn_name)]` or `#[before_action(fn_name, only = [action1, action2])]`
#[proc_macro_attribute]
pub fn before_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Registers an after-action filter on the following action method.
/// Usage: `#[after_action(fn_name)]`
#[proc_macro_attribute]
pub fn after_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
```

- [ ] **Step 3: Create `doido-controller/Cargo.toml`**

```toml
[package]
name = "doido-controller"
version = "0.1.0"
edition = "2021"

[dependencies]
doido-core = { path = "../doido-core" }
doido-controller-macros = { path = "../doido-controller-macros" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
http = "1"

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 4: Create `doido-controller/src/response.rs` stub**

```rust
use axum::response::IntoResponse;

/// Controller actions return this type.
/// It wraps an axum response so we can add helpers later.
pub type Response = axum::response::Response;

/// Convenience re-export so action impls can use it
pub use axum::response::IntoResponse as IntoControllerResponse;
```

- [ ] **Step 5: Create `doido-controller/src/context.rs` stub**

```rust
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    response::Response,
};
use serde::Serialize;

/// Per-request context passed to every action.
pub struct Context {
    pub(crate) parts: http::request::Parts,
}

impl Context {
    pub fn from_request_parts(parts: http::request::Parts) -> Self {
        Self { parts }
    }

    /// Deserialize typed params from the request URI query string.
    /// For body params see `params_from_body`.
    pub fn params<T: serde::de::DeserializeOwned>(&self) -> doido_core::Result<T> {
        let query = self.parts.uri.query().unwrap_or("");
        serde_urlencoded::from_str(query)
            .map_err(|e| doido_core::anyhow::anyhow!("params deserialization failed: {e}"))
    }

    /// Return a plain-text 200 response (placeholder until doido-view is wired).
    pub fn render(&self, template: &str, _data: serde_json::Value) -> Response {
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!("render:{template}")))
            .unwrap()
    }

    /// Return a JSON 200 response.
    pub fn json<T: Serialize>(&self, data: T) -> Response {
        let body = serde_json::to_vec(&data).unwrap_or_default();
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    /// Return a 302 redirect.
    pub fn redirect_to(&self, location: impl AsRef<str>) -> Response {
        axum::response::Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, HeaderValue::from_str(location.as_ref()).unwrap())
            .body(Body::empty())
            .unwrap()
    }

    /// Return a response with an explicit status code and empty body.
    pub fn status(&self, code: u16) -> Response {
        axum::response::Response::builder()
            .status(code)
            .body(Body::empty())
            .unwrap()
    }
}
```

- [ ] **Step 6: Create `doido-controller/src/lib.rs` stub**

```rust
pub mod context;
pub mod response;

pub use context::Context;
pub use response::Response;
pub use doido_controller_macros::{after_action, before_action, controller};
```

- [ ] **Step 7: Add both crates to the workspace**

Edit workspace `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "doido-core",
    "doido-config",
    "doido-view",
    "doido-router-macros",
    "doido-router",
    "doido-controller-macros",
    "doido-controller",
]
```

- [ ] **Step 8: Verify both crates compile**

Run: `cargo check -p doido-controller-macros && cargo check -p doido-controller`
Expected: both check cleanly.

- [ ] **Step 9: Add `serde_urlencoded` dependency**

`serde_urlencoded` is already a transitive dep of axum. Add it explicitly to `doido-controller/Cargo.toml` so it's stable:

```toml
serde_urlencoded = "0.7"
```

Run: `cargo check -p doido-controller`
Expected: clean.

- [ ] **Step 10: Commit**

```bash
git add doido-controller/ doido-controller-macros/ Cargo.toml
git commit -m "feat(controller): add doido-controller and doido-controller-macros crate scaffolds"
```

---

### Task 2: `Context` — params, json, redirect_to, status

**Files:**
- Modify: `doido-controller/src/context.rs`
- Create: `doido-controller/tests/controller_test.rs`

- [ ] **Step 1: Write the failing tests**

Create `doido-controller/tests/controller_test.rs`:

```rust
use doido_controller::Context;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde::Deserialize;

fn make_ctx(uri: &str) -> Context {
    let req = Request::builder().uri(uri).body(()).unwrap();
    let (parts, _) = req.into_parts();
    Context::from_request_parts(parts)
}

#[derive(Deserialize, Debug, PartialEq)]
struct SearchParams {
    q: String,
    page: Option<u32>,
}

#[tokio::test]
async fn test_ctx_params_deserializes_query_string() {
    let ctx = make_ctx("/search?q=hello&page=2");
    let p: SearchParams = ctx.params().unwrap();
    assert_eq!(p.q, "hello");
    assert_eq!(p.page, Some(2));
}

#[tokio::test]
async fn test_ctx_params_errors_on_invalid_input() {
    let ctx = make_ctx("/search?page=not_a_number");
    let result: doido_core::Result<SearchParams> = ctx.params();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ctx_json_returns_200_with_json_body() {
    let ctx = make_ctx("/");
    let resp = ctx.json(serde_json::json!({"ok": true}));
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").unwrap();
    assert!(ct.to_str().unwrap().contains("application/json"));
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["ok"], true);
}

#[tokio::test]
async fn test_ctx_redirect_to_returns_302_with_location() {
    let ctx = make_ctx("/");
    let resp = ctx.redirect_to("/dashboard");
    assert_eq!(resp.status(), StatusCode::FOUND);
    let loc = resp.headers().get("location").unwrap();
    assert_eq!(loc.to_str().unwrap(), "/dashboard");
}

#[tokio::test]
async fn test_ctx_status_returns_custom_status_code() {
    let ctx = make_ctx("/");
    let resp = ctx.status(422);
    assert_eq!(resp.status().as_u16(), 422);
}

#[tokio::test]
async fn test_ctx_render_returns_ok_with_template_name() {
    let ctx = make_ctx("/");
    let resp = ctx.render("posts/index", serde_json::json!({}));
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert!(std::str::from_utf8(&body).unwrap().contains("posts/index"));
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-controller --test controller_test`
Expected: compile errors — `http_body_util` not in dev-deps yet; `doido_core::Result` may not be imported.

- [ ] **Step 3: Add missing dev dependency**

`http-body-util` is already in `doido-controller/Cargo.toml` dev-deps from Task 1 Step 3. Verify `doido-core` is also a dependency (it is, from Step 3). Run: `cargo test -p doido-controller --test controller_test`
Expected: 6 tests PASS (Context methods are already implemented in the stub from Task 1).

If `test_ctx_params_errors_on_invalid_input` fails because `serde_urlencoded` silently ignores bad input, update `context.rs` to use strict parsing:

```rust
pub fn params<T: serde::de::DeserializeOwned>(&self) -> doido_core::Result<T> {
    let query = self.parts.uri.query().unwrap_or("");
    serde_urlencoded::from_str(query)
        .map_err(|e| doido_core::anyhow::anyhow!("params deserialization failed: {e}"))
}
```

The error will propagate from serde's strict field type validation.

- [ ] **Step 4: Run to verify all pass**

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 6 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-controller/src/context.rs doido-controller/tests/controller_test.rs
git commit -m "feat(controller): implement Context with params, json, redirect_to, status, render"
```

---

### Task 3: `#[controller]` Macro — Plain Actions (No Filters)

**Files:**
- Modify: `doido-controller-macros/src/lib.rs`
- Create: `doido-controller-macros/src/controller.rs`
- Modify: `doido-controller/tests/controller_test.rs`

The `#[controller]` attribute macro rewrites each `async fn` in the `impl` block into an axum handler: the handler receives `axum::extract::Request`, splits into parts, builds a `Context`, calls the original action body, and returns the result.

- [ ] **Step 1: Write the failing tests**

Append to `doido-controller/tests/controller_test.rs`:

```rust
use axum::body::Body;
use tower::ServiceExt;

// A minimal controller struct with #[controller] applied
#[doido_controller::controller]
struct HelloController;

impl HelloController {
    async fn index(ctx: Context) -> doido_controller::Response {
        ctx.json(serde_json::json!({"message": "hello"}))
    }

    async fn show(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }
}

#[tokio::test]
async fn test_controller_index_action_via_axum() {
    // After #[controller], HelloController::index becomes an axum handler fn
    let app = axum::Router::new()
        .route("/hello", axum::routing::get(HelloController::index));

    let resp = app
        .oneshot(Request::builder().uri("/hello").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["message"], "hello");
}

#[tokio::test]
async fn test_controller_show_action_via_axum() {
    let app = axum::Router::new()
        .route("/hello/:id", axum::routing::get(HelloController::show));

    let resp = app
        .oneshot(Request::builder().uri("/hello/1").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-controller --test controller_test controller_index`
Expected: compile error — `#[doido_controller::controller]` is a passthrough stub; `HelloController::index` has signature `(ctx: Context) -> Response` which is not a valid axum handler (axum expects `FromRequestParts`/`FromRequest` extractors).

- [ ] **Step 3: Create `doido-controller-macros/src/controller.rs`**

This rewrites each action `async fn name(ctx: Context) -> Response` into a free function `async fn name(req: axum::extract::Request) -> axum::response::Response` that builds a `Context` from request parts and calls the original body. The original impl block is kept for helper methods and non-action fns; only methods with the signature `async fn foo(ctx: Context) -> Response` are rewritten.

```rust
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    FnArg, ImplItem, ItemImpl, Pat, PatIdent, PatType, Result, ReturnType, Type, parse2,
};

pub fn expand_controller(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut impl_block: ItemImpl = parse2(item)?;
    let self_ty = &impl_block.self_ty;

    // Collect rewritten handler items
    let mut handler_fns: Vec<TokenStream> = Vec::new();

    for impl_item in &mut impl_block.items {
        if let ImplItem::Fn(method) = impl_item {
            if !method.sig.asyncness.is_some() {
                continue;
            }
            // Check: first param named `ctx` of type `Context`
            let is_action = method.sig.inputs.iter().any(|arg| {
                if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                    if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                        if ident == "ctx" {
                            return true;
                        }
                    }
                }
                false
            });
            if !is_action {
                continue;
            }

            let fn_name = &method.sig.ident;
            let body = &method.block;

            // Generate a free async fn that builds Context and calls the body
            handler_fns.push(quote! {
                pub async fn #fn_name(
                    req: ::axum::extract::Request,
                ) -> ::axum::response::Response {
                    let (parts, _body) = req.into_parts();
                    let ctx = ::doido_controller::Context::from_request_parts(parts);
                    #body
                }
            });
        }
    }

    // Remove action methods from the impl block (they become free fns instead)
    impl_block.items.retain(|item| {
        if let ImplItem::Fn(method) = item {
            if method.sig.asyncness.is_some() {
                let is_action = method.sig.inputs.iter().any(|arg| {
                    if let FnArg::Typed(PatType { pat, .. }) = arg {
                        if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                            return ident == "ctx";
                        }
                    }
                    false
                });
                return !is_action;
            }
        }
        true
    });

    let output = quote! {
        #impl_block

        impl #self_ty {
            #(#handler_fns)*
        }
    };

    Ok(output)
}
```

- [ ] **Step 4: Update `doido-controller-macros/src/lib.rs` to call `expand_controller`**

```rust
mod controller;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    match controller::expand_controller(attr.into(), item.into()) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn before_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Stripped by #[controller] when it processes the impl block.
    // If applied outside a #[controller] block, it's a no-op passthrough.
    item
}

#[proc_macro_attribute]
pub fn after_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
```

- [ ] **Step 5: Run to verify tests pass**

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 8 tests (6 original + 2 new).

- [ ] **Step 6: Commit**

```bash
git add doido-controller-macros/src/ doido-controller/tests/controller_test.rs
git commit -m "feat(controller): #[controller] macro rewrites action methods as axum handlers"
```

---

### Task 4: `#[before_action]` Filter Chain

**Files:**
- Modify: `doido-controller-macros/src/controller.rs`
- Modify: `doido-controller/tests/controller_test.rs`

Filter signature: `async fn name(ctx: &mut Context) -> Result<(), Response>`. Returning `Err(resp)` halts the chain and returns the error response. The macro collects all `#[before_action]` attrs on an action method and generates a sequential call chain before the action body.

- [ ] **Step 1: Write the failing tests**

Append to `doido-controller/tests/controller_test.rs`:

```rust
use doido_controller::Context;

// Filter functions — note signature: async fn name(ctx: &mut Context) -> Result<(), doido_controller::Response>
async fn require_auth(ctx: &mut Context) -> Result<(), doido_controller::Response> {
    if ctx.header("x-auth-token").is_none() {
        return Err(ctx.status(401));
    }
    Ok(())
}

async fn set_locale(_ctx: &mut Context) -> Result<(), doido_controller::Response> {
    Ok(()) // always passes
}

#[doido_controller::controller]
struct SecureController;

impl SecureController {
    #[before_action(require_auth)]
    async fn secret(ctx: Context) -> doido_controller::Response {
        ctx.json(serde_json::json!({"secret": "data"}))
    }

    #[before_action(require_auth)]
    #[before_action(set_locale)]
    async fn double_filtered(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }
}

#[tokio::test]
async fn test_before_action_halts_when_filter_returns_err() {
    let app = axum::Router::new()
        .route("/secret", axum::routing::get(SecureController::secret));

    // No auth token — filter should return 401
    let resp = app.clone()
        .oneshot(Request::builder().uri("/secret").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // With auth token — filter passes, action runs
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/secret")
                .header("x-auth-token", "valid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_before_actions_run_in_order() {
    let app = axum::Router::new()
        .route("/double", axum::routing::get(SecureController::double_filtered));

    // Without auth — first filter halts
    let resp = app.clone()
        .oneshot(Request::builder().uri("/double").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // With auth — both filters pass, action runs
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/double")
                .header("x-auth-token", "valid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-controller --test controller_test before_action`
Expected: compile error — `ctx.header(...)` not defined; `#[before_action]` attr not processed by `#[controller]`.

- [ ] **Step 3: Add `header` helper to `Context`**

Append to `doido-controller/src/context.rs`:

```rust
impl Context {
    // ... existing methods ...

    /// Get a request header by name (lowercase).
    pub fn header(&self, name: &str) -> Option<&http::HeaderValue> {
        self.parts.headers.get(name)
    }
}
```

- [ ] **Step 4: Update `doido-controller-macros/src/controller.rs` to process `#[before_action]`**

Replace the `expand_controller` function with this version that collects before-action filters and wraps the action body in a filter chain:

```rust
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Attribute, FnArg, ImplItem, ItemImpl, Meta, Pat, PatIdent, PatType, Result,
    parse2, parse_str,
};

/// Extract the filter function name from a `#[before_action(fn_name)]` or
/// `#[before_action(fn_name, only = [...])]` attribute.
fn parse_filter_attr(attr: &Attribute) -> Option<(proc_macro2::Ident, Option<Vec<String>>)> {
    // Get the list of tokens inside the attribute parens
    let meta = &attr.meta;
    let path = meta.path();
    let attr_name = path.get_ident()?.to_string();
    if attr_name != "before_action" && attr_name != "after_action" {
        return None;
    }
    match meta {
        Meta::List(list) => {
            // Parse as: fn_name  OR  fn_name, only = [a, b]
            let tokens = list.tokens.clone();
            let mut iter = tokens.into_iter();
            // First token: the filter fn name
            let fn_ident: proc_macro2::Ident = match iter.next() {
                Some(proc_macro2::TokenTree::Ident(i)) => i,
                _ => return None,
            };
            // Check for `only = [...]`
            let mut only: Option<Vec<String>> = None;
            // consume optional comma and `only = [...]`
            let remaining: TokenStream = iter.collect();
            let remaining_str = remaining.to_string();
            if remaining_str.contains("only") {
                // naive parse: extract ident tokens from brackets
                let start = remaining_str.find('[').unwrap_or(remaining_str.len());
                let end = remaining_str.find(']').unwrap_or(remaining_str.len());
                if start < end {
                    let inner = &remaining_str[start + 1..end];
                    let actions: Vec<String> = inner
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    only = Some(actions);
                }
            }
            Some((fn_ident, only))
        }
        _ => None,
    }
}

pub fn expand_controller(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut impl_block: ItemImpl = parse2(item)?;
    let self_ty = &impl_block.self_ty;

    let mut handler_fns: Vec<TokenStream> = Vec::new();

    // Clone items for iteration (we'll retain non-action items below)
    let items_snapshot = impl_block.items.clone();

    for impl_item in &items_snapshot {
        let ImplItem::Fn(method) = impl_item else { continue };
        if method.sig.asyncness.is_none() { continue; }

        // Check if first param is `ctx: Context`
        let is_action = method.sig.inputs.iter().any(|arg| {
            if let FnArg::Typed(PatType { pat, .. }) = arg {
                if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                    return ident == "ctx";
                }
            }
            false
        });
        if !is_action { continue; }

        let fn_name = &method.sig.ident;
        let fn_name_str = fn_name.to_string();
        let body = &method.block;

        // Collect before_action and after_action attrs
        let mut before_filters: Vec<(proc_macro2::Ident, Option<Vec<String>>)> = Vec::new();
        let mut after_filters: Vec<proc_macro2::Ident> = Vec::new();

        for attr in &method.attrs {
            let path = attr.meta.path();
            let name = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
            if name == "before_action" {
                if let Some(info) = parse_filter_attr(attr) {
                    before_filters.push(info);
                }
            } else if name == "after_action" {
                if let Meta::List(list) = &attr.meta {
                    let filter_ident: proc_macro2::Ident =
                        syn::parse2(list.tokens.clone()).unwrap();
                    after_filters.push(filter_ident);
                }
            }
        }

        // Build the before-filter chain
        // Each filter: `if let Err(r) = filter_fn(&mut ctx).await { return r; }`
        let before_chain: Vec<TokenStream> = before_filters
            .iter()
            .map(|(filter_fn, only)| {
                if let Some(only_actions) = only {
                    // Only apply if current action is in the `only` list
                    if !only_actions.iter().any(|a| a == &fn_name_str) {
                        return quote! {}; // skip this filter for this action
                    }
                }
                quote! {
                    if let Err(__early_response) = #filter_fn(&mut ctx).await {
                        return __early_response;
                    }
                }
            })
            .collect();

        // Build the after-filter chain (run after action body)
        let after_chain: Vec<TokenStream> = after_filters
            .iter()
            .map(|filter_fn| {
                quote! {
                    #filter_fn(&mut ctx).await;
                }
            })
            .collect();

        handler_fns.push(quote! {
            pub async fn #fn_name(
                req: ::axum::extract::Request,
            ) -> ::axum::response::Response {
                let (parts, _body) = req.into_parts();
                let mut ctx = ::doido_controller::Context::from_request_parts(parts);
                #(#before_chain)*
                let __response = (move || async move { #body })().await;
                #(#after_chain)*
                __response
            }
        });
    }

    // Remove action methods from original impl block
    impl_block.items.retain(|item| {
        if let ImplItem::Fn(method) = item {
            if method.sig.asyncness.is_some() {
                let is_action = method.sig.inputs.iter().any(|arg| {
                    if let FnArg::Typed(PatType { pat, .. }) = arg {
                        if let Pat::Ident(PatIdent { ident, .. }) = pat.as_ref() {
                            return ident == "ctx";
                        }
                    }
                    false
                });
                return !is_action;
            }
        }
        true
    });

    Ok(quote! {
        #impl_block
        impl #self_ty {
            #(#handler_fns)*
        }
    })
}
```

- [ ] **Step 5: Run to verify tests pass**

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 10 tests (8 original + 2 new).

- [ ] **Step 6: Commit**

```bash
git add doido-controller-macros/src/ doido-controller/src/context.rs doido-controller/tests/controller_test.rs
git commit -m "feat(controller): #[before_action] filter chain — halts on Err, runs in order"
```

---

### Task 5: `#[before_action(fn, only = [...])]` — Scoped Filters

**Files:**
- Modify: `doido-controller/tests/controller_test.rs`

The `only = [...]` parsing is already implemented in `expand_controller` from Task 4. This task adds tests to verify that a filter with `only` only fires on the listed actions.

- [ ] **Step 1: Write the failing tests**

Append to `doido-controller/tests/controller_test.rs`:

```rust
async fn load_record(ctx: &mut Context) -> Result<(), doido_controller::Response> {
    // Simulate a "not found" filter that halts for id=0
    if ctx.header("x-id").map(|h| h.to_str().unwrap_or("")) == Some("0") {
        return Err(ctx.status(404));
    }
    Ok(())
}

#[doido_controller::controller]
struct ScopedController;

impl ScopedController {
    // load_record only fires for show and edit
    #[before_action(load_record, only = [show, edit])]
    async fn index(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }

    #[before_action(load_record, only = [show, edit])]
    async fn show(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }

    #[before_action(load_record, only = [show, edit])]
    async fn edit(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }
}

#[tokio::test]
async fn test_before_action_only_fires_for_specified_actions() {
    let app = axum::Router::new()
        .route("/items", axum::routing::get(ScopedController::index))
        .route("/items/:id", axum::routing::get(ScopedController::show))
        .route("/items/:id/edit", axum::routing::get(ScopedController::edit));

    // index — filter should NOT fire (not in `only` list) → 200 even with id=0
    let resp = app.clone()
        .oneshot(
            Request::builder().uri("/items").header("x-id", "0").body(Body::empty()).unwrap()
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // show — filter fires, id=0 → 404
    let resp = app.clone()
        .oneshot(
            Request::builder().uri("/items/1").header("x-id", "0").body(Body::empty()).unwrap()
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // show — filter fires, id≠0 → 200
    let resp = app
        .oneshot(
            Request::builder().uri("/items/1").header("x-id", "1").body(Body::empty()).unwrap()
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-controller --test controller_test only_fires`
Expected: failure — `index` currently still runs `load_record` (the `only` logic may not compile/filter correctly yet).

- [ ] **Step 3: Fix the `only` filter in codegen if needed**

In Task 4, the `only` check in `before_chain` is done at macro expansion time (compile time). Verify the logic: the macro checks `only_actions.iter().any(|a| a == &fn_name_str)`. If the action's name is NOT in `only`, the filter call is replaced with `quote! {}` (empty). This is correct — run the test to confirm.

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 11 tests.

- [ ] **Step 4: Commit**

```bash
git add doido-controller/tests/controller_test.rs
git commit -m "test(controller): verify before_action only: scopes filter to listed actions"
```

---

### Task 6: `#[after_action]` Filter

**Files:**
- Modify: `doido-controller/tests/controller_test.rs`

After-action filters receive `&mut Context` but their return type is `()` — they cannot halt the response. The generated code calls them after the action body has produced a response.

- [ ] **Step 1: Write the failing tests**

Append to `doido-controller/tests/controller_test.rs`:

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// Use a thread-local flag to verify after_action fired
thread_local! {
    static AFTER_FIRED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

async fn log_response(_ctx: &mut Context) {
    AFTER_FIRED.with(|f| f.set(true));
}

#[doido_controller::controller]
struct LoggedController;

impl LoggedController {
    #[after_action(log_response)]
    async fn index(ctx: Context) -> doido_controller::Response {
        ctx.status(200)
    }
}

#[tokio::test]
async fn test_after_action_fires_after_action_body() {
    AFTER_FIRED.with(|f| f.set(false));

    let app = axum::Router::new()
        .route("/logged", axum::routing::get(LoggedController::index));

    let resp = app
        .oneshot(Request::builder().uri("/logged").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(AFTER_FIRED.with(|f| f.get()), "after_action was not called");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p doido-controller --test controller_test after_action`
Expected: compile error or test failure — `log_response` has return type `()`, but the after-filter code in Task 4 calls `.await` without return type check.

- [ ] **Step 3: Update after-filter codegen for `()` return type**

In `doido-controller-macros/src/controller.rs`, the after-chain currently does:

```rust
#filter_fn(&mut ctx).await;
```

This is already correct for `()` return type. If the compile error is about signature mismatch, update `expand_controller` to not `if let Err` wrap after-filters:

```rust
let after_chain: Vec<TokenStream> = after_filters
    .iter()
    .map(|filter_fn| {
        quote! {
            #filter_fn(&mut ctx).await;
        }
    })
    .collect();
```

Verify the generated handler ends with:

```rust
let __response = (move || async move { #body })().await;
#(#after_chain)*
__response
```

- [ ] **Step 4: Run to verify all tests pass**

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 12 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-controller-macros/src/ doido-controller/tests/controller_test.rs
git commit -m "feat(controller): #[after_action] filter runs after action body completes"
```

---

### Task 7: Full Integration Test — Controller + Router

**Files:**
- Modify: `doido-controller/tests/controller_test.rs`

Wire a controller through the `routes!` macro to verify the full stack: router → before-filter → action → after-filter.

Note: `doido-router` is a separate crate. This integration test adds `doido-router` as a dev-dependency to `doido-controller` and uses `routes!` directly.

- [ ] **Step 1: Add `doido-router` to dev-dependencies**

Edit `doido-controller/Cargo.toml`:

```toml
[dev-dependencies]
doido-router = { path = "../doido-router" }
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 2: Write the failing integration test**

Append to `doido-controller/tests/controller_test.rs`:

```rust
async fn auth_guard(ctx: &mut Context) -> Result<(), doido_controller::Response> {
    if ctx.header("authorization").is_none() {
        return Err(ctx.status(401));
    }
    Ok(())
}

#[doido_controller::controller]
struct ArticlesController;

impl ArticlesController {
    #[before_action(auth_guard)]
    async fn index(ctx: Context) -> doido_controller::Response {
        ctx.json(serde_json::json!({"articles": []}))
    }

    async fn show(ctx: Context) -> doido_controller::Response {
        ctx.json(serde_json::json!({"id": 1}))
    }
}

mod articles_mod {
    pub use super::ArticlesController::index;
    pub use super::ArticlesController::show;
}

#[tokio::test]
async fn test_full_stack_controller_with_router_and_filters() {
    let app = doido_router::routes! {
        get!("/articles", articles_mod::index)
        get!("/articles/:id", articles_mod::show)
    };

    // No auth — before_action halts
    let resp = app.clone()
        .oneshot(Request::builder().uri("/articles").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // With auth — action runs
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .uri("/articles")
                .header("authorization", "Bearer token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // show has no filter — always 200
    let resp = app
        .oneshot(Request::builder().uri("/articles/1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p doido-controller --test controller_test full_stack`
Expected: compile error — `doido-router` not yet in dev-deps, or `articles_mod` not accessible from `routes!` block.

- [ ] **Step 4: Run after adding dev-dependency**

Run: `cargo test -p doido-controller --test controller_test`
Expected: PASS — 13 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-controller/Cargo.toml doido-controller/tests/controller_test.rs
git commit -m "test(controller): full-stack integration test with doido-router and before_action"
```

---

### Task 8: Final Check

**Files:**
- Modify: `Cargo.toml` (workspace, verify both crates listed)
- Modify: `doido-controller/src/lib.rs` (clean up re-exports)

- [ ] **Step 1: Run all controller tests**

Run: `cargo test -p doido-controller`
Expected: PASS — 13 tests, 0 failures.

- [ ] **Step 2: Run workspace-level check for warnings**

Run: `cargo build -p doido-controller -p doido-controller-macros 2>&1 | grep -E "^warning"`
Expected: no warnings (fix any that appear).

- [ ] **Step 3: Run clippy**

Run: `cargo clippy -p doido-controller -p doido-controller-macros -- -D warnings`
Expected: clean. Fix any lint errors.

- [ ] **Step 4: Commit any fixes**

```bash
git add -u
git commit -m "chore(controller): fix clippy warnings and clean up re-exports"
```

(Skip this step if there are no changes.)

---

## Self-Review

### Spec Coverage

| Spec requirement | Covered by |
|---|---|
| `#[controller]` attribute macro on struct | Task 3 |
| Actions are `async fn(ctx: Context) -> Response` | Tasks 2, 3 |
| `ctx.params::<T>()` typed param deserialization | Task 2 |
| `ctx.render(template, data)` delegates to view | Task 2 (stub body) |
| `ctx.redirect_to(path)` returns 302 | Task 2 |
| `ctx.json(data)` returns JSON 200 | Task 2 |
| `ctx.status(code)` returns given status | Task 2 |
| `#[before_action(fn_name)]` runs before action | Task 4 |
| Filter `Err(response)` halts chain | Task 4 |
| `#[before_action(fn, only = [...])]` scoped | Tasks 4, 5 |
| `#[after_action(fn_name)]` runs after action | Task 6 |
| Multiple before-actions run in declaration order | Task 4 |
| Tower middleware at router level (separate concern) | Delegated to `doido-router` `.layer()` — not in this crate |
| Test helper: construct `Context` directly without HTTP | Task 2 (`make_ctx` helper) |
| Integration test: full HTTP round-trip with filters | Task 7 |

### Placeholder Scan

No TODOs, TBDs, or placeholder phrases. Every step contains runnable code.

### Type Consistency

- `Context::from_request_parts(parts: http::request::Parts) -> Context` — defined in Task 1, used in Tasks 2, 3, 4, 5, 6, 7. Consistent.
- `doido_controller::Response` = `axum::response::Response` — aliased in Task 1 `response.rs`, used throughout. Consistent.
- Filter before-action signature: `async fn f(ctx: &mut Context) -> Result<(), doido_controller::Response>` — defined in Task 4 tests, generated call matches. Consistent.
- Filter after-action signature: `async fn f(ctx: &mut Context)` (returns `()`) — defined in Task 6 tests, generated call matches. Consistent.
- `#[before_action(fn_name, only = [action1, action2])]` — parsed in Task 4 `parse_filter_attr`, `only` list matched against `fn_name_str` at macro expansion time. Consistent.
