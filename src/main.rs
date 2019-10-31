use std::convert::Infallible;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{CONNECTION, UPGRADE, HeaderValue, SEC_WEBSOCKET_KEY};
use hyper::upgrade::Upgraded;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod utils;
use utils::*;

type StdResult<T, E> = std::result::Result<T, E>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn header_value(source: &'static str) -> HeaderValue {
    HeaderValue::from_static(source)
}

fn apply_cors(response: &mut Response<Body>) {
    response
      .headers_mut()
      .insert("AccessControlAllowOrigin", header_value("*"));
}

fn apply_content_type(response: &mut Response<Body>) {
    response
      .headers_mut()
      .insert("Content-Type", header_value("text/plain; charset=utf-8"));
}

fn hello() -> Response<Body> {
    let mut response = Response::new(Body::from("Hello from hyper!"));
    apply_cors(&mut response);
    apply_content_type(&mut response);
    response
}

async fn send_unmasked_single_frame_text(upgraded: &mut Upgraded, msg: &[u8]) -> Result<()> {
    let first_byte: &[u8] = &[0x81];
    let encoded_length = get_length_bytes(msg.len());
    let payload = [first_byte, &encoded_length, msg].concat();
    upgraded.write_all(&payload).await
}

async fn send_unmasked_text(upgraded: &mut Upgraded, msg: &[u8]) -> Result<()> {
    send_unmasked_single_frame_text(upgraded, msg).await
}

async fn handle_upgraded_connection(mut upgraded: Upgraded) -> Result<()> {
    send_unmasked_text(&mut upgraded, b"Hello from hyper!").await?;
    send_unmasked_text(&mut upgraded, "Hello".repeat(100).as_bytes()).await?;
    send_unmasked_text(&mut upgraded, "Hello".repeat(20000).as_bytes()).await?;
    Ok(())
}

fn handle_ws(req: Request<Body>) -> Response<Body> {
    println!("ws incoming connection");
    let sec_key = req
      .headers()
      .get("sec-websocket-key")
      .unwrap();

    let sec_accept = generate_key_from(sec_key.as_bytes());

    hyper::rt::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                println!("upgraded");
                if let Err(e) = handle_upgraded_connection(upgraded).await {
                    println!("error handling upgraded connection: {}", e);
                }
            }
            Err(e) => println!("upgrade error: {}", e),
        }
    });

    Response::builder()
      .status(StatusCode::SWITCHING_PROTOCOLS)
      .header("access-control-allow-origin", "*")
      .header("upgrade", "websocket")
      .header("connection", "upgrade")
      .header("sec-websocket-accept", sec_accept)
      .body(Body::empty())
      .unwrap()
}

fn not_found() -> Response<Body> {
    let mut response = Response::new(Body::from("Not found"));
    apply_cors(&mut response);
    apply_content_type(&mut response);
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}

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
async fn main() -> Result<()>{
    let addr = ([127, 0, 0, 1], 8000).into();

    let new_svc = make_service_fn(|_addr| {
        async {
            Ok::<_, Infallible>(service_fn(request_router))
        }
    });

    let server = Server::bind(&addr)
      .serve(new_svc);

    println!("Listening at http://127.0.0.1:8000");

    server.await?;

    Ok(())
}
