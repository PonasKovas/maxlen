use crate::{
	BSlice, BString, LengthExceeded, const_checks,
	encoding::{Encoding, Utf8},
};
use std::{
	borrow::Cow,
	ffi::{OsStr, OsString},
	fmt::Display,
	marker::PhantomData,
	net::ToSocketAddrs,
	ops::{
		Add, AddAssign, Bound, Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
		RangeToInclusive,
	},
	path::Path,
	rc::Rc,
	sync::Arc,
};

/// Bounded [`str`].
///
/// Guaranteed to not be longer than `MAX` bytes in the [`E`][crate::encoding::Encoding] encoding representation.
#[derive(Debug, Hash)]
#[repr(transparent)]
pub struct BStr<const MAX: usize, E = Utf8> {
	phantom: PhantomData<fn(E) -> E>,
	s: str,
}

impl<E: Encoding, const MAX: usize> BStr<MAX, E> {
	/// Creates a `&BStr<MAX, E>` from a `&str` without any checks.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the string is definitely
	/// not longer than `MAX` bytes in the given encoding.
	pub const unsafe fn from_str_unchecked(s: &str) -> &Self {
		unsafe { std::mem::transmute(s) }
	}
	/// Creates a `&mut BStr<MAX, E>` from a `&mut str` without any checks.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the string is definitely
	/// not longer than `MAX` bytes in the given encoding.
	pub const unsafe fn from_str_mut_unchecked(s: &mut str) -> &mut Self {
		unsafe { std::mem::transmute(s) }
	}
	/// Creates a `&BStr<MAX, E>` from a `&str`, performing a runtime check.
	pub fn from_str(s: &str) -> Result<&Self, LengthExceeded> {
		let length = E::length(s);
		if length > MAX {
			return Err(LengthExceeded {
				length,
				maximum: MAX,
			});
		}

		Ok(unsafe { Self::from_str_unchecked(s) })
	}
	/// Creates a `&mut BStr<MAX, E>` from a `&mut str`, performing a runtime check.
	pub fn from_str_mut(s: &mut str) -> Result<&mut Self, LengthExceeded> {
		let length = E::length(s);
		if length > MAX {
			return Err(LengthExceeded {
				length,
				maximum: MAX,
			});
		}

		Ok(unsafe { Self::from_str_mut_unchecked(s) })
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub const fn relax_max<const MAX2: usize>(&self) -> &BStr<MAX2, E> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BStr::from_str_unchecked(&self.s) }
	}
	/// Changes the `MAX` bound (and optionally the encoding type).
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max<E2: Encoding, const MAX2: usize>(
		&self,
	) -> Result<&BStr<MAX2, E2>, LengthExceeded> {
		BStr::from_str(self)
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub const fn relax_max_mut<const MAX2: usize>(&mut self) -> &mut BStr<MAX2, E> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BStr::from_str_mut_unchecked(&mut self.s) }
	}
	/// Changes the `MAX` bound (and optionally the encoding type).
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max_mut<E2: Encoding, const MAX2: usize>(
		&mut self,
	) -> Result<&mut BStr<MAX2, E2>, LengthExceeded> {
		BStr::from_str_mut(&mut self.s)
	}
	/// Divides one mutable string slice into two at an index.
	///
	/// See [`str::split_at_mut`] for more information.
	pub fn split_at_mut(&mut self, mid: usize) -> (&mut Self, &mut Self) {
		let (l, r) = self.s.split_at_mut(mid);

		// Two subslices will always be shorter than the original
		// and therefore valid for the length constrains
		unsafe {
			(
				Self::from_str_mut_unchecked(l),
				Self::from_str_mut_unchecked(r),
			)
		}
	}
	/// Converts this string to its ASCII upper case equivalent in-place.
	///
	/// See [`str::make_ascii_uppercase`] for more information.
	pub fn make_ascii_uppercase(&mut self) {
		self.s.make_ascii_uppercase();
	}
	/// Converts this string to its ASCII lower case equivalent in-place.
	///
	/// See [`str::make_ascii_lowercase`] for more information.
	pub fn make_ascii_lowercase(&mut self) {
		self.s.make_ascii_lowercase();
	}
}

// Trait implementations relating BStr and BString
//////////////////////////////////////////////////

impl<E: Encoding, const MAX: usize> ToOwned for BStr<MAX, E> {
	type Owned = BString<MAX, E>;

	fn to_owned(&self) -> Self::Owned {
		unsafe { BString::from_string_unchecked(self.to_string()) }
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for BString<MAX, E> {
	fn from(value: &BStr<MAX, E>) -> Self {
		value.to_owned()
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for BString<MAX, E> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		value.to_owned()
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<BString<MAX2, E2>>
	for BStr<MAX1, E1>
{
	fn eq(&self, other: &BString<MAX2, E2>) -> bool {
		(**self).eq(&***other)
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<&BString<MAX2, E2>>
	for BStr<MAX1, E1>
{
	fn eq(&self, other: &&BString<MAX2, E2>) -> bool {
		(**self).eq(&***other)
	}
}

// Trait implementations mirroring standard str
///////////////////////////////////////////////

impl<'a, E: Encoding, const MAX: usize> TryFrom<&'a str> for &'a BStr<MAX, E> {
	type Error = LengthExceeded;

	fn try_from(value: &'a str) -> Result<Self, Self::Error> {
		BStr::from_str(value)
	}
}
impl<E, const MAX: usize> Deref for BStr<MAX, E> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.s
	}
}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialEq<BStr<MAX2, E2>>
	for BStr<MAX1, E1>
{
	fn eq(&self, other: &BStr<MAX2, E2>) -> bool {
		(**self).eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Eq for BStr<MAX, E> {}
impl<E1: Encoding, E2: Encoding, const MAX1: usize, const MAX2: usize> PartialOrd<BStr<MAX2, E2>>
	for BStr<MAX1, E1>
{
	fn partial_cmp(&self, other: &BStr<MAX2, E2>) -> Option<std::cmp::Ordering> {
		(**self).partial_cmp(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Ord for BStr<MAX, E> {
	fn cmp(&self, other: &BStr<MAX, E>) -> std::cmp::Ordering {
		(**self).cmp(&**other)
	}
}
impl<E: Encoding, const MAX: usize> Default for &BStr<MAX, E> {
	fn default() -> Self {
		unsafe { &BStr::from_str_unchecked(Default::default()) }
	}
}
impl<E: Encoding, const MAX: usize> Display for BStr<MAX, E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		(**self).fmt(f)
	}
}
macro_rules! impl_index {
	($index:ty) => {
		impl<E: Encoding, const MAX: usize> Index<$index> for BStr<MAX, E> {
			type Output = Self;

			fn index(&self, index: $index) -> &Self::Output {
				unsafe { Self::from_str_unchecked((**self).index(index)) }
			}
		}
	};
}
impl_index! {(Bound<usize>, Bound<usize>)}
impl_index! {Range<usize>}
impl_index! {RangeFrom<usize>}
impl_index! {RangeFull}
impl_index! {RangeInclusive<usize>}
impl_index! {RangeTo<usize>}
impl_index! {RangeToInclusive<usize>}

impl<E: Encoding, const MAX: usize> AsRef<OsStr> for BStr<MAX, E> {
	fn as_ref(&self) -> &OsStr {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<Path> for BStr<MAX, E> {
	fn as_ref(&self) -> &Path {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<[u8]> for BStr<MAX, E> {
	fn as_ref(&self) -> &[u8] {
		(**self).as_ref()
	}
}
impl<E: Encoding, const MAX: usize> AsRef<BSlice<u8, MAX>> for BStr<MAX, E> {
	fn as_ref(&self) -> &BSlice<u8, MAX> {
		unsafe { BSlice::from_slice_unchecked(self.as_bytes()) }
	}
}
impl<E: Encoding, const MAX: usize> AsRef<Self> for BStr<MAX, E> {
	fn as_ref(&self) -> &Self {
		self
	}
}
impl<E: Encoding, const MAX: usize> AsRef<str> for BStr<MAX, E> {
	fn as_ref(&self) -> &str {
		&**self
	}
}
impl<E: Encoding, const MAX: usize> Clone for Box<BStr<MAX, E>> {
	fn clone(&self) -> Self {
		(**self).into()
	}
}
impl<'a, E: Encoding, const MAX: usize> From<&'a BStr<MAX, E>> for Cow<'a, BStr<MAX, E>> {
	fn from(value: &'a BStr<MAX, E>) -> Self {
		Self::Borrowed(value)
	}
}
impl<'a, E: Encoding, const MAX: usize> From<&'a mut BStr<MAX, E>> for Cow<'a, BStr<MAX, E>> {
	fn from(value: &'a mut BStr<MAX, E>) -> Self {
		Self::Borrowed(value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for Arc<BStr<MAX, E>> {
	fn from(value: &BStr<MAX, E>) -> Self {
		let arc = Arc::<str>::from(&**value);

		unsafe { Arc::from_raw(Arc::into_raw(arc) as *const BStr<MAX, E>) }
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for Arc<BStr<MAX, E>> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for Arc<str> {
	fn from(value: &BStr<MAX, E>) -> Self {
		Arc::<str>::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for Arc<str> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for Box<BStr<MAX, E>> {
	fn from(value: &BStr<MAX, E>) -> Self {
		let b = Box::<str>::from(&**value);

		unsafe { Box::from_raw(Box::into_raw(b) as *mut BStr<MAX, E>) }
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for Box<BStr<MAX, E>> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for Rc<BStr<MAX, E>> {
	fn from(value: &BStr<MAX, E>) -> Self {
		let b = Rc::<str>::from(&**value);

		unsafe { Rc::from_raw(Rc::into_raw(b) as *mut BStr<MAX, E>) }
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for Rc<BStr<MAX, E>> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for Box<dyn std::error::Error> {
	fn from(value: &BStr<MAX, E>) -> Self {
		Self::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for Box<dyn std::error::Error> {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>>
	for Box<dyn std::error::Error + Sync + Send>
{
	fn from(value: &BStr<MAX, E>) -> Self {
		Self::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>>
	for Box<dyn std::error::Error + Sync + Send>
{
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<&BStr<MAX, E>> for String {
	fn from(value: &BStr<MAX, E>) -> Self {
		Self::from(&**value)
	}
}
impl<E: Encoding, const MAX: usize> From<&mut BStr<MAX, E>> for String {
	fn from(value: &mut BStr<MAX, E>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<Box<BStr<MAX, E>>> for BString<MAX, E> {
	fn from(value: Box<BStr<MAX, E>>) -> Self {
		Self::from(&*value)
	}
}
impl<E: Encoding, const MAX: usize> From<Cow<'_, BStr<MAX, E>>> for Box<BStr<MAX, E>> {
	fn from(value: Cow<'_, BStr<MAX, E>>) -> Self {
		match value {
			Cow::Borrowed(v) => v.into(),
			Cow::Owned(v) => v.into(),
		}
	}
}
impl<E: Encoding, const MAX: usize> From<Box<BStr<MAX, E>>> for Box<str> {
	fn from(value: Box<BStr<MAX, E>>) -> Self {
		unsafe { Box::from_raw(Box::into_raw(value) as *mut str) }
	}
}
impl<E: Encoding, const MAX: usize> Add<&BStr<MAX, E>> for String {
	type Output = Self;

	fn add(self, rhs: &BStr<MAX, E>) -> Self::Output {
		self.add(&**rhs)
	}
}
impl<'a, E: Encoding, const MAX: usize> Add<&'a BStr<MAX, E>> for Cow<'a, str> {
	type Output = Self;

	fn add(self, rhs: &'a BStr<MAX, E>) -> Self::Output {
		self.add(&**rhs)
	}
}
impl<E: Encoding, const MAX: usize> AddAssign<&BStr<MAX, E>> for String {
	fn add_assign(&mut self, rhs: &BStr<MAX, E>) {
		self.add_assign(&**rhs);
	}
}
impl<'a, E: Encoding, const MAX: usize> AddAssign<&'a BStr<MAX, E>> for Cow<'a, str> {
	fn add_assign(&mut self, rhs: &'a BStr<MAX, E>) {
		self.add_assign(&**rhs);
	}
}
impl<'a, E: Encoding, const MAX: usize> Extend<&'a BStr<MAX, E>> for String {
	fn extend<T: IntoIterator<Item = &'a BStr<MAX, E>>>(&mut self, iter: T) {
		for i in iter {
			self.push_str(i);
		}
	}
}
impl<'a, E: Encoding, const MAX: usize> FromIterator<&'a BStr<MAX, E>> for Box<str> {
	fn from_iter<T: IntoIterator<Item = &'a BStr<MAX, E>>>(iter: T) -> Self {
		let mut s = String::new();
		s.extend(iter);
		s.into_boxed_str()
	}
}
impl<'a, E: Encoding, const MAX: usize> FromIterator<&'a BStr<MAX, E>> for String {
	fn from_iter<T: IntoIterator<Item = &'a BStr<MAX, E>>>(iter: T) -> Self {
		let mut s = String::new();
		s.extend(iter);
		s
	}
}
impl<'a, 'b, E: Encoding, const MAX: usize> FromIterator<&'a BStr<MAX, E>> for Cow<'b, str> {
	fn from_iter<T: IntoIterator<Item = &'a BStr<MAX, E>>>(iter: T) -> Self {
		Cow::Owned(String::from_iter(iter))
	}
}
impl<E: Encoding, const MAX: usize> FromIterator<Box<BStr<MAX, E>>> for Box<str> {
	fn from_iter<T: IntoIterator<Item = Box<BStr<MAX, E>>>>(iter: T) -> Self {
		let mut s = String::new();
		for i in iter {
			s.push_str(&**i);
		}
		s.into_boxed_str()
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BStr<MAX, E>> for String {
	fn eq(&self, other: &BStr<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BStr<MAX, E>> for String {
	fn eq(&self, other: &&BStr<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<String> for BStr<MAX, E> {
	fn eq(&self, other: &String) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<String> for &BStr<MAX, E> {
	fn eq(&self, other: &String) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BStr<MAX, E>> for Cow<'_, str> {
	fn eq(&self, other: &BStr<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BStr<MAX, E>> for Cow<'_, str> {
	fn eq(&self, other: &&BStr<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<Cow<'_, str>> for BStr<MAX, E> {
	fn eq(&self, other: &Cow<'_, str>) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<Cow<'_, str>> for &BStr<MAX, E> {
	fn eq(&self, other: &Cow<'_, str>) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BStr<MAX, E>> for OsStr {
	fn eq(&self, other: &BStr<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BStr<MAX, E>> for OsStr {
	fn eq(&self, other: &&BStr<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsStr> for BStr<MAX, E> {
	fn eq(&self, other: &OsStr) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsStr> for &BStr<MAX, E> {
	fn eq(&self, other: &OsStr) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BStr<MAX, E>> for OsString {
	fn eq(&self, other: &BStr<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BStr<MAX, E>> for OsString {
	fn eq(&self, other: &&BStr<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsString> for BStr<MAX, E> {
	fn eq(&self, other: &OsString) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<OsString> for &BStr<MAX, E> {
	fn eq(&self, other: &OsString) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<BStr<MAX, E>> for str {
	fn eq(&self, other: &BStr<MAX, E>) -> bool {
		self.eq(&**other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<&BStr<MAX, E>> for str {
	fn eq(&self, other: &&BStr<MAX, E>) -> bool {
		self.eq(&***other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<str> for BStr<MAX, E> {
	fn eq(&self, other: &str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> PartialEq<str> for &BStr<MAX, E> {
	fn eq(&self, other: &str) -> bool {
		(**self).eq(other)
	}
}
impl<E: Encoding, const MAX: usize> ToSocketAddrs for BStr<MAX, E> {
	type Iter = <str as ToSocketAddrs>::Iter;

	fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
		(**self).to_socket_addrs()
	}
}
