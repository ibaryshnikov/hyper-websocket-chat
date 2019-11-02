use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::encode_length;
use crate::shared_types::Result;

pub async fn send_close_frame<T: AsyncWrite + Unpin>(writer: &mut T) -> Result<()> {
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

pub async fn send_text<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
    send_single_frame_text(upgraded, msg).await
}

pub async fn send_single_frame_binary<T: AsyncWrite + Unpin>(
    writer: &mut T,
    msg: &[u8],
) -> Result<()> {
    let first_byte: &[u8] = &[0x82];
    let length = encode_length(msg.len());
    let payload = [first_byte, &length, msg].concat();
    writer.write_all(&payload).await?;
    Ok(())
}

pub async fn send_binary<T: AsyncWrite + Unpin>(upgraded: &mut T, msg: &[u8]) -> Result<()> {
    send_single_frame_binary(upgraded, msg).await
}
