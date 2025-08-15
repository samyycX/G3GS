const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

// 方法1：编码时减1，解码时加1（推荐）
pub fn encode_id(id: u64) -> String {
  if id == 0 {
      return "".to_string();
  }
  
  let mut adjusted_id = id - 1;
  let mut result = Vec::new();
  
  if adjusted_id == 0 {
      return "1".to_string();
  }
  
  while adjusted_id > 0 {
      let remainder = (adjusted_id % 58) as usize;
      result.push(BASE58_CHARS[remainder]);
      adjusted_id /= 58;
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
          'A'..='H' => (ch as u8 - b'A' + 9) as u64,
          'J'..='N' => (ch as u8 - b'J' + 17) as u64,
          'P'..='Z' => (ch as u8 - b'P' + 22) as u64,
          'a'..='k' => (ch as u8 - b'a' + 33) as u64,
          'm'..='z' => (ch as u8 - b'm' + 44) as u64,
          _ => return Err("Invalid character"),
      };
      result = result
          .checked_mul(58)
          .and_then(|r| r.checked_add(value))
          .ok_or("Overflow")?;
  }
  
  Ok(result + 1)
}