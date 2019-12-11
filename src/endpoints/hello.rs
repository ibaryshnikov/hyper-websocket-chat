use hyper::{Body, Response};

use crate::shared::utils::*;

pub fn hello() -> Response<Body> {
    let mut response = Response::new(Body::from("Hello from hyper!\n"));
    apply_cors(&mut response);
    apply_content_type(&mut response);
    response
}
