// Wicci Shim Module
// Manage [u8] latin1 text

use std::ascii::AsciiExt;

#[cfg(feature = "never")]
pub fn make_lower<T: AsciiExt>(bytes: &mut T) {
  bytes.make_ascii_lowercase();
}

pub fn make_lower_vec_u8(bytes: &mut Vec<u8>) {
  for i in 0 .. bytes.len() {
    bytes[i] = bytes[i].to_ascii_lowercase();
  }
}

// tried to make this generic over numeric types but
// in Rust 1.0 ... 1.1 this is now hard!
pub fn digits_to_usize(digits: &Vec<u8>)-> Option<usize> {
  let mut val: usize = 0;
  for d in digits {
    if *d < b'0' || *d > b'9' { return None; }
    val = val * 10 + (*d - b'0') as usize;
  }
  Some(val)
}
