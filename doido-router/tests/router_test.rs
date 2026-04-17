use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

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
