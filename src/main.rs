use std::convert::Infallible;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{CONNECTION, UPGRADE, HeaderValue, SEC_WEBSOCKET_KEY};
use hyper::upgrade::Upgraded;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

async fn handle_upgraded_connection(mut upgraded: Upgraded) -> Result<()> {
    println!("before write");
    upgraded
      .write(&[0x81, 0x11])
      .await?;
    upgraded.write_all(b"Hello from hyper!").await?;
    println!("after write");

    Ok(())
}

fn handle_ws(req: Request<Body>) -> Response<Body> {
    println!("ws incoming connection");
    let key = req
      .headers()
      .get("sec-websocket-key")
      .unwrap();

    let mut concatenated = key
      .to_str()
      .unwrap()
      .to_owned();

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


    concatenated.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

    let mut hasher = Sha1::new();
    hasher.input_str(&concatenated);

    let mut hash = [0; 20];
    hasher.result(&mut hash);
    println!("hash {:?}", hash);
    println!("hash len {}", hash.len());
    let accept = base64::encode(&hash);
    println!("accept {}", accept);

    Response::builder()
      .status(StatusCode::SWITCHING_PROTOCOLS)
      .header("access-control-allow-origin", "*")
      .header("upgrade", "websocket")
      .header("connection", "upgrade")
      .header("sec-websocket-accept", accept)
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
