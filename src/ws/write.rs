use anyhow::Result;
use futures::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::encoding::encode_length;
use super::event::EventAddress;
use crate::shared::types::*;

use super::consts::*;
use super::event::*;
use super::opcode::Opcode;

pub fn encode_close_frame() -> Vec<u8> {
    [Opcode::Close.encode() | FIN_MASK].to_vec()
}

pub fn encode_text(msg: &[u8]) -> Vec<u8> {
    let first_byte: &[u8] = &[Opcode::Text.encode() | FIN_MASK];
    let length = encode_length(msg.len());
    [first_byte, &length, msg].concat()
}

pub fn encode_binary(msg: &[u8]) -> Vec<u8> {
    let first_byte: &[u8] = &[Opcode::Binary.encode() | FIN_MASK];
    let length = encode_length(msg.len());
    [first_byte, &length, msg].concat()
}

pub async fn broadcast_buffer(
    clients: &mut ClientsMap,
    address: EventAddress,
    buffer: &[u8],
) -> Result<()> {
    match address {
        EventAddress::All => {
            for writer in clients.values_mut() {
                writer.write_all(buffer).await?;
            }
        }
        EventAddress::Client(id) => {
            if let Some(writer) = clients.get_mut(&id) {
                writer.write_all(buffer).await?;
            } else {
                println!("Can't send message, client {} not found", id);
            }
        }
        EventAddress::ClientRange(list) => {
            for id in list {
                if let Some(writer) = clients.get_mut(&id) {
                    writer.write_all(buffer).await?;
                } else {
                    println!("Can't send message, client {} not found", id);
                }
            }
        }
    }
    Ok(())
}

pub async fn broadcast(mut stream: Receiver, clients: ClientsArc) -> Result<()> {
    while let Some(event) = stream.next().await {
        let buffer = match event.kind {
            EventKind::Text => encode_text(&event.payload),
            EventKind::Binary => encode_binary(&event.payload),
            EventKind::Close => encode_close_frame(),
        };
        let mut clients = clients.lock().await;
        println!("broadcasting {:?} to {:?}", event.kind, event.address);
        if let EventAddress::All = event.address {
            println!("clients connected: {}", clients.len());
        }
        broadcast_buffer(&mut *clients, event.address, &buffer).await?;
    }

    println!("Stopping writing messages");

    Ok(())
}

pub async fn send_directly<T: AsyncWrite + Unpin>(
    writer: &mut T,
    id: u128,
    kind: EventKind,
    data: &[u8],
) -> Result<()> {
    let buffer = match kind {
        EventKind::Text => encode_text(data),
        EventKind::Binary => encode_binary(data),
        EventKind::Close => encode_close_frame(),
    };
    println!("sending direct message {:?} to {}", kind, id);
    writer.write_all(&buffer).await?;
    Ok(())
}
