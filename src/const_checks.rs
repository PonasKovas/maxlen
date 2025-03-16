pub trait AssertGe {
	const VALID: ();
}

pub struct Pair<const A: usize, const B: usize>;
impl<const A: usize, const B: usize> AssertGe for Pair<A, B> {
	const VALID: () = assert!(A >= B);
}
