//! Test helper functions

use actix_web::{dev::ServiceResponse, test, web, App};
use serde::de::DeserializeOwned;

/// Extract JSON body from a test response
pub async fn response_json<T: DeserializeOwned>(resp: ServiceResponse) -> T {
    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).expect("Failed to parse response as JSON")
}

/// Assert response status code
pub fn assert_status(resp: &ServiceResponse, expected: u16) {
    assert_eq!(
        resp.status().as_u16(),
        expected,
        "Expected status {}, got {}",
        expected,
        resp.status()
    );
}

/// Assert response is success (2xx)
pub fn assert_success(resp: &ServiceResponse) {
    assert!(
        resp.status().is_success(),
        "Expected success status, got {}",
        resp.status()
    );
}

/// Assert response is created (201)
pub fn assert_created(resp: &ServiceResponse) {
    assert_status(resp, 201);
}

/// Assert response is no content (204)
pub fn assert_no_content(resp: &ServiceResponse) {
    assert_status(resp, 204);
}

/// Assert response is not found (404)
pub fn assert_not_found(resp: &ServiceResponse) {
    assert_status(resp, 404);
}

/// Assert response is bad request (400)
pub fn assert_bad_request(resp: &ServiceResponse) {
    assert_status(resp, 400);
}

/// Assert response is unauthorized (401)
pub fn assert_unauthorized(resp: &ServiceResponse) {
    assert_status(resp, 401);
}
