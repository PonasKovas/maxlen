use thiserror::Error;

/// Length exceeded error.
#[derive(Error, Debug)]
#[error("length of {length} exceeded ({maximum})")]
pub struct LengthExceeded {
	pub length: usize,
	pub maximum: usize,
}
