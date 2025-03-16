use std::{
	borrow::Borrow,
	ops::{Deref, DerefMut, RangeBounds},
};

use crate::{BSlice, LengthExceeded, const_checks};

/// Bounded [`Vec`].
///
/// Guaranteed to not be longer than `MAX` elements.
#[derive(Debug, Default, Hash)]
pub struct BVec<T, const MAX: usize> {
	s: Vec<T>,
}

impl<T, const MAX: usize> BVec<T, MAX> {
	pub unsafe fn from_vec_unchecked(s: Vec<T>) -> Self {
		Self { s }
	}
	pub unsafe fn from_slice_unchecked(s: &[T]) -> Self
	where
		T: Clone,
	{
		Self { s: s.to_owned() }
	}
	pub fn from_slice(s: &[T]) -> Result<Self, LengthExceeded>
	where
		T: Clone,
	{
		BSlice::from_slice(s).map(|s| s.to_owned())
	}
	pub fn from_vec(s: Vec<T>) -> Result<Self, LengthExceeded> {
		BSlice::<T, MAX>::from_slice(&s)?;

		Ok(unsafe { Self::from_vec_unchecked(s) })
	}
	/// Gives the inner [`Vec<T>`].
	pub fn into_inner(self) -> Vec<T> {
		self.s
	}
	/// Relaxes the `MAX` bound, converting to a type with a bigger one.
	///
	/// This conversion is free and does not involve any checks. It is
	/// asserted at compile time that the new `MAX` is bigger than before.
	pub fn relax_max<const MAX2: usize>(self) -> BVec<T, MAX2> {
		// assert that MAX2 >= MAX at compile time
		_ = <const_checks::Pair<MAX2, MAX> as const_checks::AssertGe>::VALID;

		unsafe { BVec::from_vec_unchecked(self.s) }
	}
	/// Changes the `MAX` bound.
	///
	/// This involves a check whether the new bound is met.
	pub fn change_max<const MAX2: usize>(self) -> Result<BVec<T, MAX2>, LengthExceeded> {
		BVec::from_vec(self.s)
	}
	/// Returns a raw mutable pointer to the vector’s buffer, or a dangling raw pointer valid for zero sized reads if the vector didn’t allocate.
	///
	/// See [`Vec::as_mut_ptr`] for more information.
	pub const fn as_mut_ptr(&mut self) -> *mut T {
		self.s.as_mut_ptr()
	}
	/// Returns a raw pointer to the vector’s buffer, or a dangling raw pointer valid for zero sized reads if the vector didn’t allocate.
	///
	/// See [`Vec::as_ptr`] for more information.
	pub const fn as_ptr(&self) -> *const T {
		self.s.as_ptr()
	}
	/// Extracts a mutable slice of the entire vector.
	///
	/// See [`Vec::as_mut_slice`] for more information.
	pub const fn as_mut_slice(&mut self) -> &mut BSlice<T, MAX> {
		unsafe { BSlice::from_slice_mut_unchecked(self.s.as_mut_slice()) }
	}
	/// Returns the total number of elements the vector can hold without reallocating.
	///
	/// See [`Vec::capacity`] for more information.
	pub const fn capacity(&self) -> usize {
		self.s.capacity()
	}
	/// Clears the vector, removing all values.
	///
	/// See [`Vec::clear`] for more information.
	pub fn clear(&mut self) {
		self.s.clear()
	}
	/// Removes all but the first of consecutive elements in the vector satisfying a given equality relation.
	///
	/// See [`Vec::dedup_by`] for more information.
	pub fn dedup_by<F>(&mut self, f: F)
	where
		F: FnMut(&mut T, &mut T) -> bool,
	{
		self.s.dedup_by(f)
	}
	/// Removes all but the first of consecutive elements in the vector that resolve to the same key.
	///
	/// See [`Vec::dedup_by_key`] for more information.
	pub fn dedup_by_key<F, K: PartialEq>(&mut self, f: F)
	where
		F: FnMut(&mut T) -> K,
	{
		self.s.dedup_by_key(f)
	}
	/// Removes the subslice indicated by the given range from the vector, returning a double-ended iterator over the removed subslice.
	///
	/// See [`Vec::drain`] for more information.
	pub fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> std::vec::Drain<T> {
		self.s.drain(range)
	}
	/// Creates an iterator which uses a closure to determine if element in the range should be removed.
	///
	/// See [`Vec::extract_if`] for more information.
	pub fn extract_if<F, R: RangeBounds<usize>>(
		&mut self,
		range: R,
		filter: F,
	) -> std::vec::ExtractIf<T, F>
	where
		F: FnMut(&mut T) -> bool,
	{
		self.s.extract_if(range, filter)
	}
	/// Converts the vector into [`Box<BSlice<T, MAX>>`].
	///
	/// See [`Vec::into_boxed_slice`] for more information.
	pub fn into_boxed_slice(self) -> Box<BSlice<T, MAX>> {
		unsafe { Box::from_raw(Box::into_raw(self.s.into_boxed_slice()) as *mut BSlice<T, MAX>) }
	}
	/// Consumes and leaks the [`BVec`], returning a mutable reference to the contents, `&'a mut BSlice<T, MAX>`.
	///
	/// See [`Vec::leak`] for more information.
	pub fn leak<'a>(self) -> &'a mut BSlice<T, MAX> {
		unsafe { BSlice::from_slice_mut_unchecked(self.s.leak()) }
	}
	/// Constructs a new, empty [`BVec<T>`].
	///
	/// See [`Vec::new`] for more information.
	pub fn new() -> Self {
		Self { s: Vec::new() }
	}
	/// Removes the last element from a vector and returns it, or None if it is empty.
	///
	/// See [`Vec::pop`] for more information.
	pub fn pop(&mut self) -> Option<T> {
		self.s.pop()
	}
	/// Removes and returns the last element from a vector if the predicate returns `true`,
	/// or `None` if the predicate returns `false` or the vector is empty (the predicate will not be called in that case).
	///
	/// See [`Vec::pop_if`] for more information.
	pub fn pop_if(&mut self, f: impl FnOnce(&mut T) -> bool) -> Option<T> {
		self.s.pop_if(f)
	}
	/// Removes and returns the element at position index within the vector, shifting all elements after it to the left.
	///
	/// See [`Vec::remove`] for more information.
	pub fn remove(&mut self, idx: usize) -> T {
		self.s.remove(idx)
	}
	/// Reserves capacity for at least `additional` more elements to be inserted in the given [`BVec<T>`].
	/// The collection may reserve more space to speculatively avoid frequent reallocations. After calling `reserve`,
	/// capacity will be greater than or equal to `self.len() + additional`. Does nothing if capacity is already sufficient.
	///
	/// See [`Vec::reserve`] for more information.
	pub fn reserve(&mut self, additional: usize) {
		self.s.reserve(additional)
	}
	/// Reserves the minimum capacity for at least `additional` more elements to be inserted in the given [`BVec<T>`].
	/// Unlike reserve, this will not deliberately over-allocate to speculatively avoid frequent allocations.
	/// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`.
	/// Does nothing if the capacity is already sufficient.
	///
	/// See [`Vec::reserve_exact`] for more information.
	pub fn reserve_exact(&mut self, additional: usize) {
		self.s.reserve_exact(additional)
	}
	/// Retains only the elements specified by the predicate.
	///
	/// See [`Vec::retain`] for more information.
	pub fn retain(&mut self, f: impl FnMut(&T) -> bool) {
		self.s.retain(f)
	}
	/// Retains only the elements specified by the predicate, passing a mutable reference to it.
	///
	/// See [`Vec::retain_mut`] for more information.
	pub fn retain_mut(&mut self, f: impl FnMut(&mut T) -> bool) {
		self.s.retain_mut(f)
	}
	/// Shrinks the capacity of the vector with a lower bound.
	///
	/// See [`Vec::shrink_to`] for more information.
	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.s.shrink_to(min_capacity)
	}
	/// Shrinks the capacity of the vector as much as possible.
	///
	/// See [`Vec::shrink_to_fit`] for more information.
	pub fn shrink_to_fit(&mut self) {
		self.s.shrink_to_fit()
	}
	/// Returns the remaining spare capacity of the vector as a slice of `MaybeUninit<T>`.
	///
	/// See [`Vec::shrink_to_fit`] for more information.
	pub fn spare_capacity_mut(&mut self) -> &mut [std::mem::MaybeUninit<T>] {
		self.s.spare_capacity_mut()
	}
	/// Creates a splicing iterator that replaces the specified range in the vector with the
	/// given replace_with iterator and yields the removed items. replace_with does not need to be the same length as range.
	///
	/// See [`Vec::splice`] for more information.
	pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> std::vec::Splice<I::IntoIter>
	where
		R: RangeBounds<usize>,
		I: IntoIterator<Item = T>,
	{
		self.s.splice(range, replace_with)
	}
	/// Splits the collection into two at the given index.
	///
	/// See [`Vec::split_off`] for more information.
	pub fn split_off(&mut self, idx: usize) -> BVec<T, MAX> {
		unsafe { BVec::from_vec_unchecked(self.s.split_off(idx)) }
	}
	/// Removes an element from the vector and returns it.
	///
	/// See [`Vec::swap_remove`] for more information.
	pub fn swap_remove(&mut self, idx: usize) -> T {
		self.s.swap_remove(idx)
	}
	/// Shortens the vector, keeping the first len elements and dropping the rest.
	///
	/// See [`Vec::truncate`] for more information.
	pub fn truncate(&mut self, len: usize) {
		self.s.truncate(len)
	}
	/// Tries to reserve capacity for at least `additional` more elements to be inserted in the given [`BVec<T>`].
	/// The collection may reserve more space to speculatively avoid frequent reallocations.
	/// After calling `try_reserve`, capacity will be greater than or equal to `self.len() + additional` if it
	/// returns `Ok(())`. Does nothing if capacity is already sufficient. This method preserves the contents even if an error occurs.
	///
	/// See [`Vec::try_reserve`] for more information.
	pub fn try_reserve(
		&mut self,
		additional: usize,
	) -> Result<(), std::collections::TryReserveError> {
		self.s.try_reserve(additional)
	}
	/// Tries to reserve the minimum capacity for at least `additional` elements to be inserted in the given `BVec<T>`.
	/// Unlike `try_reserve`, this will not deliberately over-allocate to speculatively avoid frequent allocations.
	/// After calling `try_reserve_exact`, capacity will be greater than or equal to `self.len() + additional` if it returns `Ok(())`.
	/// Does nothing if the capacity is already sufficient.
	///
	/// See [`Vec::try_reserve_exact`] for more information.
	pub fn try_reserve_exact(
		&mut self,
		additional: usize,
	) -> Result<(), std::collections::TryReserveError> {
		self.s.try_reserve_exact(additional)
	}
	/// Constructs a new, empty [`BVec<T>`] with at least the specified capacity.
	///
	/// See [`Vec::with_capacity`] for more information.
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			s: Vec::with_capacity(capacity),
		}
	}
}
impl<T: PartialEq, const MAX: usize> BVec<T, MAX> {
	/// Removes consecutive repeated elements in the vector according to the [`PartialEq`] trait implementation.
	///
	/// See [`Vec::dedup`] for more information.
	pub fn dedup(&mut self) {
		self.s.dedup()
	}
}

// Trait implementations relating BStr and BString
//////////////////////////////////////////////////

impl<T, const MAX: usize> Deref for BVec<T, MAX> {
	type Target = BSlice<T, MAX>;

	fn deref(&self) -> &Self::Target {
		unsafe { BSlice::from_slice_unchecked(&self.s) }
	}
}
impl<T, const MAX: usize> DerefMut for BVec<T, MAX> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { BSlice::from_slice_mut_unchecked(&mut self.s) }
	}
}
impl<T, const MAX: usize> Borrow<BSlice<T, MAX>> for BVec<T, MAX> {
	fn borrow(&self) -> &BSlice<T, MAX> {
		&**self
	}
}
impl<T, const MAX: usize> AsRef<BSlice<T, MAX>> for BVec<T, MAX> {
	fn as_ref(&self) -> &BSlice<T, MAX> {
		self
	}
}
impl<T: Clone, const MAX: usize> From<BVec<T, MAX>> for Box<BSlice<T, MAX>> {
	fn from(value: BVec<T, MAX>) -> Self {
		let b = Box::<[T]>::from(value.s);

		unsafe { Box::from_raw(Box::into_raw(b) as *mut BSlice<T, MAX>) }
	}
}
impl<T: PartialEq, const MAX1: usize, const MAX2: usize> PartialEq<BSlice<T, MAX2>>
	for BVec<T, MAX1>
{
	fn eq(&self, other: &BSlice<T, MAX2>) -> bool {
		(***self).eq(&**other)
	}
}
impl<T: PartialEq, const MAX1: usize, const MAX2: usize> PartialEq<&BSlice<T, MAX2>>
	for BVec<T, MAX1>
{
	fn eq(&self, other: &&BSlice<T, MAX2>) -> bool {
		(***self).eq(&***other)
	}
}
impl<T: PartialEq, const MAX1: usize, const MAX2: usize> PartialEq<&mut BSlice<T, MAX2>>
	for BVec<T, MAX1>
{
	fn eq(&self, other: &&mut BSlice<T, MAX2>) -> bool {
		(***self).eq(&***other)
	}
}

// Trait implementations mirroring standard String
//////////////////////////////////////////////////
