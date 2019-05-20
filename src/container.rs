#[cfg(feature = "doc")]
use crate::{scope, scope_val};
use {
    crate::{particle::*, proof::*, traits::*},
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

/// Intrinsic properties
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
    // FUTURE: Does this need the `Array: TrustedContainerMut` bound?
    //         For now, it's there as an overly cautious measure.
    pub fn untrusted_mut(&mut self) -> &Array
    where
        Array: TrustedContainerMut,
    {
        &mut self.array
    }

    /// This container without the branding.
    ///
    /// # Note
    ///
    /// If you use this method to strip the brand, you cannot get it back!
    pub fn into_untrusted(self) -> Array
    where
        Array: Sized,
    {
        self.array
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

/// Upgrading particles
impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    pub fn vet<V: Vettable<'id>>(&self, particle: V) -> Result<V::Vetted, IndexError> {
        particle.vet(self)
    }
}

// ~~~ Accessors ~~~ //

impl<'id, Array: ?Sized> ops::Index<ops::RangeFull> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, _index: ops::RangeFull) -> &Self::Output {
        unsafe { self.array.slice_unchecked(0..self.len()) }
    }
}

impl<'id, Array: ?Sized> ops::IndexMut<ops::RangeFull> for Container<'id, Array>
where
    Array: TrustedContainerMut,
{
    fn index_mut(&mut self, _index: ops::RangeFull) -> &mut Self::Output {
        unsafe { self.array.slice_unchecked_mut(0..self.len()) }
    }
}

// ~~ Perfect ~~ //

// ~ ref ~ //

impl<'id, Array: ?Sized, P> ops::Index<perfect::Range<'id, P>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, index: perfect::Range<'id, P>) -> &Self::Output {
        unsafe { self.array.slice_unchecked(index.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeTo<perfect::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, index: ops::RangeTo<perfect::Index<'id, P>>) -> &Self::Output {
        unsafe { self.array.slice_unchecked(0..index.end.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeFrom<perfect::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Slice;

    fn index(&self, index: ops::RangeFrom<perfect::Index<'id, P>>) -> &Self::Output {
        unsafe {
            self.array
                .slice_unchecked(index.start.untrusted()..self.len())
        }
    }
}

impl<'id, Array: ?Sized> ops::Index<perfect::Index<'id, NonEmpty>> for Container<'id, Array>
where
    Array: TrustedContainer,
{
    type Output = Array::Item;

    fn index(&self, index: perfect::Index<'id, NonEmpty>) -> &Self::Output {
        unsafe { self.array.get_unchecked(index.untrusted()) }
    }
}

// ~ mut ~ //

impl<'id, Array: ?Sized, P> ops::IndexMut<perfect::Range<'id, P>> for Container<'id, Array>
where
    Array: TrustedContainerMut,
{
    fn index_mut(&mut self, index: perfect::Range<'id, P>) -> &mut Self::Output {
        unsafe { self.array.slice_unchecked_mut(index.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::IndexMut<ops::RangeTo<perfect::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainerMut,
{
    fn index_mut(&mut self, index: ops::RangeTo<perfect::Index<'id, P>>) -> &mut Self::Output {
        unsafe { self.array.slice_unchecked_mut(0..index.end.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::IndexMut<ops::RangeFrom<perfect::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainerMut,
{
    fn index_mut(&mut self, index: ops::RangeFrom<perfect::Index<'id, P>>) -> &mut Self::Output {
        unsafe {
            self.array
                .slice_unchecked_mut(index.start.untrusted()..self.len())
        }
    }
}

impl<'id, Array: ?Sized> ops::IndexMut<perfect::Index<'id, NonEmpty>> for Container<'id, Array>
where
    Array: TrustedContainerMut,
{
    fn index_mut(&mut self, index: perfect::Index<'id, NonEmpty>) -> &mut Self::Output {
        unsafe { self.array.get_unchecked_mut(index.untrusted()) }
    }
}

// ~~ Simple ~~ //

// ~ ref ~ //

impl<'id, Array: ?Sized, P> ops::Index<simple::Range<'id, P>> for Container<'id, Array>
where
    Array: TrustedContainer,
    Array::Item: TrustedUnit<Array>,
{
    type Output = Array::Slice;

    fn index(&self, index: simple::Range<'id, P>) -> &Self::Output {
        unsafe { self.array.slice_unchecked(index.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeTo<simple::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainer,
    Array::Item: TrustedUnit<Array>,
{
    type Output = Array::Slice;

    fn index(&self, index: ops::RangeTo<simple::Index<'id, P>>) -> &Self::Output {
        unsafe { self.array.slice_unchecked(0..index.end.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::Index<ops::RangeFrom<simple::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainer,
    Array::Item: TrustedUnit<Array>,
{
    type Output = Array::Slice;

    fn index(&self, index: ops::RangeFrom<simple::Index<'id, P>>) -> &Self::Output {
        unsafe {
            self.array
                .slice_unchecked(index.start.untrusted()..self.len())
        }
    }
}

impl<'id, Array: ?Sized> ops::Index<simple::Index<'id, NonEmpty>> for Container<'id, Array>
where
    Array: TrustedContainer,
    Array::Item: TrustedUnit<Array>,
{
    type Output = Array::Item;

    fn index(&self, index: simple::Index<'id, NonEmpty>) -> &Self::Output {
        unsafe { self.array.get_unchecked(index.untrusted()) }
    }
}

// ~ mut ~ //

impl<'id, Array: ?Sized, P> ops::IndexMut<simple::Range<'id, P>> for Container<'id, Array>
where
    Array: TrustedContainerMut,
    Array::Item: TrustedUnit<Array>,
{
    fn index_mut(&mut self, index: simple::Range<'id, P>) -> &mut Self::Output {
        unsafe { self.array.slice_unchecked_mut(index.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::IndexMut<ops::RangeTo<simple::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainerMut,
    Array::Item: TrustedUnit<Array>,
{
    fn index_mut(&mut self, index: ops::RangeTo<simple::Index<'id, P>>) -> &mut Self::Output {
        unsafe { self.array.slice_unchecked_mut(0..index.end.untrusted()) }
    }
}

impl<'id, Array: ?Sized, P> ops::IndexMut<ops::RangeFrom<simple::Index<'id, P>>>
    for Container<'id, Array>
where
    Array: TrustedContainerMut,
    Array::Item: TrustedUnit<Array>,
{
    fn index_mut(&mut self, index: ops::RangeFrom<simple::Index<'id, P>>) -> &mut Self::Output {
        unsafe {
            self.array
                .slice_unchecked_mut(index.start.untrusted()..self.len())
        }
    }
}

impl<'id, Array: ?Sized> ops::IndexMut<simple::Index<'id, NonEmpty>> for Container<'id, Array>
where
    Array: TrustedContainerMut,
    Array::Item: TrustedUnit<Array>,
{
    fn index_mut(&mut self, index: simple::Index<'id, NonEmpty>) -> &mut Self::Output {
        unsafe { self.array.get_unchecked_mut(index.untrusted()) }
    }
}

// ~~~ Deref ~~~ //

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
