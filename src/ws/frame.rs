use super::consts::*;
use super::opcode::Opcode;

#[derive(Debug)]
pub struct Headers {
    pub fin: bool,
    pub rsv1: bool,
    pub rsv2: bool,
    pub rsv3: bool,
    pub mask: bool,
}
impl Headers {
    pub fn decode(byte: u8) -> Self {
        Headers {
            fin: is_fin(byte),
            rsv1: is_rsv1(byte),
            rsv2: is_rsv2(byte),
            rsv3: is_rsv3(byte),
            mask: is_mask(byte),
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub headers: Headers,
    pub opcode: Opcode,
    pub payload: Vec<u8>,
}
