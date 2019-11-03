use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::consts::LENGTH_MASK;
use super::frame::{Frame, Headers};
use super::opcode::Opcode;

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

pub async fn read_frame<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Frame> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).await?;
    let headers = Headers::decode(buf[0]);
    let opcode = Opcode::decode(buf[0]);

    let length = match buf[1] & LENGTH_MASK {
        value @ 0..=125 => value as usize,
        126 => read_length_u16(reader).await?,
        127 => read_length_u64(reader).await?,
        // as length is 7 bit, this should never panic
        value => panic!("Unexpected length value {:#X}", value),
    };
    let maybe_mask = if headers.mask {
        Some(read_mask(reader).await?)
    } else {
        None
    };

    // reading payload data
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload).await?;

    if let Some(mask) = maybe_mask {
        // unmasking the message
        for i in 0..payload.len() {
            payload[i] ^= mask[i % 4];
        }
    }

    Ok(Frame {
        headers,
        opcode,
        payload,
    })
}
