use {
    crate::{
        particle::{perfect::Index, simple},
        proof::*,
    },
    core::{
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
    pub(crate) unsafe fn new(start: u32, end: u32) -> Self {
        Range::from(simple::Range::new(start, end))
    }

    pub(crate) unsafe fn from(simple: simple::Range<'id, Emptiness>) -> Self {
        Range { simple }
    }
}

/// Downgrade
impl<'id, Emptiness> Range<'id, Emptiness> {
    pub fn untrusted(self) -> ops::Range<u32> {
        self.simple.untrusted()
    }

    pub fn erased(self) -> Range<'id, Unknown> {
        unsafe { Range::from(self.simple.erased()) }
    }

    pub fn simple(self) -> simple::Range<'id, Emptiness> {
        self.simple
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
        unsafe { Range::new(0, 0) }
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
