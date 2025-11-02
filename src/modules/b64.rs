
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
    encode_to_bytes(bytes)
        .into_iter()
        .flat_map(|n| n.to_be_bytes())
        .map(|b| BYTES[b as usize] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine as _};

    fn assert_base64(input: &str) {
        let expected = general_purpose::STANDARD.encode(input.as_bytes());
        let actual = encode_to_string(input.as_bytes().to_vec());
        assert_eq!(actual, expected, "Base64 mismatch for input: {:?}", input);
    }

    #[test]
    fn test_empty() {
        assert_base64("");
    }

    #[test]
    fn test_ascii() {
        assert_base64("foo");
        assert_base64("foobar");
        assert_base64("hello world");
        assert_base64("Base64!");
    }

    #[test]
    fn test_binary_data() {
        let input = [
            0x00, 0xFF, 0xAB, 0x20, 0x7F, 0x10,
            0x99, 0x42, 0x00, 0x01,
        ];
        let expected = general_purpose::STANDARD.encode(&input);
        let actual = encode_to_string(input.to_vec());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_unicode_text() {
        // UTF-8 input — Base64 encodes raw bytes, not characters
        assert_base64("こんにちは");
        assert_base64("🚀🔥");
    }

    #[test]
    fn test_explain_encoding_3_bytes() {
        // Input: "foo" = [0x66, 0x6F, 0x6F]
        // 0x66 = 0b01100110
        // 0x6F = 0b01101111
        // 0x6F = 0b01101111
        let input = vec![0x66, 0x6F, 0x6F];

        let encoded_u32s = encode_to_bytes(input);

        // Should produce one u32 (no padding needed for 3 bytes)
        assert_eq!(encoded_u32s.len(), 1);

        let u32_val = encoded_u32s[0];
        let bytes = u32_val.to_be_bytes();

        // Print to see what we got
        println!("u32: 0x{:08X}", u32_val);
        println!("Byte 0: 0x{:02X} = {} (binary: {:08b})", bytes[0], bytes[0], bytes[0]);
        println!("Byte 1: 0x{:02X} = {} (binary: {:08b})", bytes[1], bytes[1], bytes[1]);
        println!("Byte 2: 0x{:02X} = {} (binary: {:08b})", bytes[2], bytes[2], bytes[2]);
        println!("Byte 3: 0x{:02X} = {} (binary: {:08b})", bytes[3], bytes[3], bytes[3]);

        // Verify top 2 bits are always 0 (value <= 63, no padding in this case)
        assert!(bytes[0] < 64, "Byte 0 should be 0-63, got {}", bytes[0]);
        assert!(bytes[1] < 64, "Byte 1 should be 0-63, got {}", bytes[1]);
        assert!(bytes[2] < 64, "Byte 2 should be 0-63, got {}", bytes[2]);
        assert!(bytes[3] < 64, "Byte 3 should be 0-63, got {}", bytes[3]);

        // The actual Base64 encoding of "foo" is "Zm9v"
        // Z = index 25, m = index 38, 9 = index 61, v = index 47
        assert_eq!(bytes[0], 25); // 'Z'
        assert_eq!(bytes[1], 38); // 'm'
        assert_eq!(bytes[2], 61); // '9'
        assert_eq!(bytes[3], 47); // 'v'

        let result = encode_to_string(vec![0x66, 0x6F, 0x6F]);
        assert_eq!(result, "Zm9v");
    }
}
