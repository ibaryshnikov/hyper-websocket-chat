use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

mod endpoints;
mod shared_types;
mod utils;

use crate::endpoints::*;
use crate::shared_types::*;

async fn request_router(req: Request<Body>) -> StdResult<Response<Body>, Infallible> {
    println!("req uri in endpoint {}", req.uri());
    let response = match &req.uri().to_string()[..] {
        "/" => hello(),
        "/ws" => handle_ws(req),
        _ => not_found(),
    };
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = ([127, 0, 0, 1], 8000).into();

    let new_svc =
        make_service_fn(|_addr| async { Ok::<_, Infallible>(service_fn(request_router)) });

    let server = Server::bind(&addr).serve(new_svc);

    println!("Listening at http://127.0.0.1:8000");

    server.await?;

    Ok(())
}
