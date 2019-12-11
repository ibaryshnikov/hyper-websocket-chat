use hyper::{Body, Response, StatusCode};

use crate::shared::utils::*;

pub fn not_found() -> Response<Body> {
    let mut response = Response::new(Body::from("Not found\n"));
    apply_cors(&mut response);
    apply_content_type(&mut response);
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}
