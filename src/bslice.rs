use crate::{BVec, LengthExceeded, const_checks};
use std::{
	borrow::{Borrow, BorrowMut, Cow},
	collections::VecDeque,
	io::{BufRead, Read, Write},
	mem::transmute,
	ops::{
		Bound, Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive,
		RangeTo, RangeToInclusive,
	},
	rc::Rc,
	sync::Arc,
};

/// Bounded [`[T]`][slice].
///
/// Guaranteed to not be longer than `MAX` elements .
#[derive(Debug, Hash)]
#[repr(transparent)]
pub struct BSlice<T, const MAX: usize> {
	s: [T],
}

/// Creates a static `&'static BSlice<T, MAX>` with a compile-time check.
///
/// ```
/// # use maxlen::{bslice, BSlice};
/// let _: &BSlice<u8, 255> = bslice![];
/// let _: &BSlice<_, 255> = bslice![0; 20];
/// let _: &BSlice<_, 255> = bslice![0, 1, 2, 3, 4];
///
/// // let _: &BSlice<_, 1> = bslice![0; 2]; // will not compile
/// ```
#[macro_export]
macro_rules! bslice {
	() => {
		unsafe { $crate::BSlice::from_slice_unchecked(&[]) }
	};
	($elem:expr; $n:expr) => {
		$crate::BSlice::from_array(&[$elem; $n])
	};
	($($x:expr),+ $(,)?) => {
		$crate::BSlice::from_array(&[$($x),+])
	};
}

impl<T, const MAX: usize> BSlice<T, MAX> {
	/// Creates a `&BSlice<T, MAX>` from a slice without any checks.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the slice is definitely not longer than `MAX` elements.
	pub const unsafe fn from_slice_unchecked(s: &[T]) -> &Self {
		unsafe { std::mem::transmute(s) }
	}
	/// Creates a `&mut BSlice<T, MAX>` from a mutable slice without any checks.
	///
	/// # Safety
	///
	/// The caller is responsible for making sure that the slice is definitely not longer than `MAX` elements.
	pub const unsafe fn from_slice_mut_unchecked(s: &mut [T]) -> &mut Self {
		unsafe { std::mem::transmute(s) }
	}
	/// Creates a `&BSlice<T, MAX>` from a slice, performing a runtime check.
	pub fn from_slice(s: &[T]) -> Result<&Self, LengthExceeded> {
		if s.len() > MAX {
			return Err(LengthExceeded {
				length: s.len(),
				maximum: MAX,
			});
		}

		Ok(unsafe { Self::from_slice_unchecked(s) })
	}
	/// Creates a `&mut BSlice<T, MAX>` from a mutable slice, performing a runtime check.
	pub fn from_slice_mut(s: &mut [T]) -> Result<&mut Self, LengthExceeded> {
		if s.len() > MAX {
			return Err(LengthExceeded {
				length: s.len(),
				maximum: MAX,
			});
		}

		Ok(unsafe { Self::from_slice_mut_unchecked(s) })
	}
	/// Constructs a `&BSlice<T, MAX>` from a `&[T; N]` using a compile-time check
	pub const fn from_array<const N: usize>(v: &[T; N]) -> &BSlice<T, MAX> {
		// compile time check
		_ = <const_checks::Pair<MAX, N> as const_checks::AssertGe>::VALID;

		unsafe { BSlice::from_slice_unchecked(v) }
	}
	/// Constructs a `&mut BSlice<T, MAX>` from a `&mut [T; N]` using a compile-time check
	pub const fn from_array_mut<const N: usize>(v: &mut [T; N]) -> &mut BSlice<T, MAX> {
		// compile time check
		_ = <const_checks::Pair<MAX, N> as const_checks::AssertGe>::VALID;

		unsafe { BSlice::from_slice_mut_unchecked(v) }
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub const fn relax_max<const MAX2: usize>(&self) -> &BSlice<T, MAX2> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BSlice::from_slice_unchecked(&self.s) }
	}
	/// Changes the `MAX` bound.
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max<const MAX2: usize>(&self) -> Result<&BSlice<T, MAX2>, LengthExceeded> {
		BSlice::from_slice(self)
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub const fn relax_max_mut<const MAX2: usize>(&mut self) -> &mut BSlice<T, MAX2> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BSlice::from_slice_mut_unchecked(&mut self.s) }
	}
	/// Changes the `MAX` bound.
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max_mut<const MAX2: usize>(
		&mut self,
	) -> Result<&mut BSlice<T, MAX2>, LengthExceeded> {
		BSlice::from_slice_mut(self)
	}
}

// Trait implementations relating BSlice and BVec
//////////////////////////////////////////////////

impl<T: Clone, const MAX: usize> ToOwned for BSlice<T, MAX> {
	type Owned = BVec<T, MAX>;

	fn to_owned(&self) -> Self::Owned {
		unsafe { BVec::from_vec_unchecked(self.to_vec()) }
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for BVec<T, MAX> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		value.to_owned()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for BVec<T, MAX> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		value.to_owned()
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<BVec<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &BVec<U, MAX2>) -> bool {
		(**self).eq(&***other)
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<&BVec<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &&BVec<U, MAX2>) -> bool {
		(**self).eq(&****other)
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<&mut BVec<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &&mut BVec<U, MAX2>) -> bool {
		(**self).eq(&****other)
	}
}

// Trait implementations mirroring standard slice
/////////////////////////////////////////////////

impl<'a, T, const MAX: usize> TryFrom<&'a [T]> for &'a BSlice<T, MAX> {
	type Error = LengthExceeded;

	fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
		BSlice::from_slice(value)
	}
}
impl<'a, T, const MAX: usize> TryFrom<&'a mut [T]> for &'a mut BSlice<T, MAX> {
	type Error = LengthExceeded;

	fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
		BSlice::from_slice_mut(value)
	}
}
impl<T, const MAX: usize> Deref for BSlice<T, MAX> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&self.s
	}
}
impl<T, const MAX: usize> DerefMut for BSlice<T, MAX> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.s
	}
}
impl<T, const MAX: usize> AsRef<Self> for BSlice<T, MAX> {
	fn as_ref(&self) -> &Self {
		self
	}
}
impl<T, const MAX: usize> AsMut<Self> for BSlice<T, MAX> {
	fn as_mut(&mut self) -> &mut Self {
		self
	}
}
impl<T, const MAX: usize> AsRef<[T]> for BSlice<T, MAX> {
	fn as_ref(&self) -> &[T] {
		self
	}
}
impl<T, const MAX: usize> AsMut<[T]> for BSlice<T, MAX> {
	fn as_mut(&mut self) -> &mut [T] {
		self
	}
}
impl<T, const MAX: usize> Borrow<[T]> for BSlice<T, MAX> {
	fn borrow(&self) -> &[T] {
		self
	}
}
impl<T, const MAX: usize> BorrowMut<[T]> for BSlice<T, MAX> {
	fn borrow_mut(&mut self) -> &mut [T] {
		self
	}
}
impl<const MAX: usize> BufRead for &BSlice<u8, MAX> {
	fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
		convert_mut_ref(self).fill_buf()
	}
	fn consume(&mut self, amt: usize) {
		convert_mut_ref(self).consume(amt)
	}
}
impl<T: Clone, const MAX: usize> Clone for Box<BSlice<T, MAX>> {
	fn clone(&self) -> Self {
		unsafe {
			Box::from_raw(Box::into_raw(self.to_vec().into_boxed_slice()) as *mut BSlice<T, MAX>)
		}
	}
}
impl<T, const MAX: usize> Default for &BSlice<T, MAX> {
	fn default() -> Self {
		unsafe { BSlice::from_slice_unchecked(Default::default()) }
	}
}
impl<T, const MAX: usize> Default for &mut BSlice<T, MAX> {
	fn default() -> Self {
		unsafe { BSlice::from_slice_mut_unchecked(Default::default()) }
	}
}
impl<T, const MAX: usize> Default for Box<BSlice<T, MAX>> {
	fn default() -> Self {
		unsafe { Box::from_raw(Box::into_raw(Box::<[T]>::default()) as *mut BSlice<T, MAX>) }
	}
}
impl<'a, T: Clone, const MAX: usize> From<&'a BSlice<T, MAX>> for Cow<'a, BSlice<T, MAX>> {
	fn from(value: &'a BSlice<T, MAX>) -> Self {
		Cow::Borrowed(value)
	}
}
impl<'a, T: Clone, const MAX: usize> From<&'a BSlice<T, MAX>> for Cow<'a, [T]> {
	fn from(value: &'a BSlice<T, MAX>) -> Self {
		Cow::Borrowed(value)
	}
}
impl<'a, T: Clone, const MAX: usize> From<&'a mut BSlice<T, MAX>> for Cow<'a, BSlice<T, MAX>> {
	fn from(value: &'a mut BSlice<T, MAX>) -> Self {
		Cow::Borrowed(value)
	}
}
impl<'a, T: Clone, const MAX: usize> From<&'a mut BSlice<T, MAX>> for Cow<'a, [T]> {
	fn from(value: &'a mut BSlice<T, MAX>) -> Self {
		Cow::Borrowed(value)
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Arc<BSlice<T, MAX>> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		Box::<BSlice<T, MAX>>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Arc<[T]> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Arc<BSlice<T, MAX>> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		Box::<BSlice<T, MAX>>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Arc<[T]> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Box<BSlice<T, MAX>> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		let b = Box::<[T]>::from(&**value);

		unsafe { Box::from_raw(Box::into_raw(b) as *mut BSlice<T, MAX>) }
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Box<[T]> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(&**value)
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Box<BSlice<T, MAX>> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		let b = Box::<[T]>::from(&**value);

		unsafe { Box::from_raw(Box::into_raw(b) as *mut BSlice<T, MAX>) }
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Box<[T]> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(&**value)
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Rc<BSlice<T, MAX>> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		Box::<BSlice<T, MAX>>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Rc<[T]> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Rc<BSlice<T, MAX>> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		Box::<BSlice<T, MAX>>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Rc<[T]> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		Box::<[T]>::from(value).into()
	}
}
impl<T: Clone, const MAX: usize> From<&BSlice<T, MAX>> for Vec<T> {
	fn from(value: &BSlice<T, MAX>) -> Self {
		value.to_vec()
	}
}
impl<T: Clone, const MAX: usize> From<&mut BSlice<T, MAX>> for Vec<T> {
	fn from(value: &mut BSlice<T, MAX>) -> Self {
		value.to_vec()
	}
}
impl<T, const MAX: usize> Index<usize> for BSlice<T, MAX> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		(**self).index(index)
	}
}
impl<T, const MAX: usize> IndexMut<usize> for BSlice<T, MAX> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		(**self).index_mut(index)
	}
}
macro_rules! impl_index {
	($index:ty) => {
		impl<T, const MAX: usize> Index<$index> for BSlice<T, MAX> {
			type Output = Self;

			fn index(&self, index: $index) -> &Self::Output {
				unsafe { Self::from_slice_unchecked((**self).index(index)) }
			}
		}
		impl<T, const MAX: usize> IndexMut<$index> for BSlice<T, MAX> {
			fn index_mut(&mut self, index: $index) -> &mut Self::Output {
				unsafe { Self::from_slice_mut_unchecked((**self).index_mut(index)) }
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
impl<'a, T, const MAX: usize> IntoIterator for &'a BSlice<T, MAX> {
	type Item = &'a T;
	type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&**self).into_iter()
	}
}
impl<'a, T, const MAX: usize> IntoIterator for &'a mut BSlice<T, MAX> {
	type Item = &'a mut T;
	type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&mut **self).into_iter()
	}
}
impl<'a, 'b, T, const MAX: usize> IntoIterator for &'b &'a BSlice<T, MAX> {
	type Item = &'b T;
	type IntoIter = <&'b [T] as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&**self).into_iter()
	}
}
impl<T, const MAX: usize> IntoIterator for Box<BSlice<T, MAX>> {
	type Item = T;
	type IntoIter = <Box<[T]> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		unsafe { Box::from_raw(Box::into_raw(self) as *mut [T]) }.into_iter()
	}
}
impl<'a, T, const MAX: usize> IntoIterator for &'a Box<BSlice<T, MAX>> {
	type Item = &'a T;
	type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&**self).into_iter()
	}
}
impl<'a, T, const MAX: usize> IntoIterator for &'a mut Box<BSlice<T, MAX>> {
	type Item = &'a mut T;
	type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		(&mut **self).into_iter()
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<BSlice<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &BSlice<U, MAX2>) -> bool {
		(**self).eq(&**other)
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<&BSlice<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &&BSlice<U, MAX2>) -> bool {
		(**self).eq(&***other)
	}
}
impl<T: PartialEq<U>, U, const MAX1: usize, const MAX2: usize> PartialEq<&mut BSlice<U, MAX2>>
	for BSlice<T, MAX1>
{
	fn eq(&self, other: &&mut BSlice<U, MAX2>) -> bool {
		(**self).eq(&***other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<BSlice<U, MAX>> for [T] {
	fn eq(&self, other: &BSlice<U, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<&BSlice<U, MAX>> for [T] {
	fn eq(&self, other: &&BSlice<U, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<&mut BSlice<U, MAX>> for [T] {
	fn eq(&self, other: &&mut BSlice<U, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<[U]> for BSlice<T, MAX> {
	fn eq(&self, other: &[U]) -> bool {
		(**self).eq(other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<[U]> for &BSlice<T, MAX> {
	fn eq(&self, other: &[U]) -> bool {
		(**self).eq(other)
	}
}
impl<T: PartialEq<U>, U, const MAX: usize> PartialEq<[U]> for &mut BSlice<T, MAX> {
	fn eq(&self, other: &[U]) -> bool {
		(**self).eq(other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<BSlice<T, MAX>> for [U; N]
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &BSlice<T, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<&BSlice<T, MAX>> for [U; N]
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&BSlice<T, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<&mut BSlice<T, MAX>> for [U; N]
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&mut BSlice<T, MAX>) -> bool {
		self.eq(&**other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<[U; N]> for BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &[U; N]) -> bool {
		(**self).eq(other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<[U; N]> for &BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &[U; N]) -> bool {
		(**self).eq(other)
	}
}
impl<T, U, const MAX: usize, const N: usize> PartialEq<[U; N]> for &mut BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &[U; N]) -> bool {
		(**self).eq(other)
	}
}
impl<T, U, const MAX: usize> PartialEq<BSlice<T, MAX>> for Cow<'_, [U]>
where
	U: PartialEq<T> + Clone,
{
	fn eq(&self, other: &BSlice<T, MAX>) -> bool {
		self.eq(&&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&BSlice<T, MAX>> for Cow<'_, [U]>
where
	U: PartialEq<T> + Clone,
{
	fn eq(&self, other: &&BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&mut BSlice<T, MAX>> for Cow<'_, [U]>
where
	U: PartialEq<T> + Clone,
{
	fn eq(&self, other: &&mut BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Cow<'_, [U]>> for BSlice<T, MAX>
where
	T: PartialEq<U>,
	U: Clone,
{
	fn eq(&self, other: &Cow<'_, [U]>) -> bool {
		(**self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Cow<'_, [U]>> for &BSlice<T, MAX>
where
	T: PartialEq<U>,
	U: Clone,
{
	fn eq(&self, other: &Cow<'_, [U]>) -> bool {
		(***self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Cow<'_, [U]>> for &mut BSlice<T, MAX>
where
	T: PartialEq<U>,
	U: Clone,
{
	fn eq(&self, other: &Cow<'_, [U]>) -> bool {
		(***self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<BSlice<T, MAX>> for Vec<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &BSlice<T, MAX>) -> bool {
		self.eq(&&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&BSlice<T, MAX>> for Vec<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&mut BSlice<T, MAX>> for Vec<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&mut BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Vec<U>> for BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &Vec<U>) -> bool {
		(**self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Vec<U>> for &BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &Vec<U>) -> bool {
		(***self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<Vec<U>> for &mut BSlice<T, MAX>
where
	T: PartialEq<U>,
{
	fn eq(&self, other: &Vec<U>) -> bool {
		(***self).eq(&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<BSlice<T, MAX>> for VecDeque<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &BSlice<T, MAX>) -> bool {
		self.eq(&&**other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&BSlice<T, MAX>> for VecDeque<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T, U, const MAX: usize> PartialEq<&mut BSlice<T, MAX>> for VecDeque<U>
where
	U: PartialEq<T>,
{
	fn eq(&self, other: &&mut BSlice<T, MAX>) -> bool {
		self.eq(&&***other)
	}
}
impl<T: Eq, const MAX: usize> Eq for BSlice<T, MAX> {}
impl<T: PartialOrd, const MAX1: usize, const MAX2: usize> PartialOrd<BSlice<T, MAX2>>
	for BSlice<T, MAX1>
{
	fn partial_cmp(&self, other: &BSlice<T, MAX2>) -> Option<std::cmp::Ordering> {
		(**self).partial_cmp(&**other)
	}
}
impl<T: PartialOrd, const MAX: usize> PartialOrd<BSlice<T, MAX>> for [T] {
	fn partial_cmp(&self, other: &BSlice<T, MAX>) -> Option<std::cmp::Ordering> {
		self.partial_cmp(&**other)
	}
}
impl<T: PartialOrd, const MAX: usize> PartialOrd<&BSlice<T, MAX>> for [T] {
	fn partial_cmp(&self, other: &&BSlice<T, MAX>) -> Option<std::cmp::Ordering> {
		self.partial_cmp(&***other)
	}
}
impl<T: PartialOrd, const MAX: usize> PartialOrd<&mut BSlice<T, MAX>> for [T] {
	fn partial_cmp(&self, other: &&mut BSlice<T, MAX>) -> Option<std::cmp::Ordering> {
		self.partial_cmp(&***other)
	}
}
impl<T: Ord, const MAX: usize> Ord for BSlice<T, MAX> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		(**self).cmp(&**other)
	}
}
impl<const MAX: usize> Read for &BSlice<u8, MAX> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		convert_mut_ref(self).read(buf)
	}
}
impl<const MAX: usize> Write for &mut BSlice<u8, MAX> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		convert_mut_mut(self).write(buf)
	}
	fn flush(&mut self) -> std::io::Result<()> {
		convert_mut_mut(self).flush()
	}
}

fn convert_mut_ref<'a, 'b, T, const MAX: usize>(v: &'a mut &'b BSlice<T, MAX>) -> &'a mut &'b [T] {
	unsafe { transmute(v) }
}
fn convert_mut_mut<'a, 'b, T, const MAX: usize>(
	v: &'a mut &'b mut BSlice<T, MAX>,
) -> &'a mut &'b mut [T] {
	unsafe { transmute(v) }
}

#[cfg(feature = "serde")]
mod serde_impls {
	use super::*;
	use serde::{Deserialize, Serialize, de::Visitor, ser::SerializeSeq};

	impl<'a, T: Serialize, const MAX: usize> Serialize for &'a BSlice<T, MAX> {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			let mut seq = serializer.serialize_seq(Some(self.len()))?;
			for e in self {
				seq.serialize_element(e)?;
			}
			seq.end()
		}
	}

	// Deserialize only implemented for [u8] because its a SLICE.
	struct BSLiceVisitor<const MAX: usize>;
	impl<'de, const MAX: usize> Visitor<'de> for BSLiceVisitor<MAX> {
		type Value = &'de BSlice<u8, MAX>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a byte slice")
		}
		fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			match BSlice::from_slice(v) {
				Ok(b) => Ok(b),
				Err(_e) => Err(serde::de::Error::invalid_length(
					v.len(),
					&format!("{MAX}").as_str(),
				)),
			}
		}
	}
	impl<'de, const MAX: usize> Deserialize<'de> for &'de BSlice<u8, MAX> {
		fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			deserializer.deserialize_seq(BSLiceVisitor)
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::*;

	#[test]
	fn test_bslice_macro() {
		let _: &BSlice<u8, 255> = bslice![];
		let _: &BSlice<_, 255> = bslice![0; 20];
		// let _: &BSlice<_, 1> = bslice![0; 2]; // should fail
		let _: &BSlice<_, 255> = bslice![0, 1, 2, 3, 4];
		// let _: &BSlice<_, 3> = bslice![0, 1, 2, 3, 4]; // should fail
	}
}
