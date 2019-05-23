use {
    crate::{
        particle::{perfect::Index, simple},
        proof::*,
    },
    core::{
        cmp,
        fmt::{self, Debug},
        hash::{self, Hash},
        ops,
    },
};

#[repr(transparent)]
pub struct Range<'id, Emptiness = Unknown> {
    simple: simple::Range<'id, Emptiness>,
}

/// Constructors
impl<'id, Emptiness> Range<'id, Emptiness> {
    pub(crate) unsafe fn new(start: u32, end: u32, guard: generativity::Id<'id>) -> Self {
        Range::from(simple::Range::new(start, end, guard))
    }

    pub(crate) unsafe fn from(simple: simple::Range<'id, Emptiness>) -> Self {
        Range { simple }
    }

    pub(crate) fn id(self) -> generativity::Id<'id> {
        self.simple.id()
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
        self.simple.untrusted()
    }

    /// This range without the emptiness proof.
    pub fn erased(self) -> Range<'id, Unknown> {
        unsafe { Range::from(self.simple.erased()) }
    }

    /// This range in simple manipulation mode.
    pub fn simple(self) -> simple::Range<'id, Emptiness> {
        self.simple
    }
}

/// Intrinsic properties
impl<'id, Emptiness> Range<'id, Emptiness> {
    /// The start index of this range.
    pub fn start(self) -> Index<'id, Emptiness> {
        unsafe { Index::new(self.simple.start().untrusted(), self.id()) }
    }

    /// The end index of this range.
    pub fn end(self) -> Index<'id, Unknown> {
        unsafe { Index::new(self.simple.end().untrusted(), self.id()) }
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
    ///
    /// (Returns a simple index, as it isn't guaranteed on an item boundary.)
    pub fn vet(self, ix: u32) -> Option<simple::Index<'id, Emptiness>> {
        self.simple.vet(ix)
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

impl<'id, Emptiness> Copy for Range<'id, Emptiness> {}

impl<'id, Emptiness> Clone for Range<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Debug for Range<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("perfect::Range<'id>").finish()
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
        self.simple.eq(&other.simple)
    }
}

impl<'id, 'jd, Emptiness, P> PartialEq<simple::Range<'jd, P>> for Range<'id, Emptiness> {
    fn eq(&self, other: &simple::Range<'jd, P>) -> bool {
        self.simple.eq(other)
    }
}

impl<'id, Emptiness> Hash for Range<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.simple.hash(state)
    }
}
