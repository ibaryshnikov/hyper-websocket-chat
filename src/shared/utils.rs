use hyper::header::HeaderValue;
use hyper::{Body, Response};

fn header_value(source: &'static str) -> HeaderValue {
    HeaderValue::from_static(source)
}

pub fn apply_cors(response: &mut Response<Body>) {
    response
        .headers_mut()
        .insert("AccessControlAllowOrigin", header_value("*"));
}

pub fn apply_content_type(response: &mut Response<Body>) {
    response
        .headers_mut()
        .insert("Content-Type", header_value("text/plain; charset=utf-8"));
}
