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
