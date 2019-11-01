use crypto::digest::Digest;
use crypto::sha1::Sha1;

const WS_MAGIC_CONST: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn sha1(msg: &[u8]) -> [u8; 20] {
  let mut hasher = Sha1::new();
  hasher.input(&msg);
  let mut buffer = [0; 20];
  hasher.result(&mut buffer);
  buffer
}

pub fn generate_key_from(input: &[u8]) -> String {
  let concatenated = [
    input,
    WS_MAGIC_CONST,
  ].concat();
  let hash = sha1(&concatenated);
  base64::encode(&hash)
}

pub fn encode_length(length: usize) -> Vec<u8> {
  if length <= 125 {
    // the first byte is the length
    vec![length as u8]
  } else if length <= 65535 {
    // the first byte is 126, read the next 2 bytes as u16 for a length
    [&[126][..], &(length as u16).to_be_bytes()].concat()
  } else {
    // the first byte is 127, read the next 8 bytes as u64 for a length
    [&[127][..], &(length as u64).to_be_bytes()].concat()
  }
}
