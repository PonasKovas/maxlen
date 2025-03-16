#![doc = include_str!("../README.md")]

mod bslice;
mod bstr;
mod bstring;
mod bvec;
mod error;

/// For macro usage
#[doc(hidden)]
pub mod const_checks;
/// Different string encodings
pub mod encoding;

pub use bslice::BSlice;
pub use bstr::BStr;
pub use bstring::BString;
pub use bvec::BVec;
pub use error::LengthExceeded;

/// Creates a static `&'static BStr<MAX, E>` with a compile-time check.
///
/// ```
/// # use maxlen::{bstr, BStr, encoding::Cesu8};
/// let _: &BStr<255> = bstr!(255, "test string");
/// let _: &BStr<255, Cesu8> = bstr!(255, Cesu8, "255 bytes in cesu-8 encoding!");
///
/// // let _: &BStr<1> = bstr!(1, "longer than 1 char"); // will not compile
/// ```
pub use maxlen_macro::bstr;
