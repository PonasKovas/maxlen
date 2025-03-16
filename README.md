# maxlen

Length-bounded wrappers over standard types.

This crate provides several types that enforce length limits at the type level:
- `BSlice<T, MAX>` is a `[T]` but guaranteed to not have more than `MAX` elements.
- `BVec<T, MAX>` is a `Vec<T>` but guaranteed to not have more than `MAX` elements.
- `BStr<MAX, E>` is a `str` but guarantees that the text will not be longer than `MAX` bytes in the specified encoding (not necessarily UTF-8).
- `BString<MAX, E>` is a `String` but guarantees that the text will not be longer than `MAX` bytes in the specified encoding (not necessarily UTF-8).
