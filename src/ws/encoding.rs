const LENGTH_U16: &[u8] = &[126];
const LENGTH_U64: &[u8] = &[127];

pub fn encode_length(length: usize) -> Vec<u8> {
    if length <= 125 {
        // the first byte is the length
        vec![length as u8]
    } else if length <= 65535 {
        // the first byte is 126, read the next 2 bytes as u16 for a length
        [LENGTH_U16, &(length as u16).to_be_bytes()].concat()
    } else {
        // the first byte is 127, read the next 8 bytes as u64 for a length
        [LENGTH_U64, &(length as u64).to_be_bytes()].concat()
    }
}
