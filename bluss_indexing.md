This file details the translation from bluss/indexing to cad97/windex.
Not every function has a direct translation, but all functionality
should be available in some form. Pointer particles are omitted.
(In fact, I'm convinced they're not sound in the current Stacked Borrows
proposal, but that may change, as the issue is identical to `Pin<&mut>`
as it's "upgrading" a pointer/reference by having the parent `&mut` lock.)

# `Range`

## Compatible with `perfect::Range` without a `Container` reference

- `Range<P>`
  - `len(&self) -> usize` ⟹ `len(self) -> u32`
  - `is_empty(&self) -> bool` ⟹ `is_empty(self) -> bool`
  - `nonempty(&self) -> Result<Range<NonEmpty>, _>` ⟹
    `nonempty(self) -> Option<Range<NonEmpty>>`
  - `start(&self) -> usize` ⟹
    `start(self) -> Index` ▷ `untrusted(self) -> u32` or  
    `untrusted(self) -> ops::Range<u32>` ▷ `.start: u32`
  - `end(&self) -> usize` ⟹
    `end(self) -> Index` ▷ `untrusted(self) -> u32` or  
    `untrusted(self) -> ops::Range<u32>` ▷ `.end: u32`
  - `split_at(&self, usize) -> (Range<Unknown>, Range<Unknown>)` ⟹
    `split_at(self, Index) -> Option<(Range<Unknown>, Range<P>)>`
    (A simple range can vet the index itself!)
  - `contains(&self, usize) -> Option<Index<NonEmpty>>` ⟹
    `vet(self, u__) -> Option<Index<NonEmpty>>`
  - `join(&self, Range<Q>) -> Result<Range<P+Q, _>` ⟹
    `join(self, Range<Q>) -> Option<Range<P+Q>>`
  - `join_cover(&self, Range<Q>) -> Range<P+Q>` (unsound) ⟹
    `extend_end(self, Index) -> Range<P>`
  - `join_cover_both(&self, Range<Q>) -> Range<P+Q>` ⟹
    `join_cover(self, Range<Q>) -> Range<P+Q>`
  - `as_range(&self) -> ops::Range<usize>` ⟹
    `untrusted(self) -> ops::Range<u32>`
  - `frontiers(&self) -> (Range<Unknown>, Range<Unknown>)` ⟹
    `frontiers(self) -> (Range<Unknown>, Range<Unknown>)`
  - `no_proof(self) -> Range<Unknown>` ⟹ `untrusted(self) -> Range<Unknown>`
  - `first(&self) -> Index<P>` ⟹ `start(self) -> Index<P>`
  - `past_the_end(self) -> Index<Unknown>` ⟹ `end(self) -> Index<Unknown>`

## Compatible with `perfect::Range` with a `Container` reference

- `Range<P>`
  - `split_in_half(self)` -> `(Range<Unknown>, Range<P>)` ⟹ TODO
  - `subdivide(&self, usize) -> impl Iterator<Range<NonEmpty>>` ⟹ TODO
  - `forward_by(&self, &mut Index<NonEmpty>, usize) -> bool` ⟹ TODO
  - `forward_range_by(&self, Range, usize) -> Range<Unknown>` ⟹ TODO
  - `upper_middle(&self) -> Index<P>` ⟹ TODO

- `Range<NonEmpty>`
  - `lower_middle(&self) -> Index<NonEmpty>` ⟹ TODO
  - `last(&self) -> Index<NonEmpty>` ⟹ TODO
  - `tail(self) -> Range<Unknown>` ⟹ TODO
  - `init(&self) -> Range<Unknown>` ⟹ TODO
  - `advance(&mut self) -> bool` ⟹ TODO
  - `advance_by(&mut self, usize) -> bool` ⟹ TODO
  - `advance_back(&mut self) -> bool` ⟹ TODO

# `Index`

## Compatible with `perfect::Index` without a `Container` reference

- `Index<P>`
  - `integer(&self) -> usize` ⟹ `untrusted(self) -> u32`

## Compatible with `perfect::Index` with a `Container` reference

- `Index<NonEmpty>`
  - `after(self) -> Index<Unknown>` ⟹ TODO

# `Container`

- `Container<Array>`
  - `len(&self) -> usize` ⟹ `len(&self) -> u32`
  - `empty_range(&self) -> Range<Unknown>` ⟹ `default()`
  - `range(&self) -> Range<Unknown>` ⟹ `as_range(&self) -> Range<Unknown>`
  - `vet(&self, usize) -> Result<Index<NonEmpty>, _>` ⟹
    `vet(&self, u__) -> Result<Index<NonEmpty>, _>`
  - `vet_range(&self, ops::Range<usize>) -> Result<Range<Unknown>, _>` ⟹
    `vet(&self, ops::Range*<u__>) -> Result<Range<Unknown, _>`
  - `split_at(&self, Index<P>) -> (Range<Unknown>, Range<P>)` ⟹ TODO
  - `split_after(&self, Index<NonEmpty>) -> (Range<NonEmpty>, Range<Unknown>)` ⟹ TODO
  - `split_around(&self, Range) -> (Range<Unknown>, Range<Unknown>)` ⟹ TODO
  - `before(&self, Index) -> Range<Unkown>` ⟹ TODO
  - `after(&self, Index<NonEmpty>) -> Range<Unknown>` ⟹ TODO
  - `range_of(&self, impl OnePointRange) -> Range<Unkown>` ⟹ TODO
  - `forward(&self, &mut Index<NonEmpty>) -> bool` ⟹ TODO
  - `forward_by(&self, &mut Index<NonEmpty>, usize) -> bool` ⟹ TODO
  - `forward_range_by(&self, Range, usize) -> Range<Unknown>` ⟹ TODO
  - `backward(&self, &mut Index<NonEmpty>) -> bool` ⟹ TODO
  - `scan_from(&self, Index<NonEmpty>, impl FnMut) -> Range<NonEmpty>` ⟹ TODO
  - `scan_from_rev(&self, Index<NonEmpty>, impl FnMut) -> Range<NonEmpty>` ⟹ TODO
  - `scan_range(&self, Range, impl FnMut) -> (Range<Unknown>, Range<Unknown>)` ⟹ TODO
  - `swap(&mut self, Index<NonEmpty>, Index<NonEmpty>)` ⟹ TODO
  - `rotate1_up(&mut self, Range<NonEmpty>)` ⟹ TODO
  - `rotate1_down(&mut self, Range<NonEmpty>)` ⟹ TODO
  - `index_twice(&mut self, Range, Range) -> Result<(&mut [T], &mut [T]), _>` ⟹ TODO
  - `zip_mut_raw(&mut self, Range, Range, impl FnMut)` ⟹ TODO

- `Container<Array: Growable>`
  - `push(&mut self, T) -> Index<NonEmpty>` ⟹ TODO
  - `insert(&mut self, Index, T)` ⟹ TODO
