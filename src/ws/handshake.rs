use sha1::{Digest, Sha1};

const WS_MAGIC_CONST: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn sha1(msg: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.input(&msg);
    hasher.result().into()
}

pub fn generate_key_from(input: &[u8]) -> String {
    let concatenated = [input, WS_MAGIC_CONST].concat();
    let hash = sha1(&concatenated);
    base64::encode(&hash)
}
