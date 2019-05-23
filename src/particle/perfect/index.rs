use {
    crate::{particle::simple, proof::*},
    core::{
        cmp,
        fmt::{self, Debug},
        hash::{self, Hash},
    },
};

#[repr(transparent)]
pub struct Index<'id, Emptiness = NonEmpty> {
    simple: simple::Index<'id, Emptiness>,
}

/// Constructors
impl<'id, Emptiness> Index<'id, Emptiness> {
    pub(crate) unsafe fn new(ix: u32, guard: generativity::Id<'id>) -> Self {
        Index::from(simple::Index::new(ix, guard))
    }

    pub(crate) unsafe fn from(simple: simple::Index<'id, Emptiness>) -> Self {
        Index { simple }
    }

    pub(crate) fn id(self) -> generativity::Id<'id> {
        self.simple.id()
    }
}

/// Downgrade
impl<'id, Emptiness> Index<'id, Emptiness> {
    /// This index without the brand.
    pub fn untrusted(self) -> u32 {
        self.simple.untrusted()
    }

    /// This index without the emptiness proof.
    pub fn erased(self) -> Index<'id, Unknown> {
        unsafe { Index::from(self.simple.erased()) }
    }

    /// This index in simple manipulation mode.
    pub fn simple(self) -> simple::Index<'id, Emptiness> {
        self.simple
    }
}

// ~~~ Standard traits ~~~ //

impl<'id, Emptiness> Copy for Index<'id, Emptiness> {}

impl<'id, Emptiness> Clone for Index<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Debug for Index<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("perfect::Index<'id>").finish()
    }
}

impl<'id> Default for Index<'id, Unknown> {
    fn default() -> Self {
        unsafe { Index::new(0, generativity::Id::new()) }
    }
}

impl<'id, Emptiness> Ord for Index<'id, Emptiness> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.simple.cmp(&other.simple)
    }
}

impl<'id, 'jd, Emptiness, P> PartialOrd<Index<'jd, P>> for Index<'id, Emptiness> {
    fn partial_cmp(&self, other: &Index<'jd, P>) -> Option<cmp::Ordering> {
        self.simple.partial_cmp(&other.simple)
    }
}

impl<'id, 'jd, Emptiness, P> PartialOrd<simple::Index<'jd, P>> for Index<'id, Emptiness> {
    fn partial_cmp(&self, other: &simple::Index<'jd, P>) -> Option<cmp::Ordering> {
        self.simple.partial_cmp(other)
    }
}

impl<'id, Emptiness> Eq for Index<'id, Emptiness> {}

impl<'id, 'jd, Emptiness, P> PartialEq<Index<'jd, P>> for Index<'id, Emptiness> {
    fn eq(&self, other: &Index<'jd, P>) -> bool {
        self.simple.eq(&other.simple)
    }
}

impl<'id, 'jd, Emptiness, P> PartialEq<simple::Index<'jd, P>> for Index<'id, Emptiness> {
    fn eq(&self, other: &simple::Index<'jd, P>) -> bool {
        self.simple.eq(other)
    }
}

impl<'id, Emptiness> Hash for Index<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.simple.hash(state)
    }
}
