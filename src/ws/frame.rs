#[derive(Debug)]
pub enum FrameAddress {
    All,
    Client(u128),
    ClientRange(Vec<u128>),
}

#[derive(Debug)]
pub enum FrameKind {
    Text,
    Binary,
    Close,
}

#[derive(Debug)]
pub enum FrameSource {
    System,
    Client(u128),
}

pub struct Frame {
    pub kind: FrameKind,
    pub address: FrameAddress,
    pub data: Vec<u8>,
}

pub enum PayloadLength {
    U16,
    U64,
}
