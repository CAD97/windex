#[cfg(feature = "doc")]
use crate::{scope, scope_ref};
use {
    crate::{
        proof::{Id, NonEmpty, Unknown},
        traits::{TrustedContainer, TrustedItem},
        Index, IndexError, Range,
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
pub struct Container<'id, Array: ?Sized>
where
    Array: TrustedContainer,
{
    #[allow(unused)]
    id: Id<'id>,
    array: Array,
}

impl<'id, Array> Container<'id, Array>
where
    Array: TrustedContainer,
{
    pub(crate) unsafe fn new(array: Array) -> Self {
        Container {
            id: Id::default(),
            array,
        }
    }
}

impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    /// This container without the branding.
    ///
    /// # Note
    ///
    /// The returned lifetime of `&Array` is _not_ `'id`! It's completely
    /// valid to drop the container during the [`scope`], in which case this
    /// reference would become invalid. If you need a longer lifetime,
    /// consider using [`scope_ref`] such that the reference is guaranteed to
    /// live for the entire scope.
    pub fn untrusted(&self) -> &Array {
        &self.array
    }

    /// The length of the container in base item units.
    pub fn unit_len(&self) -> u32 {
        self.array.unit_len()
    }

    /// The zero index without a proof of contents.
    pub fn start(&self) -> Index<'id, Unknown> {
        unsafe { Index::new(0) }
    }

    /// The index one past the end of this container.
    pub fn end(&self) -> Index<'id, Unknown> {
        unsafe { Index::new(self.unit_len()) }
    }

    /// The empty range `0..0`.
    pub fn empty_range(&self) -> Range<'id, Unknown> {
        Range::from(self.start(), self.start())
    }

    /// The full range of the container.
    pub fn range(&self) -> Range<'id, Unknown> {
        Range::from(self.start(), self.end())
    }

    /// Vet an absolute index.
    pub fn vet(&self, idx: u32) -> Result<Index<'id, NonEmpty>, IndexError> {
        if idx < self.end().untrusted() {
            unsafe { TrustedItem::vet_inbounds(idx, self).ok_or(IndexError::Invalid) }
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    /// Vet an absolute range.
    // Future: Error type `EitherOrBoth<IndexError, IndexError>`?
    pub fn vet_range(&self, r: ops::Range<u32>) -> Result<Range<'id, Unknown>, IndexError> {
        Ok(Range::from(
            TrustedItem::vet(r.start, self)?,
            TrustedItem::vet(r.end, self)?,
        ))
    }

    /// Split the container in two at the given index,
    /// such that the second range contains the index.
    pub fn split_at<P>(&self, idx: Index<'id, P>) -> (Range<'id, Unknown>, Range<'id, P>) {
        (self.before(idx), self.after_inclusive(idx))
    }

    /// Split the container in two after the given index,
    /// such that the first range contains the index.
    pub fn split_after(
        &self,
        idx: Index<'id, NonEmpty>,
    ) -> (Range<'id, NonEmpty>, Range<'id, Unknown>) {
        (self.before_inclusive(idx), self.after(idx))
    }

    /// Split around the range `r` creating ranges `0..r.start` and `r.end..`.
    ///
    /// The input `r` and return values `(s, t)` cover the whole container in
    /// the order `s`, `r`, `t`.
    pub fn split_around<P>(&self, r: Range<'id, P>) -> (Range<'id, Unknown>, Range<'id, Unknown>) {
        (self.before(r.start()), self.after_inclusive(r.end()))
    }

    /// Return the range before but not including the index.
    pub fn before<P>(&self, idx: Index<'id, P>) -> Range<'id, Unknown> {
        Range::from(self.start(), idx)
    }

    /// Return the range before the index, inclusive.
    pub fn before_inclusive(&self, idx: Index<'id, NonEmpty>) -> Range<'id, NonEmpty> {
        let after = TrustedItem::after(idx, self);
        unsafe { Range::new_nonempty(self.start().untrusted(), after.untrusted()) }
    }

    /// Return the range after but not including the index.
    pub fn after(&self, idx: Index<'id, NonEmpty>) -> Range<'id, Unknown> {
        let after = TrustedItem::after(idx, self);
        Range::from(after, self.end())
    }

    /// Return the range after the index, inclusive.
    pub fn after_inclusive<P>(&self, idx: Index<'id, P>) -> Range<'id, P> {
        unsafe { Range::new_any(idx.untrusted(), self.end().untrusted()) }
    }

    /// Advance an index to the next item in the container, if there is one.
    pub fn advance(&self, idx: Index<'id, NonEmpty>) -> Option<Index<'id, NonEmpty>> {
        TrustedItem::advance(idx, self)
    }

    /// Advance an index by a given base unit offset,
    /// if the index at said offset is a valid item index.
    pub fn advance_by<P>(
        &self,
        idx: Index<'id, P>,
        offset: u32,
    ) -> Result<Index<'id, NonEmpty>, IndexError> {
        self.vet(idx.untrusted() + offset)
    }

    /// Retreat an index to the prior item in the container, if there is one.
    pub fn retreat<P>(&self, idx: Index<'id, P>) -> Option<Index<'id, NonEmpty>> {
        TrustedItem::before(idx, self)
    }

    /// Decrease an index by a given base unit offset,
    /// if the index at said offset is a valid item index.
    pub fn decrease_by<P>(
        &self,
        idx: Index<'id, P>,
        offset: u32,
    ) -> Result<Index<'id, NonEmpty>, IndexError> {
        if idx.untrusted() >= offset {
            self.vet(idx.untrusted() - offset)
        } else {
            Err(IndexError::OutOfBounds)
        }
    }
}

impl<'id, Array: ?Sized, D> ops::Deref for Container<'id, D>
where
    Array: TrustedContainer,
    D: TrustedContainer + ops::Deref<Target = Array>,
{
    type Target = Container<'id, Array>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(&*self.array as *const Array as *const Container<'id, Array>) }
    }
}

impl<'id, Array: ?Sized, D> ops::DerefMut for Container<'id, D>
where
    Array: TrustedContainer,
    D: TrustedContainer + ops::DerefMut<Target = Array>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(&mut *self.array as *mut Array as *mut Container<'id, Array>) }
    }
}

impl<'id, Array: ?Sized> ops::Index<Index<'id, NonEmpty>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Item;

    fn index(&self, index: Index<'id, NonEmpty>) -> &Self::Output {
        unsafe { self.array.get_unchecked(index.untrusted() as usize) }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<Range<'id, P>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, r: Range<'id, P>) -> &Self::Output {
        unsafe {
            self.array
                .slice_unchecked(r.start().untrusted() as usize..r.end().untrusted() as usize)
        }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeFrom<Index<'id, P>>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, r: ops::RangeFrom<Index<'id, P>>) -> &Self::Output {
        &self[self.after_inclusive(r.start)]
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeTo<Index<'id, P>>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, r: ops::RangeTo<Index<'id, P>>) -> &Self::Output {
        &self[self.before(r.end)]
    }
}

impl<'id, Array: ?Sized> ops::Index<ops::RangeFull> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, _: ops::RangeFull) -> &Self::Output {
        &self[self.range()]
    }
}

impl<'id, Array: Copy> Copy for Container<'id, Array> where Array: TrustedContainer {}

impl<'id, Array: Clone> Clone for Container<'id, Array>
where
    Array: TrustedContainer,
{
    fn clone(&self) -> Self {
        unsafe { Container::new(self.array.clone()) }
    }
}

impl<'id, Array: ?Sized + fmt::Debug> fmt::Debug for Container<'id, Array>
where
    Array: TrustedContainer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Container<'id>").field(&&self.array).finish()
    }
}
