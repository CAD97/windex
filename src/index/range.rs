#[cfg(feature = "doc")]
use crate::scope;
use {
    crate::{
        proof::{NonEmpty, ProofAdd, Unknown},
        traits::TrustedContainer,
        Container, Index,
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
//
//  # Safety
//
//  The range must be non-decreasing even when Emptiness = Unknown; otherwise
//  we cannot safely do an unchecked index! When Emptiness = NonEmpty,
//  0 <= start < end <= len for the branded container; i.e. start is NonEmpty.
pub struct Range<'id, Emptiness = Unknown> {
    start: Index<'id, Unknown>,
    end: Index<'id, Unknown>,
    phantom: PhantomData<Emptiness>,
}

// ~~~ Private Helpers ~~~ //

impl<'id> Range<'id, Unknown> {
    pub(crate) unsafe fn new(start: u32, end: u32) -> Self {
        Range::new_any(start, end)
    }
}

impl<'id> Range<'id, NonEmpty> {
    pub(crate) unsafe fn new_nonempty(start: u32, end: u32) -> Self {
        Range::new_any(start, end)
    }
}

impl<'id, Emptiness> Range<'id, Emptiness> {
    pub(crate) unsafe fn new_any(start: u32, end: u32) -> Self {
        Range {
            start: Index::new(start),
            end: Index::new(end),
            phantom: PhantomData,
        }
    }

    pub(crate) fn from<P>(start: Index<'id, Emptiness>, end: Index<'id, P>) -> Self {
        unsafe { Range::new_any(start.untrusted(), end.untrusted()) }
    }

    pub(crate) unsafe fn trusted(self) -> Range<'id, NonEmpty> {
        Range::new_nonempty(self.start.untrusted(), self.end.untrusted())
    }
}

// ~~~ Public Constructors ~~~ //

// ~~~ Discarding Proofs ~~~ //

impl<'id, Emptiness> Range<'id, Emptiness> {
    /// This range without the branding.
    pub fn untrusted(self) -> ops::Range<u32> {
        self.start.untrusted()..self.end.untrusted()
    }

    /// This range without the emptiness proof.
    pub fn erased(self) -> Range<'id, Unknown> {
        Range::from(self.start().erased(), self.end())
    }
}

// ~~~ Gaining Proofs ~~~ //

// ~~~ Accessors ~~~ //

impl<'id, Emptiness> Range<'id, Emptiness> {
    /// The length of the range.
    pub fn len(self) -> u32 {
        if self.is_empty() {
            0
        } else {
            self.end.untrusted() - self.start.untrusted()
        }
    }

    /// `true` if the range is empty.
    pub fn is_empty(self) -> bool {
        self.start.untrusted() >= self.end.untrusted()
    }

    /// Try to create a proof that the range is nonempty.
    pub fn nonempty(self) -> Option<Range<'id, NonEmpty>> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                Some(Range::new_nonempty(
                    self.start.untrusted(),
                    self.end.untrusted(),
                ))
            }
        }
    }

    /// The starting index. (`NonEmpty` if the range is `NonEmpty`.)
    pub fn start(self) -> Index<'id, Emptiness> {
        unsafe { Index::new_any(self.start.untrusted()) }
    }

    /// The ending index.
    pub fn end(self) -> Index<'id, Unknown> {
        self.end
    }

    /// Split around the middle `index` if it is in this range,
    /// such that the second range contains the index.
    pub fn split_at<P>(self, index: Index<'id, P>) -> Option<(Range<'id, Unknown>, Range<'id, P>)> {
        if index >= self.start && index <= self.end {
            Some((Range::from(self.start().erased(), index), unsafe {
                Range::new_any(index.untrusted(), self.end.untrusted())
            }))
        } else {
            None
        }
    }

    /// If the index is a valid absolute index within this range.
    pub fn contains_in<Array: ?Sized + TrustedContainer>(
        self,
        index: u32,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        if index >= self.start().untrusted() && index < self.end().untrusted() {
            // we need a full vet to check that we're on an item index
            container.vet(index).ok()
        } else {
            None
        }
    }

    /// If the index is within this range. Provides a nonempty proof.
    pub fn contains<P>(self, index: Index<'id, P>) -> Option<Index<'id, NonEmpty>> {
        index.in_range(self)
    }

    /// Join together two adjacent ranges.
    /// (They must be exactly touching, non-overlapping, and in order.)
    pub fn join<Q>(
        self,
        other: Range<'id, Q>,
    ) -> Option<Range<'id, <(Emptiness, Q) as ProofAdd>::Sum>>
    where
        (Emptiness, Q): ProofAdd,
    {
        if self.end == other.start {
            unsafe {
                Some(Range::new_any(
                    self.start.untrusted(),
                    other.end.untrusted(),
                ))
            }
        } else {
            None
        }
    }

    /// Extend the range to cover all of `other`, including any space between.
    pub fn join_cover<Q>(
        self,
        other: Range<'id, Q>,
    ) -> Range<'id, <(Emptiness, Q) as ProofAdd>::Sum>
    where
        (Emptiness, Q): ProofAdd,
    {
        let start = cmp::min(self.start, other.start);
        let end = cmp::max(self.end, other.end);
        unsafe { Range::new_any(start.untrusted(), end.untrusted()) }
    }

    /// Create two empty ranges, at the front and the back of this range.
    pub fn frontiers(self) -> (Range<'id, Unknown>, Range<'id, Unknown>) {
        (
            Range::from(self.start().erased(), self.start()),
            Range::from(self.end(), self.end()),
        )
    }
}

impl<'id> Range<'id, NonEmpty> {
    #[doc(hidden)]
    pub fn observe_proof(self) {}

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
impl<'id, Emptiness> From<ops::Range<Index<'id, Emptiness>>> for Range<'id, Unknown> {
    fn from(r: ops::Range<Index<'id, Emptiness>>) -> Self {
        Range::from(r.start.erased(), r.end)
    }
}

// ~~~ Derive traits but without unneeded bounds ~~~ //

impl<'id, Emptiness> fmt::Debug for Range<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Range<'id>")
            .field(&self.start.untrusted())
            .field(&self.end.untrusted())
            .finish()
    }
}

impl<'id, Emptiness> Copy for Range<'id, Emptiness> {}
impl<'id, Emptiness> Clone for Range<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Eq for Range<'id, Emptiness> {}
impl<'id, P, Q> PartialEq<Range<'id, Q>> for Range<'id, P> {
    fn eq(&self, other: &Range<'id, Q>) -> bool {
        self.start.eq(&other.start) && self.end.eq(&other.end)
    }
}

impl<'id, Emptiness> Hash for Range<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}
