#[derive(Debug)]
pub enum EventAddress {
    All,
    Client(u128),
    ClientRange(Vec<u128>),
}

#[derive(Debug)]
pub enum EventKind {
    Text,
    Binary,
    Close,
}

#[derive(Debug)]
pub enum EventSource {
    System,
    Client(u128),
}

#[derive(Debug)]
pub struct Event {
    pub kind: EventKind,
    pub address: EventAddress,
    pub payload: Vec<u8>,
}
