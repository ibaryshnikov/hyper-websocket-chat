use futures::{SinkExt, StreamExt};
use hyper::upgrade::Upgraded;
use hyper::{Body, Request, Response, StatusCode};
use tokio::io::{AsyncRead, AsyncReadExt};
use uuid::Uuid;

use super::ws_consts::*;
use super::ws_utils::*;
use crate::shared::types::*;
use crate::ClientsArc;

pub async fn write_messages(mut stream: Receiver, clients: ClientsArc) -> Result<()> {
    while let Some(frame) = stream.next().await {
        let buffer = match frame.kind {
            FrameKind::Text => encode_text(&frame.data),
            FrameKind::Binary => encode_binary(&frame.data),
            FrameKind::Close => encode_close_frame(),
        };
        let mut clients = clients.lock().await;
        println!("broadcasting {:?} to {:?}", frame.kind, frame.address);
        broadcast_buffer(&mut *clients, frame.address, &buffer).await?;
    }

    println!("Stopping writing messages");

    Ok(())
}

async fn read_messages<T>(mut reader: T, mut sink: Sender, id: u128) -> Result<()>
where
    T: AsyncRead + Unpin,
{
    send_messages(&mut sink, id).await?;
    loop {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf).await?;
        let fin_bit = (buf[0] & FIN_BIT_MASK) >> 7;
        println!("fin bit is {}", fin_bit);
        let opcode = buf[0] & OPCODE_MASK;
        let length = buf[1] & LENGTH_MASK;
        let masked = (buf[1] & MASKED_MASK) >> 7;
        println!("opcode {:#x}", opcode);
        println!("length {:#x} {}", length, length);
        println!("is masked: {}", masked);
        match opcode {
            OPCODE_CONTINUATION => println!("continuation frame opcode"),
            OPCODE_TEXT => println!("text opcode"),
            OPCODE_BINARY => println!("binary opcode"),
            OPCODE_CLOSE => {
                println!("close opcode");
                return Ok(());
            }
            OPCODE_PING => println!("ping opcode"),
            OPCODE_PONG => println!("pong opcode"),
            _ => {
                println!("unexpected opcode {}", opcode);
                break;
            }
        };
        let (mask, mut bytes) = match length {
            0..=125 => {
                let mask_buf = read_mask(&mut reader).await?;
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

        match String::from_utf8(bytes) {
            Ok(msg) => {
                println!("got message: {}", msg);
                let bytes = id.to_be_bytes();
                #[rustfmt::skip]
                let short_id = u32::from_be_bytes([
                    bytes[0],
                    bytes[1],
                    bytes[2],
                    bytes[3],
                ]);
                let new_msg = format!("client {:#x}: {}", short_id, msg).into_bytes();
                sink.send(Frame {
                    kind: FrameKind::Text,
                    address: FrameAddress::All,
                    data: new_msg,
                })
                .await?;
            }
            Err(e) => println!("error parsing a string: {}", e),
        }
    }
    Ok(())
}

async fn send_messages(sender: &mut Sender, id: u128) -> Result<()> {
    sender
        .send(Frame {
            kind: FrameKind::Text,
            address: FrameAddress::Client(id),
            data: b"Welcome to chat server!".to_vec(),
        })
        .await?;
    Ok(())
}

async fn handle_upgraded_connection(
    upgraded: Upgraded,
    sender: Sender,
    clients: ClientsArc,
) -> Result<()> {
    let (reader, writer) = tokio::io::split(upgraded);

    let id = Uuid::new_v4().to_u128_le();
    clients.lock().await.insert(id, writer);

    read_messages(reader, sender, id).await?;

    Ok(())
}

pub fn handle_ws(req: Request<Body>, sender: Sender, clients: ClientsArc) -> Response<Body> {
    println!("ws incoming connection");
    let sec_key = req.headers().get("sec-websocket-key").unwrap();

    let sec_accept = generate_key_from(sec_key.as_bytes());

    hyper::rt::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                println!("upgraded");
                if let Err(e) = handle_upgraded_connection(upgraded, sender, clients).await {
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
