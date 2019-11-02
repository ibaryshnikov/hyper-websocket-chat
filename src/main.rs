use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::upgrade::Upgraded;
use tokio::sync::Mutex;
use tokio_io::split::WriteHalf;

mod endpoints;
mod shared_types;
mod utils;

use crate::endpoints::*;
use crate::shared_types::*;

pub type ClientsMap = HashMap<u128, WriteHalf<Upgraded>>;
pub type ClientsArc = Arc<Mutex<ClientsMap>>;

async fn request_router(
    req: Request<Body>,
    clients: ClientsArc,
) -> StdResult<Response<Body>, Infallible> {
    println!("req uri in endpoint {}", req.uri());
    let response = match &req.uri().to_string()[..] {
        "/" => hello(),
        "/ws" => handle_ws(req, clients),
        _ => not_found(),
    };
    Ok(response)
}

//fn service_fn_wrapper(clients: ClientsMap) ->

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:8081".parse()?;

    let clients = Arc::new(Mutex::new(HashMap::new()));
    let new_svc = make_service_fn(move |_addr| {
        let clients = clients.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| request_router(req, clients.clone()))) }
    });

    println!("Listening at http://{}", addr);

    Server::bind(&addr.into()).serve(new_svc).await?;

    Ok(())
}
