pub const FIN_MASK: u8 = 0b1000_0000;
pub const RSV1_MASK: u8 = 0b0100_0000;
pub const RSV2_MASK: u8 = 0b0010_0000;
pub const RSV3_MASK: u8 = 0b0001_0000;
pub const OPCODE_MASK: u8 = 0b0000_1111;
pub const LENGTH_MASK: u8 = 0b0111_1111;
pub const MASKED_MASK: u8 = 0b1000_0000;

pub const fn is_fin(byte: u8) -> bool {
    (byte & FIN_MASK) == FIN_MASK
}
pub const fn is_rsv1(byte: u8) -> bool {
    (byte & RSV1_MASK) == RSV1_MASK
}
pub const fn is_rsv2(byte: u8) -> bool {
    (byte & RSV2_MASK) == RSV2_MASK
}
pub const fn is_rsv3(byte: u8) -> bool {
    (byte & RSV3_MASK) == RSV3_MASK
}
pub const fn is_mask(byte: u8) -> bool {
    (byte & MASKED_MASK) == MASKED_MASK
}
