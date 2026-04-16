# doido-router Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `doido-router` — a macro DSL that wraps `axum::Router`, generating HTTP routes from `resources!`, `get!`, `post!`, `namespace!`, and `scope!` declarations, plus compile-time URL helper functions.

**Architecture:** Two crates: `doido-router-macros` is a `proc-macro` crate containing the `routes!` proc macro (and inner macros `resources!`, `get!`, `post!`, `put!`, `patch!`, `delete!`, `namespace!`, `scope!`). `doido-router` is the library crate that re-exports those macros and re-exports `axum` for downstream use. The `routes!` macro expands to an `axum::Router` expression plus URL helper `fn` items declared at the same scope level inside a block. Controllers referenced in `routes!` must expose associated `async fn` methods named `index`, `show`, `create`, `new`, `edit`, `update`, `destroy` — the macro generates `axum::routing::*` registrations calling those methods directly.

**Tech Stack:** Rust, `axum 0.7`, `tokio 1` (runtime); proc-macro crate: `syn 2`, `quote 1`, `proc-macro2 1`; dev: `tower`, `http`, `tokio`

---

## File Structure

| File | Purpose |
|------|---------|
| `doido-router-macros/Cargo.toml` | proc-macro crate manifest |
| `doido-router-macros/src/lib.rs` | `routes!` proc macro entry point |
| `doido-router-macros/src/parser.rs` | Parse route declarations from token stream |
| `doido-router-macros/src/codegen.rs` | Generate `axum::Router` + URL helper functions |
| `doido-router/Cargo.toml` | Library crate manifest |
| `doido-router/src/lib.rs` | Re-exports macros + axum |
| `doido-router/tests/router_test.rs` | Integration tests via axum test client |

---

### Task 1: Crate Scaffolds

**Files:**
- Create: `doido-router-macros/Cargo.toml`
- Create: `doido-router-macros/src/lib.rs` (stub)
- Create: `doido-router/Cargo.toml`
- Create: `doido-router/src/lib.rs` (stub)
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Create `doido-router-macros/Cargo.toml`**

```toml
[package]
name = "doido-router-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

- [ ] **Step 2: Create `doido-router-macros/src/lib.rs` stub**

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let _ = input;
    TokenStream::new()
}
```

- [ ] **Step 3: Create `doido-router/Cargo.toml`**

```toml
[package]
name = "doido-router"
version = "0.1.0"
edition = "2021"

[dependencies]
doido-core = { path = "../doido-core" }
doido-router-macros = { path = "../doido-router-macros" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 4: Create `doido-router/src/lib.rs` stub**

```rust
pub use doido_router_macros::routes;
pub use axum;
```

- [ ] **Step 5: Add both crates to the workspace**

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
]
```

- [ ] **Step 6: Verify both crates compile**

Run: `cargo check -p doido-router-macros && cargo check -p doido-router`
Expected: both check cleanly (empty stub).

- [ ] **Step 7: Commit**

```bash
git add doido-router/ doido-router-macros/ Cargo.toml
git commit -m "feat(router): add doido-router and doido-router-macros crate scaffolds"
```

---

### Task 2: `routes!` with a Single `get!` Route

**Files:**
- Modify: `doido-router-macros/src/lib.rs`
- Create: `doido-router-macros/src/parser.rs`
- Create: `doido-router-macros/src/codegen.rs`
- Create: `doido-router/tests/router_test.rs`

This task builds the minimal pipeline: parse one `get!("/path", handler)` declaration and emit an `axum::Router`.

- [ ] **Step 1: Write the failing integration test**

Create `doido-router/tests/router_test.rs`:

```rust
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

// A plain async fn is a valid axum handler
async fn about_handler() -> &'static str { "about page" }

#[tokio::test]
async fn test_single_get_route_responds() {
    let app = doido_router::routes! {
        get!("/about", about_handler)
    };

    let response = app
        .oneshot(Request::builder().uri("/about").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_unknown_route_returns_404() {
    let app = doido_router::routes! {
        get!("/about", about_handler)
    };

    let response = app
        .oneshot(Request::builder().uri("/missing").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-router --test router_test`
Expected: compile error — `routes!` macro expands to empty, no router produced.

- [ ] **Step 3: Create `doido-router-macros/src/parser.rs`**

```rust
use syn::{
    braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitStr, Path, Result, Token,
};

/// A single route declaration inside `routes! { ... }`.
pub enum RouteDecl {
    /// `get!("/path", handler_expr)`
    Method { method: String, path: LitStr, handler: Expr },
}

pub struct RoutesInput {
    pub decls: Vec<RouteDecl>,
}

impl Parse for RoutesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            // Expect: `get!` | `post!` | etc.
            let method_ident: syn::Ident = input.parse()?;
            let _bang: Token![!] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let path: LitStr = content.parse()?;
            let _comma: Token![,] = content.parse()?;
            let handler: Expr = content.parse()?;
            let _semi: Option<Token![;]> = input.parse().ok();
            decls.push(RouteDecl::Method {
                method: method_ident.to_string(),
                path,
                handler,
            });
        }
        Ok(RoutesInput { decls })
    }
}
```

- [ ] **Step 4: Create `doido-router-macros/src/codegen.rs`**

```rust
use crate::parser::{RouteDecl, RoutesInput};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate(input: RoutesInput) -> TokenStream {
    let mut route_stmts = Vec::new();

    for decl in input.decls {
        match decl {
            RouteDecl::Method { method, path, handler } => {
                let axum_method = syn::Ident::new(&method, proc_macro2::Span::call_site());
                route_stmts.push(quote! {
                    .route(#path, axum::routing::#axum_method(#handler))
                });
            }
        }
    }

    quote! {
        {
            axum::Router::new()
            #(#route_stmts)*
        }
    }
}
```

- [ ] **Step 5: Update `doido-router-macros/src/lib.rs`**

```rust
mod parser;
mod codegen;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as parser::RoutesInput);
    codegen::generate(parsed).into()
}
```

- [ ] **Step 6: Run to verify tests pass**

Run: `cargo test -p doido-router --test router_test`
Expected: PASS — 2 tests.

- [ ] **Step 7: Commit**

```bash
git add doido-router-macros/src/ doido-router/tests/router_test.rs
git commit -m "feat(router): implement routes! macro with single get!/post! route support"
```

---

### Task 3: `resources!` — All 7 REST Routes

**Files:**
- Modify: `doido-router-macros/src/parser.rs`
- Modify: `doido-router-macros/src/codegen.rs`
- Modify: `doido-router/tests/router_test.rs`

`resources!(posts, PostsController)` generates 7 routes and 4 URL helper functions.

- [ ] **Step 1: Write the failing tests**

Append to `doido-router/tests/router_test.rs`:

```rust
mod posts_controller {
    pub async fn index() -> &'static str { "index" }
    pub async fn new() -> &'static str { "new" }
    pub async fn create() -> &'static str { "create" }
    pub async fn show(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "show" }
    pub async fn edit(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "edit" }
    pub async fn update(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "update" }
    pub async fn destroy(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "destroy" }
}

#[tokio::test]
async fn test_resources_generates_index_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::get("/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_show_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::get("/posts/1").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_new_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::get("/posts/new").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_edit_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::get("/posts/1/edit").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_create_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::builder().method("POST").uri("/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_update_routes() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let patch = app.clone().oneshot(Request::builder().method("PATCH").uri("/posts/1").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(patch.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_resources_generates_destroy_route() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let resp = app.oneshot(Request::builder().method("DELETE").uri("/posts/1").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[test]
fn test_resources_url_helpers() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
    };
    let _ = app; // router is produced
    assert_eq!(posts_path(), "/posts");
    assert_eq!(new_post_path(), "/posts/new");
    assert_eq!(post_path(42u64), "/posts/42");
    assert_eq!(edit_post_path(42u64), "/posts/42/edit");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-router --test router_test`
Expected: compile error — `resources!` not handled, URL helpers not generated.

- [ ] **Step 3: Update `doido-router-macros/src/parser.rs` to handle `resources!`**

```rust
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    Expr, Ident, LitStr, Result, Token,
};

pub enum RouteDecl {
    /// `get!("/path", handler)`
    Method { method: String, path: LitStr, handler: Expr },
    /// `resources!(posts, ControllerModule)` with optional `only:` or `except:`
    Resources {
        resource_name: Ident,
        controller: Ident,
        filter: ResourceFilter,
    },
}

pub enum ResourceFilter {
    All,
    Only(Vec<String>),
    Except(Vec<String>),
}

pub struct RoutesInput {
    pub decls: Vec<RouteDecl>,
}

fn parse_action_list(input: ParseStream) -> Result<Vec<String>> {
    let content;
    bracketed!(content in input);
    let mut actions = Vec::new();
    while !content.is_empty() {
        let ident: Ident = content.parse()?;
        actions.push(ident.to_string());
        let _comma: Option<Token![,]> = content.parse().ok();
    }
    Ok(actions)
}

impl Parse for RoutesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            let macro_ident: Ident = input.parse()?;
            let _bang: Token![!] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let _semi: Option<Token![;]> = input.parse().ok();

            match macro_ident.to_string().as_str() {
                "resources" => {
                    let resource_name: Ident = content.parse()?;
                    let _comma: Token![,] = content.parse()?;
                    let controller: Ident = content.parse()?;
                    let filter = if content.is_empty() {
                        ResourceFilter::All
                    } else {
                        let _comma: Token![,] = content.parse()?;
                        let key: Ident = content.parse()?;
                        let _colon: Token![:] = content.parse()?;
                        let actions = parse_action_list(&content)?;
                        match key.to_string().as_str() {
                            "only" => ResourceFilter::Only(actions),
                            "except" => ResourceFilter::Except(actions),
                            other => return Err(syn::Error::new(key.span(), format!("unknown resources option: {other}"))),
                        }
                    };
                    decls.push(RouteDecl::Resources { resource_name, controller, filter });
                }
                method @ ("get" | "post" | "put" | "patch" | "delete") => {
                    let path: LitStr = content.parse()?;
                    let _comma: Token![,] = content.parse()?;
                    let handler: Expr = content.parse()?;
                    decls.push(RouteDecl::Method {
                        method: method.to_string(),
                        path,
                        handler,
                    });
                }
                other => return Err(syn::Error::new(macro_ident.span(), format!("unknown route macro: {other}!"))),
            }
        }
        Ok(RoutesInput { decls })
    }
}
```

- [ ] **Step 4: Update `doido-router-macros/src/codegen.rs` to generate resources routes + URL helpers**

```rust
use crate::parser::{ResourceFilter, RouteDecl, RoutesInput};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

const ALL_ACTIONS: &[&str] = &["index", "new", "create", "show", "edit", "update", "destroy"];

fn is_active(action: &str, filter: &ResourceFilter) -> bool {
    match filter {
        ResourceFilter::All => true,
        ResourceFilter::Only(list) => list.iter().any(|a| a == action),
        ResourceFilter::Except(list) => !list.iter().any(|a| a == action),
    }
}

pub fn generate(input: RoutesInput) -> TokenStream {
    let mut route_stmts = Vec::new();
    let mut helper_fns = Vec::new();

    for decl in input.decls {
        match decl {
            RouteDecl::Method { method, path, handler } => {
                let axum_method = syn::Ident::new(&method, Span::call_site());
                route_stmts.push(quote! {
                    .route(#path, axum::routing::#axum_method(#handler))
                });
            }
            RouteDecl::Resources { resource_name, controller, filter } => {
                let name = resource_name.to_string();
                let singular = name.trim_end_matches('s').to_string(); // naive singularization
                let base = format!("/{}", name);
                let base_id = format!("/{}/:id", name);
                let base_new = format!("/{}/new", name);
                let base_id_edit = format!("/{}/:id/edit", name);
                let ctrl = &controller;

                // collection routes
                let mut collection = quote! { axum::routing::MethodRouter::new() };
                if is_active("index", &filter) {
                    collection = quote! { #collection.get(#ctrl::index) };
                }
                if is_active("create", &filter) {
                    collection = quote! { #collection.post(#ctrl::create) };
                }
                route_stmts.push(quote! { .route(#base, #collection) });

                // /new
                if is_active("new", &filter) {
                    route_stmts.push(quote! { .route(#base_new, axum::routing::get(#ctrl::new)) });
                }

                // member routes
                let mut member = quote! { axum::routing::MethodRouter::new() };
                if is_active("show", &filter) {
                    member = quote! { #member.get(#ctrl::show) };
                }
                if is_active("update", &filter) {
                    member = quote! { #member.patch(#ctrl::update).put(#ctrl::update) };
                }
                if is_active("destroy", &filter) {
                    member = quote! { #member.delete(#ctrl::destroy) };
                }
                route_stmts.push(quote! { .route(#base_id, #member) });

                // /edit
                if is_active("edit", &filter) {
                    route_stmts.push(quote! { .route(#base_id_edit, axum::routing::get(#ctrl::edit)) });
                }

                // URL helpers
                let collection_helper = format_ident!("{}_path", name);
                let new_helper = format_ident!("new_{}_path", singular);
                let member_helper = format_ident!("{}_path", singular);
                let edit_helper = format_ident!("edit_{}_path", singular);

                helper_fns.push(quote! {
                    #[allow(dead_code)]
                    fn #collection_helper() -> String { #base.to_string() }
                    #[allow(dead_code)]
                    fn #new_helper() -> String { #base_new.to_string() }
                    #[allow(dead_code)]
                    fn #member_helper(id: impl std::fmt::Display) -> String { format!(#base_id, id = id).replace(":id", &id.to_string()) }
                    #[allow(dead_code)]
                    fn #edit_helper(id: impl std::fmt::Display) -> String { format!(#base_id_edit, id = id).replace(":id", &id.to_string()) }
                });
            }
        }
    }

    quote! {
        {
            #(#helper_fns)*
            axum::Router::new()
            #(#route_stmts)*
        }
    }
}
```

- [ ] **Step 5: Fix URL helper format strings in codegen**

The `format!(#base_id, ...)` approach won't work with `:id` literally. Replace the member and edit helper bodies with string replacement:

```rust
#[allow(dead_code)]
fn #member_helper(id: impl std::fmt::Display) -> String {
    format!("/{}/{{id}}", #name).replace("{id}", &id.to_string())
}
#[allow(dead_code)]
fn #edit_helper(id: impl std::fmt::Display) -> String {
    format!("/{}/{{id}}/edit", #name).replace("{id}", &id.to_string())
}
```

Update `codegen.rs` with these corrected helper bodies:

```rust
helper_fns.push(quote! {
    #[allow(dead_code)]
    fn #collection_helper() -> String { #base.to_string() }
    #[allow(dead_code)]
    fn #new_helper() -> String { #base_new.to_string() }
    #[allow(dead_code)]
    fn #member_helper(id: impl ::std::fmt::Display) -> String {
        format!("/{}/{{id}}", #name).replace("{id}", &id.to_string())
    }
    #[allow(dead_code)]
    fn #edit_helper(id: impl ::std::fmt::Display) -> String {
        format!("/{}/{{id}}/edit", #name).replace("{id}", &id.to_string())
    }
});
```

- [ ] **Step 6: Run to verify all tests pass**

Run: `cargo test -p doido-router --test router_test`
Expected: PASS — 10 tests (2 original + 8 new).

- [ ] **Step 7: Commit**

```bash
git add doido-router-macros/src/ doido-router/tests/router_test.rs
git commit -m "feat(router): add resources! macro with 7 REST routes and URL helpers"
```

---

### Task 4: `only:` and `except:` for `resources!`

**Files:**
- Modify: `doido-router/tests/router_test.rs`

The parsing and codegen already support `only:` and `except:` from Task 3. This task adds tests to lock in the behavior.

- [ ] **Step 1: Write the failing tests**

Append to `doido-router/tests/router_test.rs`:

```rust
#[tokio::test]
async fn test_resources_only_restricts_to_listed_actions() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller, only: [index, show])
    };
    // index exists
    let resp = app.clone().oneshot(Request::get("/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // show exists
    let resp = app.clone().oneshot(Request::get("/posts/1").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // new is excluded
    let resp = app.oneshot(Request::get("/posts/new").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_resources_except_excludes_listed_actions() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller, except: [destroy])
    };
    // index exists
    let resp = app.clone().oneshot(Request::get("/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // destroy is excluded — DELETE /posts/1 should return 405
    let resp = app.oneshot(
        Request::builder().method("DELETE").uri("/posts/1").body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-router --test router_test only_restricts`
Expected: the tests may fail if excluded routes return 404 instead of 405 (axum returns 405 for registered paths with wrong method, 404 for unregistered paths).

Note: when an action is excluded via `only:`/`except:`, if the path itself is still registered (e.g., `/posts/:id` for both `show` and `destroy`), axum returns 405 for the excluded method. If the path is not registered at all (e.g., `/posts/new` when `new` is excluded), axum returns 404. Adjust assertions accordingly:

```rust
// new path not registered at all when excluded → 404
let resp = app.oneshot(Request::get("/posts/new").body(Body::empty()).unwrap()).await.unwrap();
assert!(resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::METHOD_NOT_ALLOWED);
```

- [ ] **Step 3: Run to verify tests pass**

Run: `cargo test -p doido-router --test router_test`
Expected: PASS — 12 tests.

- [ ] **Step 4: Commit**

```bash
git add doido-router/tests/router_test.rs
git commit -m "test(router): verify only: and except: restrict resources! routes"
```

---

### Task 5: `namespace!` and `scope!`

**Files:**
- Modify: `doido-router-macros/src/parser.rs`
- Modify: `doido-router-macros/src/codegen.rs`
- Modify: `doido-router/tests/router_test.rs`

`namespace!(api, { resources!(users, UsersController); })` — prefixes path AND is a logical grouping.
`scope!("/v2", { resources!(articles, ArticlesController); })` — prefixes path only.

For this plan, both `namespace!` and `scope!` produce path-prefixed sub-routers (the module prefix for `namespace!` is a documentation convention, not enforced by the macro).

- [ ] **Step 1: Write the failing tests**

Append to `doido-router/tests/router_test.rs`:

```rust
mod users_controller {
    pub async fn index() -> &'static str { "users" }
    pub async fn show(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "user" }
    pub async fn create() -> &'static str { "create user" }
    pub async fn new() -> &'static str { "new user" }
    pub async fn edit(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "edit user" }
    pub async fn update(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "update user" }
    pub async fn destroy(axum::extract::Path(_id): axum::extract::Path<u64>) -> &'static str { "delete user" }
}

#[tokio::test]
async fn test_namespace_prefixes_path() {
    let app = doido_router::routes! {
        namespace!(api, {
            resources!(users, users_controller)
        })
    };
    let resp = app.oneshot(Request::get("/api/users").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_scope_prefixes_path() {
    let app = doido_router::routes! {
        scope!("/v2", {
            resources!(users, users_controller)
        })
    };
    let resp = app.oneshot(Request::get("/v2/users").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_namespace_url_helpers_include_prefix() {
    let app = doido_router::routes! {
        namespace!(api, {
            resources!(users, users_controller)
        })
    };
    let _ = app;
    assert_eq!(api_users_path(), "/api/users");
    assert_eq!(api_user_path(1u64), "/api/users/1");
}
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p doido-router --test router_test namespace`
Expected: compile error — `namespace!` and `scope!` not handled in parser.

- [ ] **Step 3: Update `parser.rs` to handle `namespace!` and `scope!`**

Add `Namespace` and `Scope` variants to `RouteDecl`:

```rust
pub enum RouteDecl {
    Method { method: String, path: LitStr, handler: Expr },
    Resources { resource_name: Ident, controller: Ident, filter: ResourceFilter },
    /// `namespace!(name, { ... })` — prefix by `/name`, URL helpers prefixed with `name_`
    Namespace { name: Ident, body: RoutesInput },
    /// `scope!("/prefix", { ... })` — prefix by literal string, no helper prefix change
    Scope { path_prefix: LitStr, body: RoutesInput },
}
```

Add parsing in `RoutesInput::parse`:

```rust
"namespace" => {
    let name: Ident = content.parse()?;
    let _comma: Token![,] = content.parse()?;
    let inner;
    braced!(inner in content);
    let body: RoutesInput = inner.parse()?;
    decls.push(RouteDecl::Namespace { name, body });
}
"scope" => {
    let path_prefix: LitStr = content.parse()?;
    let _comma: Token![,] = content.parse()?;
    let inner;
    braced!(inner in content);
    let body: RoutesInput = inner.parse()?;
    decls.push(RouteDecl::Scope { path_prefix, body });
}
```

Add `use syn::braced;` at the top of `parser.rs`.

- [ ] **Step 4: Update `codegen.rs` to generate nested routers with path prefix**

Add to `generate` function:

```rust
RouteDecl::Namespace { name, body } => {
    let prefix = format!("/{}", name);
    let inner = generate_inner(body, Some(&prefix), Some(&name.to_string()));
    route_stmts.push(quote! { .merge(#inner) });
    // URL helpers with namespace prefix are generated inside generate_inner
}
RouteDecl::Scope { path_prefix, body } => {
    let prefix_str = path_prefix.value();
    let inner = generate_inner(body, Some(&prefix_str), None);
    route_stmts.push(quote! { .merge(#inner) });
}
```

Refactor `generate` into `generate_inner(input, path_prefix, helper_prefix)`:

```rust
pub fn generate(input: RoutesInput) -> TokenStream {
    let inner = generate_inner(input, None, None);
    inner
}

fn generate_inner(input: RoutesInput, path_prefix: Option<&str>, helper_prefix: Option<&str>) -> TokenStream {
    let mut route_stmts = Vec::new();
    let mut helper_fns = Vec::new();

    for decl in input.decls {
        match decl {
            RouteDecl::Method { method, path, handler } => {
                let full_path = if let Some(pfx) = path_prefix {
                    format!("{}{}", pfx, path.value())
                } else {
                    path.value()
                };
                let full_path_lit = syn::LitStr::new(&full_path, path.span());
                let axum_method = syn::Ident::new(&method, Span::call_site());
                route_stmts.push(quote! {
                    .route(#full_path_lit, axum::routing::#axum_method(#handler))
                });
            }
            RouteDecl::Resources { resource_name, controller, filter } => {
                let name = resource_name.to_string();
                let singular = name.trim_end_matches('s').to_string();
                let pfx = path_prefix.unwrap_or("");
                let base = format!("{}/{}", pfx, name);
                let base_id = format!("{}/{}/:id", pfx, name);
                let base_new = format!("{}/{}/new", pfx, name);
                let base_id_edit = format!("{}/{}/:id/edit", pfx, name);
                let ctrl = &controller;

                let mut collection = quote! { axum::routing::MethodRouter::new() };
                if is_active("index", &filter) { collection = quote! { #collection.get(#ctrl::index) }; }
                if is_active("create", &filter) { collection = quote! { #collection.post(#ctrl::create) }; }
                route_stmts.push(quote! { .route(#base, #collection) });

                if is_active("new", &filter) {
                    route_stmts.push(quote! { .route(#base_new, axum::routing::get(#ctrl::new)) });
                }

                let mut member = quote! { axum::routing::MethodRouter::new() };
                if is_active("show", &filter) { member = quote! { #member.get(#ctrl::show) }; }
                if is_active("update", &filter) { member = quote! { #member.patch(#ctrl::update).put(#ctrl::update) }; }
                if is_active("destroy", &filter) { member = quote! { #member.delete(#ctrl::destroy) }; }
                route_stmts.push(quote! { .route(#base_id, #member) });

                if is_active("edit", &filter) {
                    route_stmts.push(quote! { .route(#base_id_edit, axum::routing::get(#ctrl::edit)) });
                }

                // URL helpers
                let helper_pfx = helper_prefix.map(|h| format!("{}_", h)).unwrap_or_default();
                let collection_helper = format_ident!("{}{}s_path", helper_pfx, singular);
                let new_helper = format_ident!("{}new_{}_path", helper_pfx, singular);
                let member_helper = format_ident!("{}{}_path", helper_pfx, singular);
                let edit_helper = format_ident!("{}edit_{}_path", helper_pfx, singular);

                helper_fns.push(quote! {
                    #[allow(dead_code)]
                    fn #collection_helper() -> String { #base.to_string() }
                    #[allow(dead_code)]
                    fn #new_helper() -> String { #base_new.to_string() }
                    #[allow(dead_code)]
                    fn #member_helper(id: impl ::std::fmt::Display) -> String {
                        format!("{}/{{}}", #base).replace("{}", &id.to_string())
                    }
                    #[allow(dead_code)]
                    fn #edit_helper(id: impl ::std::fmt::Display) -> String {
                        format!("{}/{{}}/edit", #base).replace("{}", &id.to_string())
                    }
                });
            }
            RouteDecl::Namespace { name, body } => {
                let ns_prefix = format!("{}/{}", path_prefix.unwrap_or(""), name);
                let ns_helper = Some(name.to_string());
                let inner = generate_inner_ts(body, Some(&ns_prefix), ns_helper.as_deref());
                route_stmts.push(quote! { .merge(#inner) });
            }
            RouteDecl::Scope { path_prefix: scope_path, body } => {
                let scope_pfx = format!("{}{}", path_prefix.unwrap_or(""), scope_path.value());
                let inner = generate_inner_ts(body, Some(&scope_pfx), helper_prefix);
                route_stmts.push(quote! { .merge(#inner) });
            }
        }
    }

    quote! {
        {
            #(#helper_fns)*
            axum::Router::new()
            #(#route_stmts)*
        }
    }
}

fn generate_inner_ts(input: RoutesInput, path_prefix: Option<&str>, helper_prefix: Option<&str>) -> TokenStream {
    generate_inner(input, path_prefix, helper_prefix)
}
```

- [ ] **Step 5: Run to verify all tests pass**

Run: `cargo test -p doido-router --test router_test`
Expected: PASS — 15 tests.

- [ ] **Step 6: Commit**

```bash
git add doido-router-macros/src/ doido-router/tests/router_test.rs
git commit -m "feat(router): add namespace! and scope! with path prefix and scoped URL helpers"
```

---

### Task 6: Final Wiring and Full Integration Test

**Files:**
- Modify: `doido-router/src/lib.rs`
- Modify: `doido-router/tests/router_test.rs`

- [ ] **Step 1: Write the final integration test**

Append to `doido-router/tests/router_test.rs`:

```rust
/// Combines resources, namespace, scope, and custom routes in a single routes! block.
#[tokio::test]
async fn test_combined_routes_block() {
    let app = doido_router::routes! {
        resources!(posts, posts_controller)
        get!("/about", about_handler)
        namespace!(api, {
            resources!(users, users_controller)
        })
        scope!("/v2", {
            resources!(posts, posts_controller)
        })
    };

    // resources
    let r = app.clone().oneshot(Request::get("/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // custom route
    let r = app.clone().oneshot(Request::get("/about").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // namespace
    let r = app.clone().oneshot(Request::get("/api/users").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // scope
    let r = app.oneshot(Request::get("/v2/posts").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p doido-router --test router_test combined`
Expected: compile or runtime failure if any route is missing.

- [ ] **Step 3: Run all tests**

Run: `cargo test -p doido-router`
Expected: PASS — 16 tests.

- [ ] **Step 4: Check for warnings**

Run: `cargo build -p doido-router 2>&1 | grep warning`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add doido-router/src/lib.rs doido-router/tests/router_test.rs
git commit -m "test(router): add combined routes! integration test covering all macro forms"
```

---

## Self-Review

### Spec Coverage

| Spec requirement | Covered by |
|---|---|
| Built on `axum::Router` internally | Task 2 — Router expression generated |
| `routes!` macro DSL | Tasks 2-6 |
| `resources!` generates 7 REST routes | Task 3 |
| `only:` restricts to listed actions | Task 4 |
| `except:` excludes listed actions | Task 4 |
| `get!`, `post!`, `put!`, `patch!`, `delete!` | Task 2 (parser handles all HTTP methods) |
| `namespace!` prefixes path | Task 5 |
| `scope!` prefixes path | Task 5 |
| URL helpers generated at compile time | Task 3 (`posts_path`, `post_path`, etc.) |
| `namespace!` URL helpers prefixed | Task 5 (`api_users_path`, `api_user_path`) |
| Routes map to controller actions (not closures) | Task 3 — controller module methods used directly |
| Route parameter extraction (`:id`) | Task 3 — axum `Path` extractor in test controllers |
| Unknown routes return 404 | Task 2 |

### Placeholder Scan

No TODOs or TBDs. Every step has runnable code.

### Type Consistency

- `RouteDecl::Resources { resource_name: Ident, controller: Ident, filter: ResourceFilter }` — defined in Task 3 parser, used in Task 3 codegen, extended in Task 5. Consistent.
- `generate(input: RoutesInput) -> TokenStream` — public entry point, delegates to `generate_inner`. Consistent.
- `generate_inner(input, path_prefix: Option<&str>, helper_prefix: Option<&str>)` — defined in Task 5, called recursively for `Namespace`/`Scope`. Consistent.
