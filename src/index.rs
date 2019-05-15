#[cfg(feature = "doc")]
use crate::scope;
use {
    crate::{
        container::Container,
        proof::{Id, NonEmpty, ProofAdd, Unknown},
        traits::{Idx, TrustedContainer},
    },
    core::{
        cmp, fmt,
        hash::{self, Hash},
        marker::PhantomData,
        ops,
    },
};

/// A branded index.
///
/// `Index<'id>` only indexes the container instantiated with the exact same
/// lifetime for the parameter `'id` created by the [`scope`] function.
///
/// The type parameter `Emptiness` determines if the index is followable.
/// A `NonEmpty` index points to a valid element. An `Unknown` index is
/// unknown, or it points to an edge index (one past the end).
pub struct Index<'id, I: Idx = u32, Emptiness = NonEmpty> {
    #[allow(unused)]
    id: Id<'id>,
    idx: I,
    phantom: PhantomData<Emptiness>,
}

impl<'id, I: Idx> Index<'id, I, Unknown> {
    pub(crate) unsafe fn new(idx: I) -> Self {
        Index::new_any(idx)
    }
}

impl<'id, I: Idx> Index<'id, I, NonEmpty> {
    pub(crate) unsafe fn new_nonempty(idx: I) -> Self {
        Index::new_any(idx)
    }
}

impl<'id, I: Idx, Emptiness> Index<'id, I, Emptiness> {
    pub(crate) unsafe fn new_any(idx: I) -> Self {
        Index {
            id: Id::default(),
            idx,
            phantom: PhantomData,
        }
    }
}

impl<'id, I: Idx, Emptiness> Index<'id, I, Emptiness> {
    /// This index without the branding.
    pub fn untrusted(&self) -> I {
        self.idx
    }

    /// This index without the emptiness proof.
    pub fn erased(&self) -> Index<'id, I, Unknown> {
        unsafe { Index::new(self.idx) }
    }
}

impl<'id, I: Idx> Index<'id, I, NonEmpty> {
    #[doc(hidden)]
    pub fn observe_proof(&self) {}
}

impl<'id, I: Idx, Emptiness> fmt::Debug for Index<'id, I, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Index<'id>").field(&self.idx).finish()
    }
}

impl<'id, I: Idx, Emptiness> Copy for Index<'id, I, Emptiness> {}
impl<'id, I: Idx, Emptiness> Clone for Index<'id, I, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, I: Idx, Emptiness> Ord for Index<'id, I, Emptiness> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl<'id, I: Idx, Emptiness> PartialOrd for Index<'id, I, Emptiness> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl<'id, I: Idx, Emptiness> Eq for Index<'id, I, Emptiness> {}
impl<'id, I: Idx, Emptiness> PartialEq for Index<'id, I, Emptiness> {
    fn eq(&self, other: &Self) -> bool {
        self.idx.eq(&other.idx)
    }
}

impl<'id, I: Idx, Emptiness> Hash for Index<'id, I, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}

/// A branded range.
///
/// `Range<'id>` only indexes the container instantiated with the exact same
/// lifetime for the parameter `'id` created by the [`scope`] function.
///
/// The range may carry a proof of non-emptiness (`Emptiness`),
/// which enables further operations.
pub struct Range<'id, I: Idx = u32, Emptiness = Unknown> {
    start: Index<'id, I, Unknown>,
    end: Index<'id, I, Unknown>,
    phantom: PhantomData<Emptiness>,
}

impl<'id, I: Idx> Range<'id, I, Unknown> {
    pub(crate) unsafe fn new(start: I, end: I) -> Self {
        Range::new_any(start, end)
    }
}

impl<'id, I: Idx> Range<'id, I, NonEmpty> {
    pub(crate) unsafe fn new_nonempty(start: I, end: I) -> Self {
        Range::new_any(start, end)
    }
}

impl<'id, I: Idx, Emptiness> Range<'id, I, Emptiness> {
    pub(crate) unsafe fn new_any(start: I, end: I) -> Self {
        Range {
            start: Index::new(start),
            end: Index::new(end),
            phantom: PhantomData,
        }
    }
}

impl<'id, I: Idx> Range<'id, I, Unknown> {
    /// Construct a range from two trusted indices.
    pub fn from<P, Q>(start: Index<'id, I, P>, end: Index<'id, I, Q>) -> Self {
        unsafe { Range::new(start.untrusted(), end.untrusted()) }
    }
}

impl<'id, I: Idx, Emptiness> Range<'id, I, Emptiness> {
    /// This range without the branding.
    pub fn untrusted(&self) -> ops::Range<I> {
        self.start.idx..self.end.idx
    }

    /// This range without the emptiness proof.
    pub fn erased(&self) -> Range<'id, I, Unknown> {
        unsafe { Range::new(self.start.idx, self.end.idx) }
    }

    /// The length of the range.
    pub fn len(&self) -> I {
        if self.is_empty() {
            I::zero()
        } else {
            self.end.idx.sub(self.start.idx.as_usize())
        }
    }

    /// `true` if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.start.idx >= self.end.idx
    }

    /// Try to create a proof that the range is nonempty.
    pub fn nonempty(&self) -> Option<Range<'id, I, NonEmpty>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(Range::new_nonempty(self.start.idx, self.end.idx)) }
        }
    }

    /// The starting index. (Accessible if the range is `NonEmpty`.)
    pub fn start(&self) -> Index<'id, I, Emptiness> {
        unsafe { Index::new_any(self.start.idx) }
    }

    /// The ending index.
    pub fn end(&self) -> Index<'id, I, Unknown> {
        self.end
    }

    /// Split around the middle `index` if it is in this range.
    pub fn split_at<E>(&self, index: Index<'id, I, E>) -> Option<(Range<'id, I>, Range<'id, I>)> {
        let index = index.erased();
        if index >= self.start && index <= self.end {
            unsafe {
                Some((
                    Range::new(self.start.idx, index.idx),
                    Range::new(index.idx, self.end.idx),
                ))
            }
        } else {
            None
        }
    }

    /// If the index is a valid absolute index within this range.
    pub fn contains_in<Array: TrustedContainer>(
        &self,
        index: I,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        if index >= self.start().untrusted() && index < self.end().untrusted() {
            container.vet(index).ok()
        } else {
            None
        }
    }

    /// If the index is within this range. Provides a nonempty proof.
    pub fn contains<P>(&self, index: Index<'id, I, P>) -> Option<Index<'id, I, NonEmpty>> {
        if index.erased() >= self.start().erased() && index.erased() < self.end() {
            unsafe { Some(Index::new_nonempty(index.untrusted())) }
        } else {
            None
        }
    }

    /// Join together two adjacent ranges.
    /// (They must be exactly touching, non-overlapping, and in order.)
    pub fn join<Q>(
        &self,
        other: Range<'id, I, Q>,
    ) -> Option<Range<'id, I, <(Emptiness, Q) as ProofAdd>::Sum>>
    where
        (Emptiness, Q): ProofAdd,
    {
        if self.end == other.start {
            unsafe { Some(Range::new_any(self.start.idx, other.end.idx)) }
        } else {
            None
        }
    }

    /// Extend the range to cover all of `other`, including any space between.
    pub fn join_cover<Q>(
        &self,
        other: Range<'id, I, Q>,
    ) -> Range<'id, I, <(Emptiness, Q) as ProofAdd>::Sum>
    where
        (Emptiness, Q): ProofAdd,
    {
        let start = cmp::min(self.start, other.start);
        let end = cmp::max(self.end, other.end);
        unsafe { Range::new_any(start.idx, end.idx) }
    }

    /// Create two empty ranges, at the front and the back of this range.
    pub fn frontiers(&self) -> (Range<'id, I, Unknown>, Range<'id, I, Unknown>) {
        (
            Range::from(self.start(), self.start()),
            Range::from(self.end(), self.end()),
        )
    }
}

impl<'id, I: Idx> Range<'id, I, NonEmpty> {
    #[doc(hidden)]
    pub fn observe_proof(&self) {}

    /// Increase the range's start, if the result is still a non-empty range.
    ///
    /// `true` if stepped successfully, `false` if the range would be empty.
    pub fn advance_in<Array: TrustedContainer>(
        &mut self,
        container: &Container<'id, Array>,
    ) -> bool {
        if let Some(next) = container.advance(self.start()) {
            if next.erased() < self.end() {
                *self = unsafe { Range::new_nonempty(next.untrusted(), self.end().untrusted()) };
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// # Note
///
/// In order to use this impl, you'll likely need to erase the emptiness proof
/// from the indices to create the range. This doesn't lose any `Range` proof.
impl<'id, I: Idx, Emptiness> From<ops::Range<Index<'id, I, Emptiness>>> for Range<'id, I, Unknown> {
    fn from(r: ops::Range<Index<'id, I, Emptiness>>) -> Self {
        Range::from(r.start, r.end)
    }
}

impl<'id, I: Idx, Emptiness> fmt::Debug for Range<'id, I, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Range<'id>")
            .field(&self.start.idx)
            .field(&self.end.idx)
            .finish()
    }
}

impl<'id, I: Idx, Emptiness> Copy for Range<'id, I, Emptiness> {}
impl<'id, I: Idx, Emptiness> Clone for Range<'id, I, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, I: Idx, Emptiness> Eq for Range<'id, I, Emptiness> {}
impl<'id, I: Idx, Emptiness> PartialEq for Range<'id, I, Emptiness> {
    fn eq(&self, other: &Self) -> bool {
        self.start.eq(&other.start) && self.end.eq(&other.end)
    }
}

impl<'id, I: Idx, Emptiness> Hash for Range<'id, I, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

/// The error returned when failing to construct an arbitrary index.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IndexError {
    /// The provided raw index was out of bounds of the container.
    OutOfBounds,
    /// The provided raw index was in bounds but not on an item border.
    Invalid,
}
