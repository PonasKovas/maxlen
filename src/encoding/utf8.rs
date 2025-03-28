use super::Encoding;

/// The standard UTF-8 encoding used natively in Rust.
#[derive(Debug, Default, Hash)]
pub struct Utf8;
impl Encoding for Utf8 {
	fn length(s: &str) -> usize {
		s.len()
	}
}
