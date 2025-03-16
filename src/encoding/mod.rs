mod cesu8;
mod mcesu8;
mod utf8;

/// Trait for string encoding types. Defines a certain string's length in that encoding.
///
/// ## Soundness
///
/// Only encodings for which the following is true can implement this trait soundly:
/// - Removing a character or taking a subslice of the string will **never** make the representation longer in that encoding.
/// - Converting an ASCII character from lowercase to uppercase and vice versa will **never** change the length of the string in that encoding.
///
pub trait Encoding {
	fn length(s: &str) -> usize;
}

pub use cesu8::Cesu8;
pub use mcesu8::MCesu8;
pub use utf8::Utf8;
