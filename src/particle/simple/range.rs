use {
    crate::{
        particle::{perfect, simple::Index},
        proof::*,
    },
    core::{
        cmp,
        fmt::{self, Debug},
        hash::{self, Hash},
        marker::PhantomData,
        ops,
    },
};

pub struct Range<'id, Emptiness = Unknown> {
    start: Index<'id, Unknown>,
    end: Index<'id, Unknown>,
    phantom: PhantomData<Emptiness>,
}

/// Constructors
impl<'id, Emptiness> Range<'id, Emptiness> {
    pub(crate) unsafe fn new(start: u32, end: u32, guard: generativity::Id<'id>) -> Self {
        debug_assert!(start < end);
        Range {
            start: Index::new(start, guard),
            end: Index::new(end, guard),
            phantom: PhantomData,
        }
    }

    pub(crate) fn id(self) -> generativity::Id<'id> {
        self.start.id()
    }
}

/// Constructors
impl<'id> Range<'id, Unknown> {
    /// Create an empty range at the given index.
    pub fn singleton<P>(index: Index<'id, P>) -> Self {
        unsafe { Range::new(index.untrusted(), index.untrusted(), index.id()) }
    }
}

/// Downgrade
impl<'id, Emptiness> Range<'id, Emptiness> {
    /// This range without the brand.
    pub fn untrusted(self) -> ops::Range<u32> {
        self.start.untrusted()..self.end.untrusted()
    }

    /// This range without the emptiness proof.
    pub fn erased(self) -> Range<'id, Unknown> {
        unsafe {
            Range::new(
                self.start.untrusted(),
                self.end.untrusted(),
                self.start.id(),
            )
        }
    }
}

/// Intrinsic properties
impl<'id, Emptiness> Range<'id, Emptiness> {
    /// The start index of this range.
    pub fn start(self) -> Index<'id, Emptiness> {
        unsafe { Index::new(self.start.untrusted(), self.id()) }
    }

    /// The end index of this range.
    pub fn end(self) -> Index<'id, Unknown> {
        self.end
    }

    /// The length of this range (in representational units).
    pub fn len(self) -> u32 {
        self.end().untrusted() - self.start().untrusted()
    }

    /// Does this range contain no items?
    pub fn is_empty(self) -> bool {
        self.start() >= self.end()
    }

    /// Is this index in this range?
    pub fn contains<P>(self, index: Index<'id, P>) -> bool {
        self.start() <= index && index < self.end()
    }

    /// Vet an untrusted index for being in range.
    pub fn vet(self, ix: u32) -> Option<Index<'id, Emptiness>> {
        // Safe because we check it immediately
        let index = unsafe { Index::new(ix, self.id()) };
        if self.contains(index) {
            Some(index)
        } else {
            None
        }
    }
}

/// Manipulation
impl<'id, Emptiness> Range<'id, Emptiness> {
    /// Split this range at an index, if that index is in the range.
    ///
    /// The given index is contained in the second range.
    pub fn split_at<P>(self, index: Index<'id, P>) -> Option<(Range<'id>, Range<'id, Emptiness>)> {
        if self.contains(index) {
            unsafe {
                Some((
                    Range::new(self.start().untrusted(), index.untrusted(), self.id()),
                    Range::new(index.untrusted(), self.end().untrusted(), self.id()),
                ))
            }
        } else {
            None
        }
    }

    /// Join together two adjacent ranges.
    ///
    /// (They must be exactly touching, in left-to-right order.)
    pub fn join<P>(
        self,
        other: Range<'id, P>,
    ) -> Option<Range<'id, <(Emptiness, P) as ProofAdd>::Sum>>
    where
        (Emptiness, P): ProofAdd,
    {
        if self.end() == other.start() {
            unsafe {
                Some(Range::new(
                    self.start().untrusted(),
                    other.end().untrusted(),
                    self.id(),
                ))
            }
        } else {
            None
        }
    }

    /// Extend this range to cover both itself and `other`,
    /// including any space inbetween.
    pub fn join_cover<P>(
        self,
        other: Range<'id, P>,
    ) -> Range<'id, <(Emptiness, P) as ProofAdd>::Sum>
    where
        (Emptiness, P): ProofAdd,
    {
        let start = cmp::min(self.start().erased(), other.start().erased());
        let end = cmp::max(self.end(), other.end());
        unsafe { Range::new(start.untrusted(), end.untrusted(), self.id()) }
    }

    /// Extend the end of this range to the given index.
    pub fn extend_end<P>(self, index: Index<'id, P>) -> Range<'id, Emptiness> {
        let end = cmp::max(self.end().erased(), index.erased());
        unsafe { Range::new(self.start().untrusted(), end.untrusted(), self.id()) }
    }

    /// The empty range at the start and end of this range.
    pub fn frontiers(&self) -> (Range<'id, Unknown>, Range<'id, Unknown>) {
        (Range::singleton(self.start()), Range::singleton(self.end()))
    }
}

// ~~~ Standard traits ~~~ //

impl<'id, Emptiness> From<perfect::Range<'id, Emptiness>> for Range<'id, Emptiness> {
    fn from(index: perfect::Range<'id, Emptiness>) -> Self {
        index.simple()
    }
}

impl<'id, Emptiness> Copy for Range<'id, Emptiness> {}

impl<'id, Emptiness> Clone for Range<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Debug for Range<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("simple::Range<'id>").finish()
    }
}

impl<'id> Default for Range<'id, Unknown> {
    fn default() -> Self {
        unsafe { Range::new(0, 0, generativity::Id::new()) }
    }
}

impl<'id, Emptiness> Eq for Range<'id, Emptiness> {}

impl<'id, 'jd, Emptiness, P> PartialEq<Range<'jd, P>> for Range<'id, Emptiness> {
    fn eq(&self, other: &Range<'jd, P>) -> bool {
        self.start.eq(&other.start) && self.end.eq(&other.end)
    }
}

impl<'id, 'jd, Emptiness, P> PartialEq<perfect::Range<'jd, P>> for Range<'id, Emptiness> {
    fn eq(&self, other: &perfect::Range<'jd, P>) -> bool {
        self.eq(&other.simple())
    }
}

impl<'id, Emptiness> Hash for Range<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}
