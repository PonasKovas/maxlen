use crate::{
	BSlice, BStr, LengthExceeded, const_checks,
	encoding::{Encoding, Utf8},
};
use std::{
	borrow::{Borrow, Cow},
	ffi::{OsStr, OsString},
	fmt::Display,
	marker::PhantomData,
	net::ToSocketAddrs,
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
	rc::Rc,
	str::FromStr,
	sync::Arc,
};

/// Bounded [`String`].
///
/// Guaranteed to not be longer than `MAX` bytes in the [`E`][crate::encoding::Encoding] encoding representation.
#[derive(Debug, Default, Hash)]
pub struct BString<const MAX: usize, E = Utf8> {
	s: String,
	phantom: PhantomData<fn(E) -> E>,
}

impl<E: Encoding, const MAX: usize> BString<MAX, E> {
	/// Creates a `BString<MAX, E>` from a `String` without any checks.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the string is definitely
	/// not longer than `MAX` bytes in the given encoding.
	pub const unsafe fn from_string_unchecked(s: String) -> Self {
		Self {
			s,
			phantom: PhantomData,
		}
	}
	/// Creates a `BString<MAX, E>` from a `&str` without any checks, allocating a new buffer.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the string is definitely
	/// not longer than `MAX` bytes in the given encoding.
	pub unsafe fn from_str_unchecked(s: &str) -> Self {
		Self {
			s: s.to_owned(),
			phantom: PhantomData,
		}
	}
	/// Creates a `BString<MAX, E>` from a `&str`, performing a runtime check and allocating a new buffer.
	pub fn from_str(s: &str) -> Result<Self, LengthExceeded> {
		BStr::from_str(s).map(|s| s.to_owned())
	}
	/// Creates a `BString<MAX, E>` from a `String`, performing a runtime check.
	pub fn from_string(s: String) -> Result<Self, LengthExceeded> {
		BStr::<MAX, E>::from_str(&s)?;

		Ok(unsafe { Self::from_string_unchecked(s) })
	}
	/// Gives the inner String.
	pub fn into_inner(self) -> String {
		self.s
	}
	/// Gives an immutable reference to the inner String.
	pub const fn as_string(&self) -> &String {
		&self.s
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub fn relax_max<const MAX2: usize>(self) -> BString<MAX2, E> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BString::from_string_unchecked(self.s) }
	}
	/// Changes the `MAX` bound (and optionally the encoding type).
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max<E2: Encoding, const MAX2: usize>(
		self,
	) -> Result<BString<MAX2, E2>, LengthExceeded> {
		BString::from_string(self.s)
	}
	/// Returns this String’s capacity, in bytes.
	///
	/// See [`String::capacity`] for more information.
	pub fn capacity(&self) -> usize {
		self.s.capacity()
	}
	/// Converts a `BString` into a byte vector.
	///
	/// See [`String::into_bytes`] for more information.
	// pub fn into_bytes(self) -> BVec<MAX, u8> {
	// 	TODO
	// }
	/// Consumes and leaks the String, returning a mutable reference to the contents, &'a mut str.
	///
	/// See [`String::leak`] for more information.
	pub fn leak<'a>(self) -> &'a mut BStr<MAX, E> {
		unsafe { BStr::from_str_mut_unchecked(self.s.leak()) }
	}
	/// Truncates this [`BString`], removing all contents.
	///
	/// See [`String::clear`] for more information.
	pub fn clear(&mut self) {
		self.s.clear();
	}
	/// Removes the specified range from the string in bulk, returning all removed characters as an iterator.
	///
	/// See [`String::drain`] for more information.
	pub fn drain<R>(&mut self, range: R)
	where
		R: std::ops::RangeBounds<usize>,
	{
		self.s.drain(range);
	}
	/// Creates a new empty [`BString`].
	///
	/// See [`String::new`] for more information.
	pub fn new() -> Self {
		Self {
			s: String::new(),
			phantom: PhantomData,
		}
	}
	/// Creates a new empty String with at least the specified capacity.
	///
	/// See [`String::with_capacity`] for more information.
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			s: String::with_capacity(capacity),
			phantom: PhantomData,
		}
	}
	/// Removes the last character from the string buffer and returns it.
	///
	/// See [`String::pop`] for more information.
	pub fn pop(&mut self) -> Option<char> {
		self.s.pop()
	}
	/// Removes a [`char`] from this [`BString`] at a byte position and returns it.
	///
	/// See [`String::remove`] for more information.
	pub fn remove(&mut self, idx: usize) -> char {
		self.s.remove(idx)
	}
	/// Retains only the characters specified by the predicate.
	///
	/// See [`String::retain`] for more information.
	pub fn retain<F>(&mut self, f: F)
	where
		F: FnMut(char) -> bool,
	{
		self.s.retain(f);
	}
	/// Shrinks the capacity of this [`BString`] with a lower bound.
	///
	/// See [`String::shrink_to`] for more information.
	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.s.shrink_to(min_capacity)
	}
	/// Shrinks the capacity of this [`BString`] to match its length.
	///
	/// See [`String::shrink_to_fit`] for more information.
	pub fn shrink_to_fit(&mut self) {
		self.s.shrink_to_fit()
	}
	/// Splits the string into two at the given byte index.
	///
	/// See [`String::split_off`] for more information.
	pub fn split_off(&mut self, at: usize) -> Self {
		Self {
			s: self.s.split_off(at),
			phantom: PhantomData,
		}
	}
	/// Shortens this [`BString`] to the specified length.
	///
	/// See [`String::truncate`] for more information.
	pub fn truncate(&mut self, new_len: usize) {
		self.s.truncate(new_len);
	}
}

// Trait implementations relating BStr and BString
//////////////////////////////////////////////////

impl<E: Encoding, const MAX: usize> Deref for BString<MAX, E> {
	type Target = BStr<MAX, E>;

	fn deref(&self) -> &Self::Target {
		unsafe { BStr::from_str_unchecked(&self.s) }
	}
}
impl<E: Encoding, const MAX: usize> DerefMut for BString<MAX, E> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { BStr::from_str_mut_unchecked(&mut self.s) }
	}
}
impl<E: Encoding, const MAX: usize> Borrow<BStr<MAX, E>> for BString<MAX, E> {
	fn borrow(&self) -> &BStr<MAX, E> {
		&**self
	}
}
impl<E: Encoding, const MAX: usize> AsRef<BStr<MAX, E>> for BString<MAX, E> {
	fn as_ref(&self) -> &BStr<MAX, E> {
		self
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Box<BStr<MAX, E>> {
	fn from(value: BString<MAX, E>) -> Self {
		let b = Box::<str>::from(value.s);

		unsafe { Box::from_raw(Box::into_raw(b) as *mut BStr<MAX, E>) }
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<BStr<MAX2, E2>>
	for BString<MAX1, E1>
{
	fn eq(&self, other: &BStr<MAX2, E2>) -> bool {
		(**self).eq(&**other)
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<&BStr<MAX2, E2>>
	for BString<MAX1, E1>
{
	fn eq(&self, other: &&BStr<MAX2, E2>) -> bool {
		(**self).eq(&**other)
	}
}

// Trait implementations mirroring standard String
//////////////////////////////////////////////////

impl<E: Encoding, const MAX: usize> Clone for BString<MAX, E> {
	fn clone(&self) -> Self {
		Self {
			s: self.s.clone(),
			phantom: PhantomData,
		}
	}
}
impl<E: Encoding, const MAX: usize> AsRef<OsStr> for BString<MAX, E> {
	fn as_ref(&self) -> &OsStr {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<Path> for BString<MAX, E> {
	fn as_ref(&self) -> &Path {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<[u8]> for BString<MAX, E> {
	fn as_ref(&self) -> &[u8] {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<BSlice<u8, MAX>> for BString<MAX, E> {
	fn as_ref(&self) -> &BSlice<u8, MAX> {
		unsafe { BSlice::from_slice_unchecked(self.as_bytes()) }
	}
}
impl<E: Encoding, const MAX: usize> AsRef<Self> for BString<MAX, E> {
	fn as_ref(&self) -> &Self {
		self
	}
}
impl<E: Encoding, const MAX: usize> AsRef<str> for BString<MAX, E> {
	fn as_ref(&self) -> &str {
		&**self
	}
}
impl<E: Encoding, const MAX: usize> Borrow<str> for BString<MAX, E> {
	fn borrow(&self) -> &str {
		&**self
	}
}
impl<E: Encoding, const MAX: usize> Display for BString<MAX, E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		(**self).fmt(f)
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<BString<MAX2, E2>>
	for BString<MAX1, E1>
{
	fn eq(&self, other: &BString<MAX2, E2>) -> bool {
		(**self).eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Eq for BString<MAX, E> {}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialOrd<BString<MAX2, E2>>
	for BString<MAX1, E1>
{
	fn partial_cmp(&self, other: &BString<MAX2, E2>) -> Option<std::cmp::Ordering> {
		(**self).partial_cmp(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Ord for BString<MAX, E> {
	fn cmp(&self, other: &BString<MAX, E>) -> std::cmp::Ordering {
		(**self).cmp(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Extend<BString<MAX, E>> for String {
	fn extend<T: IntoIterator<Item = BString<MAX, E>>>(&mut self, iter: T) {
		for i in iter {
			self.push_str(&i);
		}
	}
}
impl<'a, E: Encoding, const MAX: usize> From<BString<MAX, E>> for Cow<'a, BStr<MAX, E>> {
	fn from(value: BString<MAX, E>) -> Self {
		Self::Owned(value)
	}
}
impl<'a, E: Encoding, const MAX: usize> From<BString<MAX, E>> for Cow<'a, str> {
	fn from(value: BString<MAX, E>) -> Self {
		Self::Owned(value.into_inner())
	}
}
impl<'a, E: Encoding, const MAX: usize> From<&'a BString<MAX, E>> for Cow<'a, BStr<MAX, E>> {
	fn from(value: &'a BString<MAX, E>) -> Self {
		Self::Borrowed(value)
	}
}
impl<'a, E: Encoding, const MAX: usize> From<&'a BString<MAX, E>> for Cow<'a, str> {
	fn from(value: &'a BString<MAX, E>) -> Self {
		Self::Borrowed(value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BString<MAX, E>> for BString<MAX, E> {
	fn from(value: &BString<MAX, E>) -> Self {
		value.clone()
	}
}
impl<E: Encoding, const MAX: usize> From<&BString<MAX, E>> for String {
	fn from(value: &BString<MAX, E>) -> Self {
		value.s.clone()
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Arc<BStr<MAX, E>> {
	fn from(value: BString<MAX, E>) -> Self {
		let arc = Arc::<str>::from(value.into_inner());

		unsafe { Arc::from_raw(Arc::into_raw(arc) as *const BStr<MAX, E>) }
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Arc<str> {
	fn from(value: BString<MAX, E>) -> Self {
		Arc::<str>::from(value.into_inner())
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Box<dyn std::error::Error> {
	fn from(value: BString<MAX, E>) -> Self {
		Self::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>>
	for Box<dyn std::error::Error + Send + Sync>
{
	fn from(value: BString<MAX, E>) -> Self {
		Self::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for OsString {
	fn from(value: BString<MAX, E>) -> Self {
		Self::from(value.into_inner())
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for PathBuf {
	fn from(value: BString<MAX, E>) -> Self {
		Self::from(value.into_inner())
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Rc<BStr<MAX, E>> {
	fn from(value: BString<MAX, E>) -> Self {
		let arc = Rc::<str>::from(value.into_inner());

		unsafe { Rc::from_raw(Rc::into_raw(arc) as *const BStr<MAX, E>) }
	}
}
impl<E: Encoding, const MAX: usize> From<BString<MAX, E>> for Rc<str> {
	fn from(value: BString<MAX, E>) -> Self {
		Self::from(value.into_inner())
	}
}
impl<E: Encoding, const MAX: usize> FromStr for BString<MAX, E> {
	type Err = LengthExceeded;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::from_str(s)
	}
}
impl<E: Encoding, const MAX: usize> ToSocketAddrs for BString<MAX, E> {
	type Iter = <String as ToSocketAddrs>::Iter;

	fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
		self.s.to_socket_addrs()
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BString<MAX, E>> for Cow<'_, str> {
	fn eq(&self, other: &BString<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BString<MAX, E>> for Cow<'_, str> {
	fn eq(&self, other: &&BString<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<Cow<'_, str>> for BString<MAX, E> {
	fn eq(&self, other: &Cow<'_, str>) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<Cow<'_, str>> for &BString<MAX, E> {
	fn eq(&self, other: &Cow<'_, str>) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BString<MAX, E>> for OsStr {
	fn eq(&self, other: &BString<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BString<MAX, E>> for OsStr {
	fn eq(&self, other: &&BString<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsStr> for BString<MAX, E> {
	fn eq(&self, other: &OsStr) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsStr> for &BString<MAX, E> {
	fn eq(&self, other: &OsStr) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BString<MAX, E>> for OsString {
	fn eq(&self, other: &BString<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BString<MAX, E>> for OsString {
	fn eq(&self, other: &&BString<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsString> for BString<MAX, E> {
	fn eq(&self, other: &OsString) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsString> for &BString<MAX, E> {
	fn eq(&self, other: &OsString) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BString<MAX, E>> for str {
	fn eq(&self, other: &BString<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BString<MAX, E>> for str {
	fn eq(&self, other: &&BString<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BString<MAX, E>> for &str {
	fn eq(&self, other: &BString<MAX, E>) -> bool {
		self.eq(&&***other) // tf??
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<str> for BString<MAX, E> {
	fn eq(&self, other: &str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<str> for &BString<MAX, E> {
	fn eq(&self, other: &str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&str> for BString<MAX, E> {
	fn eq(&self, other: &&str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&mut str> for BString<MAX, E> {
	fn eq(&self, other: &&mut str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> TryFrom<String> for BString<MAX, E> {
	type Error = LengthExceeded;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		BString::from_string(value)
	}
}
impl<E: Encoding, const MAX: usize> TryFrom<&str> for BString<MAX, E> {
	type Error = LengthExceeded;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		BString::from_str(value)
	}
}
impl<E: Encoding, const MAX: usize> TryFrom<&mut str> for BString<MAX, E> {
	type Error = LengthExceeded;

	fn try_from(value: &mut str) -> Result<Self, Self::Error> {
		BString::from_str(value)
	}
}

#[cfg(feature = "serde")]
mod serde_impls {
	use super::*;
	use serde::{Deserialize, Serialize, de::Visitor};

	impl<E: Encoding, const MAX: usize> Serialize for BString<MAX, E> {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			serializer.serialize_str(self)
		}
	}

	struct BStringVisitor<E: Encoding, const MAX: usize>(PhantomData<fn(E) -> E>);
	impl<'de, E: Encoding, const MAX: usize> Visitor<'de> for BStringVisitor<E, MAX> {
		type Value = BString<MAX, E>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a string")
		}
		fn visit_str<ER>(self, v: &str) -> Result<Self::Value, ER>
		where
			ER: serde::de::Error,
		{
			match BString::from_str(v) {
				Ok(b) => Ok(b),
				Err(_e) => Err(serde::de::Error::invalid_length(
					v.len(),
					&format!("{MAX}").as_str(),
				)),
			}
		}
		fn visit_string<ER>(self, v: String) -> Result<Self::Value, ER>
		where
			ER: serde::de::Error,
		{
			let len = v.len();

			match BString::from_string(v) {
				Ok(b) => Ok(b),
				Err(_e) => Err(serde::de::Error::invalid_length(
					len,
					&format!("{MAX}").as_str(),
				)),
			}
		}
	}
	impl<'de, E: Encoding, const MAX: usize> Deserialize<'de> for BString<MAX, E> {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			deserializer.deserialize_string(BStringVisitor(PhantomData))
		}
	}
}
