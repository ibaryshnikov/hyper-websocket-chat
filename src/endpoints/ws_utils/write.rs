use tokio::io::AsyncWriteExt;
//use tokio::io::AsyncWrite;

use super::{encode_length, FrameAddress};
use crate::shared_types::Result;
use crate::ClientsMap;

use super::consts::*;

pub fn encode_close_frame() -> Vec<u8>{
    [OPCODE_CLOSE | FIN_BIT_MASK].to_vec()
}
//pub async fn send_close_frame<T: AsyncWrite + Unpin>(writer: &mut T) -> Result<()> {
//    let msg = [OPCODE_CLOSE | FIN_BIT_MASK];
//    writer.write_all(&msg).await?;
//    Ok(())
//}

//async fn send_single_frame_text<T: AsyncWrite + Unpin>(writer: &mut T, msg: &[u8]) -> Result<()> {
//    let first_byte: &[u8] = &[OPCODE_TEXT | FIN_BIT_MASK];
//    let length = encode_length(msg.len());
//    let payload = [first_byte, &length, msg].concat();
//    writer.write_all(&payload).await?;
//    Ok(())
//}

pub fn encode_text(msg: &[u8]) -> Vec<u8> {
    let first_byte: &[u8] = &[OPCODE_TEXT | FIN_BIT_MASK];
    let length = encode_length(msg.len());
    [first_byte, &length, msg].concat()
}
//pub async fn send_text<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
//    send_single_frame_text(upgraded, msg).await
//}

//pub async fn send_single_frame_binary<T: AsyncWrite + Unpin>(
//    writer: &mut T,
//    msg: &[u8],
//) -> Result<()> {
//    let first_byte: &[u8] = &[OPCODE_BINARY | FIN_BIT_MASK];
//    let length = encode_length(msg.len());
//    let payload = [first_byte, &length, msg].concat();
//    writer.write_all(&payload).await?;
//    Ok(())
//}

pub fn encode_binary(msg: &[u8]) -> Vec<u8> {
    let first_byte: &[u8] = &[OPCODE_BINARY | FIN_BIT_MASK];
    let length = encode_length(msg.len());
    [first_byte, &length, msg].concat()
}
//pub async fn send_binary<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
//    send_single_frame_binary(upgraded, msg).await
//}

pub async fn broadcast_buffer(clients: &mut ClientsMap, address: FrameAddress, buffer: &[u8]) -> Result<()> {
    match address {
        FrameAddress::All => {
            for writer in clients.values_mut() {
                writer.write_all(buffer).await?;
            }
        }
        FrameAddress::Client(id) => {
            if let Some(writer) = clients.get_mut(&id) {
                writer.write_all(buffer).await?;
            } else {
                println!("Can't send message, client {} not found", id);
            }
        }
        FrameAddress::ClientRange(list) => {
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
