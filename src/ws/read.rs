use tokio::io::{AsyncRead, AsyncReadExt};

use super::frame::PayloadLength;
use crate::shared::types::Result;

pub async fn read_length<T: AsyncRead + Unpin>(
    kind: PayloadLength,
    reader: &mut T,
) -> Result<usize> {
    use PayloadLength::*;
    let length = match kind {
        U16 => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).await?;
            u16::from_be_bytes(buf) as usize
        }
        U64 => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).await?;
            u64::from_be_bytes(buf) as usize
        }
    };
    Ok(length)
}

pub async fn read_mask<T: AsyncRead + Unpin>(reader: &mut T) -> Result<[u8; 4]> {
    let mut mask_buf = [0u8; 4];
    reader.read_exact(&mut mask_buf).await?;
    Ok(mask_buf)
}