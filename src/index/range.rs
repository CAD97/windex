#[cfg(feature = "doc")]
use crate::scope;
use {
    crate::{
        Container, Index,
        proof::{NonEmpty, ProofAdd, Unknown},
        traits::{Idx, TrustedContainer},
    },
    core::{
        cmp, fmt,
        hash::{self, Hash},
        marker::PhantomData,
        ops,
    },
};

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
        self.start.untrusted()..self.end.untrusted()
    }

    /// This range without the emptiness proof.
    pub fn erased(&self) -> Range<'id, I, Unknown> {
        Range::from(self.start(), self.end())
    }

    /// The length of the range.
    pub fn len(&self) -> I {
        if self.is_empty() {
            I::zero()
        } else {
            self.end.untrusted().sub(self.start.untrusted().as_usize())
        }
    }

    /// `true` if the range is empty.
    pub fn is_empty(&self) -> bool {
        self.start.untrusted() >= self.end.untrusted()
    }

    /// Try to create a proof that the range is nonempty.
    pub fn nonempty(&self) -> Option<Range<'id, I, NonEmpty>> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(Range::new_nonempty(self.start.untrusted(), self.end.untrusted())) }
        }
    }

    /// The starting index. (Accessible if the range is `NonEmpty`.)
    pub fn start(&self) -> Index<'id, I, Emptiness> {
        unsafe { Index::new_any(self.start.untrusted()) }
    }

    /// The ending index.
    pub fn end(&self) -> Index<'id, I, Unknown> {
        self.end
    }

    /// Split around the middle `index` if it is in this range,
    /// such that the second range contains the index.
    pub fn split_at<P>(
        &self,
        index: Index<'id, I, P>,
    ) -> Option<(Range<'id, I, Unknown>, Range<'id, I, P>)> {
        if index >= self.start && index <= self.end {
            Some((Range::from(self.start(), index), unsafe {
                Range::new_any(index.untrusted(), self.end.untrusted())
            }))
        } else {
            None
        }
    }

    /// If the index is a valid absolute index within this range.
    pub fn contains_in<Array: ?Sized + TrustedContainer>(
        &self,
        index: I,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        if index >= self.start().untrusted() && index < self.end().untrusted() {
            // we need a full vet to check that we're on an item index
            container.vet(index).ok()
        } else {
            None
        }
    }

    /// If the index is within this range. Provides a nonempty proof.
    pub fn contains<P>(&self, index: Index<'id, I, P>) -> Option<Index<'id, I, NonEmpty>> {
        index.in_range(*self)
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
            unsafe { Some(Range::new_any(self.start.untrusted(), other.end.untrusted())) }
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
        unsafe { Range::new_any(start.untrusted(), end.untrusted()) }
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
    pub fn advance_in<Array: ?Sized + TrustedContainer>(
        &mut self,
        container: &Container<'id, Array>,
    ) -> bool {
        if let Some(next) = container.advance(self.start()) {
            if next < self.end() {
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
            .field(&self.start.untrusted())
            .field(&self.end.untrusted())
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
impl<'id, I: Idx, P, Q> PartialEq<Range<'id, I, Q>> for Range<'id, I, P> {
    fn eq(&self, other: &Range<'id, I, Q>) -> bool {
        self.start.eq(&other.start) && self.end.eq(&other.end)
    }
}

impl<'id, I: Idx, Emptiness> Hash for Range<'id, I, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}
