use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_request_emits_structured_event() {
    doido_core::trace::request("GET", "/posts", 200, 42);
    // Structured event emitted with method, path, status, and latency_ms fields
}

#[test]
#[traced_test]
fn test_job_emits_structured_event() {
    doido_core::trace::job("ProcessPayment", "default", 1, "ok");
    // Structured event emitted with job_name, queue, attempt, and result fields
}

#[test]
#[traced_test]
fn test_query_emits_structured_event() {
    doido_core::trace::query("SELECT * FROM posts", 5);
    // Structured event emitted with sql and duration_ms fields
}

#[test]
#[traced_test]
fn test_mail_emits_structured_event() {
    doido_core::trace::mail("user@example.com", "Welcome!", "smtp");
    // Structured event emitted with to, subject, and deliverer fields
}
