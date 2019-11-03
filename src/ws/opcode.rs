use super::consts::OPCODE_MASK;

#[derive(Debug)]
pub enum Opcode {
    Continuation,
    Text,
    Binary,
    ReservedNonControl3,
    ReservedNonControl4,
    ReservedNonControl5,
    ReservedNonControl6,
    ReservedNonControl7,
    Close,
    Ping,
    Pong,
    ReservedControlB,
    ReservedControlC,
    ReservedControlD,
    ReservedControlE,
    ReservedControlF,
}
impl Opcode {
    // create a new opcode from unchanged input byte
    pub fn decode(byte: u8) -> Self {
        use Opcode::*;
        match byte & OPCODE_MASK {
            0x0 => Continuation,
            0x1 => Text,
            0x2 => Binary,
            0x3 => ReservedNonControl3,
            0x4 => ReservedNonControl4,
            0x5 => ReservedNonControl5,
            0x6 => ReservedNonControl6,
            0x7 => ReservedNonControl7,
            0x8 => Close,
            0x9 => Ping,
            0xA => Pong,
            0xB => ReservedControlB,
            0xC => ReservedControlC,
            0xD => ReservedControlD,
            0xE => ReservedControlE,
            0xF => ReservedControlF,
            // as opcode is 4 bit, this should never panic
            value => panic!("Unexpected opcode value {:#X}", value),
        }
    }
    pub fn encode(&self) -> u8 {
        use Opcode::*;
        match self {
            Continuation => 0x0,
            Text => 0x1,
            Binary => 0x2,
            ReservedNonControl3 => 0x3,
            ReservedNonControl4 => 0x4,
            ReservedNonControl5 => 0x5,
            ReservedNonControl6 => 0x6,
            ReservedNonControl7 => 0x7,
            Close => 0x8,
            Ping => 0x9,
            Pong => 0xA,
            ReservedControlB => 0xB,
            ReservedControlC => 0xC,
            ReservedControlD => 0xD,
            ReservedControlE => 0xE,
            ReservedControlF => 0xF,
        }
    }
}
