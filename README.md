# indexing

"Sound unchecked indexing" in Rust using "generativity"
(branding by unique lifetime parameter).

> You seemed to want guarantees stronger than I thought possible.  
> \- eddyb

Well, they're here!

This is a rewritten fork of https://github.com/bluss/indexing, with the
core design of generativity and API beats lovingly borrowed from thence.

This fork offers sound unchecked indexing for string slices as well as
normal array slices, and avoids some [pitfalls][bluss/indexing#11] that
the original library fell victim to, just by virtue of being four years
old. It also updates the API slightly (so it's not a drop-in replacement,
sorry) to more align with the author's API design ideals.

  [bluss/indexing#11]: <https://github.com/bluss/indexing/issues/11>
