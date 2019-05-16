#[cfg(feature = "doc")]
use crate::{scope, scope_val};
use {
    crate::{proof::Id, traits::TrustedContainer},
    core::fmt,
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

/// Intrinsic property accessors
impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    /// This container without the branding.
    ///
    /// # Note
    ///
    /// The returned lifetime of `&Array` is _not_ `'id`! It's completely
    /// valid to drop the container during a [`scope_val`], in which case this
    /// reference would become invalid. If you need a longer lifetime,
    /// consider using [`scope`] such that the reference is guaranteed to
    /// live for the entire scope.
    pub fn untrusted(&self) -> &Array {
        &self.array
    }

    /// This container without the branding.
    ///
    /// # Note
    ///
    /// The returned lifetime of `&Array` is _not_ `'id`! It's completely
    /// valid to drop the container during a [`scope_val`], in which case this
    /// reference would become invalid. If you need a longer lifetime,
    /// consider using [`scope`] such that the reference is guaranteed to
    /// live for the entire scope.
    pub fn untrusted_mut(&mut self) -> &Array {
        &mut self.array
    }

    /// The length of the container in base item units.
    pub fn len(&self) -> u32 {
        self.array.len()
    }

    /// Is this container empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ~~~ Standard traits ~~~ //

impl<'id, Array: ?Sized + fmt::Debug> fmt::Debug for Container<'id, Array>
where
    Array: TrustedContainer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Container<'id>").field(&&self.array).finish()
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
