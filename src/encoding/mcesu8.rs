use super::Encoding;

/// The [Modified CESU-8](https://en.wikipedia.org/wiki/CESU-8) encoding (same as CESU-8 but encodes `00` as `C0 80`)
pub struct MCesu8;
impl Encoding for MCesu8 {
	fn length(s: &str) -> usize {
		let mut extra = 0;
		for c in s.chars() {
			if c == '\u{0}' {
				extra += 1; // NUL is represented as \xC0 \x80
			}
			if c > '\u{FFFF}' {
				extra += 2; // each 4-byte UTF-8 sequence (BMP > U+FFFF) becomes 6 bytes in CESU-8 (2 extra bytes per character).
			}
		}

		s.len() + extra
	}
}
