use super::Encoding;

/// The [CESU-8](https://en.wikipedia.org/wiki/CESU-8) encoding
pub struct Cesu8;
impl Encoding for Cesu8 {
	fn length(s: &str) -> usize {
		let mut extra = 0;
		for c in s.chars() {
			if c > '\u{FFFF}' {
				extra += 2; // each 4-byte UTF-8 sequence (BMP > U+FFFF) becomes 6 bytes in CESU-8 (2 extra bytes per character).
			}
		}

		s.len() + extra
	}
}
