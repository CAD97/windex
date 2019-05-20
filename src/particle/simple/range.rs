use {
    crate::{
        particle::{perfect, simple::Index},
        proof::*,
    },
    core::{
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
    pub(crate) unsafe fn new(start: u32, end: u32) -> Self {
        debug_assert!(start < end);
        Range {
            start: Index::new(start),
            end: Index::new(end),
            phantom: PhantomData,
        }
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
        unsafe { Range::new(self.start.untrusted(), self.end.untrusted()) }
    }
}

/// Intrinsic properties
impl<'id, Emptiness> Range<'id, Emptiness> {
    /// The start index of this range.
    pub fn start(self) -> Index<'id, Emptiness> {
        unsafe { Index::new(self.start.untrusted()) }
    }

    /// The end index of this range.
    pub fn end(self) -> Index<'id, Unknown> {
        self.end
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
        unsafe { Range::new(0, 0) }
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
