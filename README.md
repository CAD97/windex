# indexing

"Sound unchecked indexing" in Rust using "generativity"
(branding by unique lifetime parameter).

This is a partial fork of https://github.com/bluss/indexing,
with most of the code directly inspired from such.

This fork is immutable only, doesn't offer pointer indices/ranges, and doesn't
allow manipulating the indices/ranges without a reference to the container. In
exchange for these limitations, it works for fully sound fully unchecked
indexing of Rust's UTF-8 string types.
