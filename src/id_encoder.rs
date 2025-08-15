const BASE62_CHARS: &[u8] = b"123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub fn encode_id(id: u64) -> String {
    if id == 0 {
        return "1".to_string();
    }
    
    let mut id = id;
    let mut result = Vec::new();
    
    while id > 0 {
        let remainder = (id % 62) as usize;
        result.push(BASE62_CHARS[remainder]);
        id /= 62;
    }
    
    result.reverse();
    String::from_utf8(result).unwrap()
}

pub fn decode_id(encoded: &str) -> Result<u64, &'static str> {
    if encoded.is_empty() {
        return Err("Empty string");
    }
    
    let mut result = 0u64;
    
    for ch in encoded.chars() {
        let value = match ch {
            '1'..='9' => (ch as u8 - b'1') as u64,
            'A'..='Z' => (ch as u8 - b'A' + 9) as u64,
            'a'..='z' => (ch as u8 - b'a' + 35) as u64,
            _ => return Err("Invalid character"),
        };
        result = result
            .checked_mul(62)
            .and_then(|r| r.checked_add(value))
            .ok_or("Overflow")?;
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let test_cases = vec![1, 10, 100, 1000, 10000, 100000, 1000000, u64::MAX / 1000];
        
        for id in test_cases {
            let encoded = encode_id(id);
            let decoded = decode_id(&encoded).unwrap();
            assert_eq!(id, decoded);
        }
    }

    #[test]
    fn test_encode_zero() {
        assert_eq!(encode_id(0), "1");
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_id("").is_err());
        assert!(decode_id("0").is_err());
        assert!(decode_id("!").is_err());
    }
}