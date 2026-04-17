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

    pub fn layout(mut self, name: impl Into<String>) -> Self {
        self.layout = Some(name.into());
        self
    }

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
