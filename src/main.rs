use std::convert::Infallible;

use futures::future::try_join3;
use futures::{Sink, SinkExt, Stream, StreamExt};
use hyper::header::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;
use hyper::{Body, Request, Response, Server, StatusCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::prelude::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::unbounded_channel;

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

async fn send_close_frame<T: AsyncWrite + Unpin>(writer: &mut T) -> Result<()> {
    let msg = [0x8 | 0b1000_0000];
    writer.write_all(&msg).await?;
    Ok(())
}

async fn send_single_frame_text<T: AsyncWrite + Unpin>(writer: &mut T, msg: &[u8]) -> Result<()> {
    let first_byte: &[u8] = &[0x81];
    let length = encode_length(msg.len());
    let payload = [first_byte, &length, msg].concat();
    writer.write_all(&payload).await?;
    Ok(())
}

async fn send_text<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
    send_single_frame_text(upgraded, msg).await
}

async fn send_single_frame_binary<T: AsyncWrite + Unpin>(writer: &mut T, msg: &[u8]) -> Result<()> {
    let first_byte: &[u8] = &[0x82];
    let length = encode_length(msg.len());
    let payload = [first_byte, &length, msg].concat();
    writer.write_all(&payload).await?;
    Ok(())
}

async fn send_binary<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
    send_single_frame_binary(upgraded, msg).await
}

#[derive(Debug)]
enum FrameKind {
    Text,
    Binary,
    Close,
}

struct Frame {
    kind: FrameKind,
    data: Vec<u8>,
}

async fn write_messages<T, S>(mut writer: T, mut stream: S) -> Result<()>
where
    T: AsyncWrite + Unpin,
    S: Stream<Item = Frame> + Unpin,
{
    while let Some(frame) = stream.next().await {
        println!("got frame {:?}", frame.kind);
        match frame.kind {
            FrameKind::Text => send_text(&mut writer, &frame.data).await?,
            FrameKind::Binary => send_binary(&mut writer, &frame.data).await?,
            FrameKind::Close => {
                send_close_frame(&mut writer).await?;
                break;
            }
        }
    }

    println!("stopping writing messages");

    Ok(())
}

enum Length {
    U16,
    U64,
}

async fn read_length<T: AsyncRead + Unpin>(kind: Length, reader: &mut T) -> Result<usize> {
    let length = match kind {
        Length::U16 => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).await?;
            u16::from_be_bytes(buf) as usize
        }
        Length::U64 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).await?;
            u64::from_be_bytes(buf) as usize
        }
    };
    Ok(length)
}

async fn read_mask<T: AsyncRead + Unpin>(reader: &mut T) -> Result<[u8; 4]> {
    let mut mask_buf = [0u8; 4];
    reader.read_exact(&mut mask_buf).await?;
    Ok(mask_buf)
}

async fn read_messages<T, S>(mut reader: T, mut sink: S) -> Result<()>
where
    T: AsyncRead + Unpin,
    S: Sink<Frame> + Unpin + Clone,
    <S as Sink<Frame>>::Error: std::error::Error + Send + Sync + 'static,
{
    loop {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).await?;
        println!("{:#X?}", buf);
        let opcode = buf[0] & 0b0000_1111;
        let length = buf[1] & 0b0111_1111;
        println!("opcode {:#x}", opcode);
        println!("length {:#x} {}", length, length);
        match opcode {
            0x0 => println!("continuation frame opcode"),
            0x1 => println!("text opcode"),
            0x2 => println!("binary opcode"),
            0x8 => {
                println!("close opcode");
                return Ok(());
            }
            0x9 => println!("ping opcode"),
            0xA => println!("pong opcode"),
            _ => {
                println!("unexpected opcode {}", opcode);
                break;
            }
        };
        let (mask, mut bytes) = match length {
            0..=125 => {
                let mask_buf = read_mask(&mut reader).await?;
                println!("mask buf {:#x?}", mask_buf);
                let mut buf = vec![0; length as usize];
                reader.read_exact(&mut buf).await?;
                (mask_buf, buf)
            }
            126 => {
                let length = read_length(Length::U16, &mut reader).await?;
                let mask_buf = read_mask(&mut reader).await?;
                let mut buf = vec![0; length];
                reader.read_exact(&mut buf).await?;
                (mask_buf, buf)
            }
            127 => {
                let length = read_length(Length::U64, &mut reader).await?;
                let mask_buf = read_mask(&mut reader).await?;
                let mut buf = vec![0; length];
                reader.read_exact(&mut buf).await?;
                (mask_buf, buf)
            }
            _ => {
                println!("unexpected length value {:#X}", length);
                break;
            }
        };

        // unmasking the message
        for i in 0..bytes.len() {
            bytes[i] ^= mask[i % 4];
        }

        println!("buffer {:#x?}", bytes);
        match String::from_utf8(bytes) {
            Ok(msg) => {
                println!("got message: {}", msg);
                sink.send(Frame {
                    kind: FrameKind::Text,
                    data: msg.into_bytes(),
                })
                .await?;
            }
            Err(e) => println!("error parsing a string: {}", e),
        }
    }
    Ok(())
}

async fn send_messages<T>(mut sender: T) -> Result<()>
where
    T: Sink<Frame> + Unpin,
    <T as Sink<Frame>>::Error: std::error::Error + Send + Sync + 'static,
{
    sender
        .send(Frame {
            kind: FrameKind::Text,
            data: b"Hello from hyper!".to_vec(),
        })
        .await?;
    sender
        .send(Frame {
            kind: FrameKind::Text,
            data: "Hello".repeat(100).into_bytes(),
        })
        .await?;
    sender
        .send(Frame {
            kind: FrameKind::Text,
            data: "Hello".repeat(20000).into_bytes(),
        })
        .await?;
    sender
        .send(Frame {
            kind: FrameKind::Binary,
            data: vec![1, 2, 3],
        })
        .await?;
    Ok(())
}

async fn handle_upgraded_connection(upgraded: Upgraded) -> Result<()> {
    let (reader, writer) = tokio::io::split(upgraded);

    let (sender, receiver) = unbounded_channel::<Frame>();
    let write_future = write_messages(writer, receiver);
    let read_future = read_messages(reader, sender.clone());
    let send_future = send_messages(sender);

    try_join3(write_future, read_future, send_future).await?;

    Ok(())
}

fn handle_ws(req: Request<Body>) -> Response<Body> {
    println!("ws incoming connection");
    let sec_key = req.headers().get("sec-websocket-key").unwrap();

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
async fn main() -> Result<()> {
    let addr = ([127, 0, 0, 1], 8000).into();

    let new_svc =
        make_service_fn(|_addr| async { Ok::<_, Infallible>(service_fn(request_router)) });

    let server = Server::bind(&addr).serve(new_svc);

    println!("Listening at http://127.0.0.1:8000");

    server.await?;

    Ok(())
}
