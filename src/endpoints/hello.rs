use hyper::{Body, Response};

use crate::utils::*;

pub fn hello() -> Response<Body> {
    let mut response = Response::new(Body::from("Hello from hyper!"));
    apply_cors(&mut response);
    apply_content_type(&mut response);
    response
}
