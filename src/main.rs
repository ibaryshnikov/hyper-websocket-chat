use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

#[macro_use]
extern crate anyhow;

use anyhow::Result;

use futures::future::try_join;
use futures::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;

mod endpoints;
mod shared;
mod ws;

use endpoints::*;
use shared::types::*;
use ws::broadcast;

async fn request_router(
    req: Request<Body>,
    sender: Sender,
    clients: ClientsArc,
) -> Result<Response<Body>, Infallible> {
    println!("req uri in endpoint {}", req.uri());
    let response = match &req.uri().to_string()[..] {
        "/" => hello(),
        "/ws" => handle_ws(req, sender, clients),
        _ => not_found(),
    };
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:8081".parse()?;

    let clients = Arc::new(Mutex::new(HashMap::new()));
    let (sender, receiver) = unbounded_channel();

    let write_future = broadcast(receiver, clients.clone());

    let service = make_service_fn(move |_addr| {
        let clients = clients.clone();
        let sender = sender.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                request_router(req, sender.clone(), clients.clone())
            }))
        }
    });
    let server_future = Server::bind(&addr).serve(service).map_err(Into::into);

    println!("Listening at http://{}", addr);

    try_join(write_future, server_future).await?;

    Ok(())
}
