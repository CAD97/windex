use {
    crate::{
        index::{Index, IndexError, Range},
        proof::{Id, NonEmpty, Unknown},
        traits::{Idx, TrustedContainer, TrustedItem},
    },
    core::{fmt, ops},
};

/// A branded container, that allows access only to indices and ranges with
/// the exact same brand in the `'id` parameter.
///
/// The elements in the underlying data structure are accessible partly
/// through special purpose methods, and through indexing/slicing.
///
/// The container can be indexed with `self[i]` where `i` is a trusted,
/// dereferenceable index or range. Indexing like this uses _no_ runtime
/// checking at all, as it is statically guaranteed correct.
#[repr(transparent)]
pub struct Container<'id, Array: TrustedContainer + ?Sized> {
    #[allow(unused)]
    id: Id<'id>,
    array: Array,
}

impl<'id, Array: TrustedContainer> Container<'id, Array> {
    pub(crate) unsafe fn new(array: Array) -> Self {
        Container {
            id: Id::default(),
            array,
        }
    }
}
impl<'id, Array: TrustedContainer + ?Sized> Container<'id, Array> {
    /// This container without the branding.
    // FUTURE: Can this return `&'id Array`?
    pub fn untrusted(&self) -> &Array {
        &self.array
    }

    /// The length of the container in base item units.
    pub fn unit_len(&self) -> usize {
        self.array.unit_len()
    }

    /// The zero index without a proof of contents.
    pub fn start<I: Idx>(&self) -> Index<'id, I, Unknown> {
        unsafe { Index::new(I::zero()) }
    }

    /// The index one past the end of this container.
    pub fn end<I: Idx>(&self) -> Index<'id, I, Unknown> {
        let len = I::from_usize(self.unit_len()).expect("len");
        unsafe { Index::new(len) }
    }

    /// The empty range `0..0`.
    pub fn empty_range<I: Idx>(&self) -> Range<'id, I, Unknown> {
        Range::from(self.start(), self.start())
    }

    /// The full range of the container.
    pub fn range<I: Idx>(&self) -> Range<'id, I, Unknown> {
        Range::from(self.start(), self.end())
    }

    /// Vet an absolute index.
    pub fn vet<I: Idx>(&self, idx: I) -> Result<Index<'id, I, NonEmpty>, IndexError> {
        // TrustedItem::vet doesn't assert that it's not at the end
        let item = TrustedItem::vet(idx, self)?;
        if item < self.end() {
            unsafe { Ok(Index::new_nonempty(item.untrusted())) }
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    /// Vet an absolute range.
    // Future: Error type `EitherOrBoth<IndexError, IndexError>`?
    pub fn vet_range<I: Idx>(
        &self,
        r: ops::Range<I>,
    ) -> Result<Range<'id, I, Unknown>, IndexError> {
        Ok(Range::from(
            TrustedItem::vet(r.start, self)?,
            TrustedItem::vet(r.end, self)?,
        ))
    }

    /// Split the container in two at the given index,
    /// such that the second range contains the index.
    pub fn split_at<I: Idx, P>(
        &self,
        idx: Index<'id, I, P>,
    ) -> (Range<'id, I, Unknown>, Range<'id, I, P>) {
        (Range::from(self.start(), idx), unsafe {
            Range::new_any(idx.untrusted(), self.end().untrusted())
        })
    }

    /// Split the container in two after the given index,
    /// such that the first range contains the index.
    pub fn split_after<I: Idx>(
        &self,
        idx: Index<'id, I, NonEmpty>,
    ) -> (Range<'id, I, NonEmpty>, Range<'id, I, Unknown>) {
        let mid = TrustedItem::after(idx, self);
        (
            unsafe { Range::new_nonempty(I::zero(), mid.untrusted()) },
            Range::from(mid, self.end()),
        )
    }

    /// Split around the range `r` creating ranges `0..r.start` and `r.end..`.
    ///
    /// The input `r` and return values `(s, t)` cover teh whole container in
    /// the order `s`, `r`, `t`.
    pub fn split_around<I: Idx, P>(
        &self,
        r: Range<'id, I, P>,
    ) -> (Range<'id, I, Unknown>, Range<'id, I, Unknown>) {
        (
            Range::from(self.start(), r.start()),
            Range::from(r.end(), self.end()),
        )
    }

    /// Return the range before but not including the index.
    pub fn before<I: Idx, P>(&self, idx: Index<'id, I, P>) -> Range<'id, I, Unknown> {
        Range::from(self.start(), idx)
    }

    /// Return the range after but not including the index.
    pub fn after<I: Idx>(&self, idx: Index<'id, I, NonEmpty>) -> Range<'id, I, Unknown> {
        let after = TrustedItem::after(idx, self);
        Range::from(after, self.end())
    }

    /// Advance an index to the next item in the container, if there is one.
    pub fn advance<I: Idx>(&self, idx: Index<'id, I, NonEmpty>) -> Option<Index<'id, I, NonEmpty>> {
        TrustedItem::advance(idx, self)
    }

    /// Advance an index by a given base unit offset,
    /// if the index at said offset is a valid item index.
    pub fn advance_by<I: Idx, P>(
        &self,
        idx: Index<'id, I, P>,
        offset: usize,
    ) -> Result<Index<'id, I, NonEmpty>, IndexError> {
        self.vet(idx.untrusted().add(offset))
    }

    /// Decrease an index by a given base unit offset,
    /// if the index at said offset is a valid item index.
    pub fn decrease_by<I: Idx, P>(
        &self,
        idx: Index<'id, I, P>,
        offset: usize,
    ) -> Result<Index<'id, I, NonEmpty>, IndexError> {
        if idx.untrusted().as_usize() >= offset {
            self.vet(idx.untrusted().sub(offset))
        } else {
            Err(IndexError::OutOfBounds)
        }
    }
}

impl<'id, Array: TrustedContainer + ?Sized, I: Idx> ops::Index<Index<'id, I, NonEmpty>>
    for Container<'id, Array>
{
    type Output = Array::Item;

    fn index(&self, index: Index<'id, I, NonEmpty>) -> &Self::Output {
        unsafe { self.array.get_unchecked(index.untrusted().as_usize()) }
    }
}

impl<'id, Array: TrustedContainer + ?Sized, I: Idx, P> ops::Index<Range<'id, I, P>>
    for Container<'id, Array>
{
    type Output = Array::Slice;

    fn index(&self, r: Range<'id, I, P>) -> &Self::Output {
        unsafe {
            self.array
                .slice_unchecked(r.start().untrusted().as_usize()..r.end().untrusted().as_usize())
        }
    }
}

impl<'id, Array: TrustedContainer + ?Sized, I: Idx, P> ops::Index<ops::RangeFrom<Index<'id, I, P>>>
    for Container<'id, Array>
{
    type Output = Array::Slice;

    fn index(&self, r: ops::RangeFrom<Index<'id, I, P>>) -> &Self::Output {
        &self[Range::from(r.start, self.end())]
    }
}

impl<'id, Array: TrustedContainer + ?Sized, I: Idx, P> ops::Index<ops::RangeTo<Index<'id, I, P>>>
    for Container<'id, Array>
{
    type Output = Array::Slice;

    fn index(&self, r: ops::RangeTo<Index<'id, I, P>>) -> &Self::Output {
        &self[Range::from(self.start(), r.end)]
    }
}

impl<'id, Array: TrustedContainer + ?Sized> ops::Index<ops::RangeFull> for Container<'id, Array> {
    type Output = Array::Slice;

    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        &self[Range::from(self.start(), self.end())]
    }
}

impl<'id, Array: TrustedContainer + Copy> Copy for Container<'id, Array> {}
impl<'id, Array: TrustedContainer + Clone> Clone for Container<'id, Array> {
    fn clone(&self) -> Self {
        unsafe { Container::new(self.array.clone()) }
    }
}

impl<'id, Array: TrustedContainer + fmt::Debug + ?Sized> fmt::Debug for Container<'id, Array> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Container<'id>").field(&&self.array).finish()
    }
}
