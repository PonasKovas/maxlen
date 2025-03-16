use std::{
	borrow::Borrow,
	ops::{Deref, DerefMut},
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
