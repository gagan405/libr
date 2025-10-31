
const BYTES: &[u8; 65] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
const PADDING_IDX: u32 = 64;

pub fn encode_to_bytes(bytes: Vec<u8>) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();
    for chunk in bytes.chunks(3) {
        let chunk_size = chunk.len();
        if chunk_size == 3 {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4)) as u32) << 16
                | (((chunk[1] & 0x0F) << 2 | ((chunk[2] & 0xC0) >> 6)) as u32) << 8
                | ((chunk[2] << 2) >> 2) as u32;
            result.push(val);
        } else if chunk_size == 2 {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4)) as u32) << 16
                | (((chunk[1] & 0x0F) << 2) as u32) << 8
                | PADDING_IDX;
            result.push(val);
        } else {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4) as u32) << 16
                | PADDING_IDX << 8
                | PADDING_IDX;
            result.push(val);
        }
    }
    result
}

pub fn encode_to_string(bytes: Vec<u8>) -> String {
    let indices: Vec<u32> = encode_to_bytes(bytes);
    indices
        .into_iter()
        .flat_map(|n| n.to_be_bytes())
        .map(|b| BYTES[b as usize] as char)
        .collect()
}