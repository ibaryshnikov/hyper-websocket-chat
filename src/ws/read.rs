use tokio::io::{AsyncRead, AsyncReadExt};

use anyhow::Result;

pub async fn read_length_u16<T: AsyncRead + Unpin>(reader: &mut T) -> Result<usize> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).await?;
    Ok(u16::from_be_bytes(buf) as usize)
}
pub async fn read_length_u64<T: AsyncRead + Unpin>(reader: &mut T) -> Result<usize> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf).await?;
    Ok(u64::from_be_bytes(buf) as usize)
}

pub async fn read_mask<T: AsyncRead + Unpin>(reader: &mut T) -> Result<[u8; 4]> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).await?;
    Ok(buf)
}
