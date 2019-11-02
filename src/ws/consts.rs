pub const OPCODE_CONTINUATION: u8 = 0x0;
pub const OPCODE_TEXT: u8 = 0x1;
pub const OPCODE_BINARY: u8 = 0x2;
pub const OPCODE_CLOSE: u8 = 0x8;
pub const OPCODE_PING: u8 = 0x9;
pub const OPCODE_PONG: u8 = 0xA;

pub const FIN_BIT_MASK: u8 = 0b1000_0000;
pub const OPCODE_MASK: u8 = 0b0000_1111;
pub const LENGTH_MASK: u8 = 0b0111_1111;
pub const MASKED_MASK: u8 = 0b1000_0000;
