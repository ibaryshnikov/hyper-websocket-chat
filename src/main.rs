use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::rc::Rc;

#[macro_use]
extern crate anyhow;

use anyhow::Result;
use futures::future::try_join;
use futures::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::sync::mpsc::unbounded_channel;

mod endpoints;
mod shared;
mod ws;

use endpoints::*;
use shared::types::*;
use ws::broadcast;

#[derive(Clone, Copy, Debug)]
struct LocalExecutor;

impl<F> hyper::rt::Executor<F> for LocalExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn_local(fut);
    }
}

fn main() -> Result<()> {
    let mut runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Should build runtime");

    let local = tokio::task::LocalSet::new();
    local.block_on(&mut runtime, serve())?;

    Ok(())
}

async fn serve() -> Result<()> {
    let addr = "0.0.0.0:8081".parse()?;

    let clients = Rc::new(RefCell::new(HashMap::new()));
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
    let server_future = Server::bind(&addr)
        .executor(LocalExecutor)
        .serve(service)
        .map_err(Into::into);

    println!("Listening at http://{}", addr);

    try_join(write_future, server_future).await?;

    Ok(())
}

async fn request_router(
    req: Request<Body>,
    sender: Sender,
    clients: ClientsRc,
) -> Result<Response<Body>, Infallible> {
    println!("req uri in endpoint {}", req.uri());
    let response = match &req.uri().to_string()[..] {
        "/" => hello(),
        "/ws" => handle_ws(req, sender, clients),
        _ => not_found(),
    };
    Ok(response)
}
