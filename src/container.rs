#[cfg(feature = "doc")]
use crate::{scope, scope_mut, scope_val};
use {
    crate::{particle::*, proof::*, traits::*},
    core::{convert::AsRef, fmt, mem, ops},
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
    id: generativity::Id<'id>,
    array: Array,
}

impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    pub(crate) fn new(array: Array, guard: generativity::Guard<'id>) -> Self
    where
        Array: Sized,
    {
        Container {
            id: guard.into(),
            array,
        }
    }

    pub(crate) fn new_ref<'a>(array: &'a Array, _guard: generativity::Guard<'id>) -> &'a Self {
        unsafe { &*(array as *const Array as *const Container<'id, Array>) }
    }

    pub(crate) fn new_ref_mut<'a>(
        array: &'a mut Array,
        _guard: generativity::Guard<'id>,
    ) -> &'a mut Self {
        unsafe { &mut *(array as *mut Array as *mut Container<'id, Array>) }
    }

    pub(crate) fn id(&self) -> generativity::Id<'id> {
        self.id
    }
}

/// Intrinsic properties
impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    /// This container without the branding.
    pub fn untrusted(&self) -> &Array {
        &self.array
    }

    /// This container without the branding.
    // FUTURE: Does this need the `Array: TrustedContainerMut` bound?
    //         For now, it's there as an overly cautious measure.
    pub fn untrusted_mut(&mut self) -> &mut Array
    where
        Array: TrustedContainerMut,
    {
        &mut self.array
    }

    /// This container without the branding.
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

    /// The full range of the container.
    pub fn as_range(&self) -> perfect::Range<'id, Unknown> {
        unsafe { perfect::Range::new(0, self.len(), self.id) }
    }

    /// The start index of the container.
    pub fn start(&self) -> perfect::Index<'id, Unknown> {
        unsafe { perfect::Index::new(0, self.id()) }
    }

    /// The end index of the container. (This is the one-past-the-end index.)
    pub fn end(&self) -> perfect::Index<'id, Unknown> {
        unsafe { perfect::Index::new(self.len(), self.id()) }
    }

    /// Take a internally trusted reference to the container.
    pub fn as_ref(&self) -> Container<'id, &'_ Array> {
        unsafe { mem::transmute(&*self) }
    }

    /// Take an internally trusted mutable reference to the container.
    pub fn as_ref_mut(&mut self) -> Container<'id, &'_ mut Array>
    where
        Array: TrustedContainerMut,
    {
        unsafe { mem::transmute(&mut *self) }
    }

    /// Convert this container into a simple container of the representational
    /// unit slice. The lifetime of the returned container _must_ be tied to
    /// the borrow here to enforce that the backing array is not mutated; if
    /// you want `Container<'id, &'id str>` and `Container<'id, &'id [u8]>`,
    /// use [`scope`] to get a `&'id Container<'id, str>`, use `simple` to get
    /// `Container<'id, &'id [u8]>`, then call [`as_ref`][`Container::as_ref`]
    /// to get `Container<'id, &'id [u8]>` and `Container<'id, &'id str>`.
    ///
    /// For owned values, Rust cannot support holding two separate views of
    /// the same value where one of which is owned or mutable. In this case,
    /// you will need to have transient sibling immutable views and batch
    /// mutability. (`Container<'id, &'a str>`, `Container<'id, &'a [u8]>`)
    pub fn simple(
        &self,
    ) -> Container<'id, &'_ [<<Array as TrustedContainer>::Item as TrustedItem<Array>>::Unit]>
    where
        Array: AsRef<[<<Array as TrustedContainer>::Item as TrustedItem<Array>>::Unit]>,
    {
        Container {
            id: self.id,
            array: self.array.as_ref(),
        }
    }
}

/// Upgrading particles
impl<'id, Array: ?Sized> Container<'id, Array>
where
    Array: TrustedContainer,
{
    /// Vet a particle for being inbounds and indexable to this container.
    pub fn vet<V: Vettable<'id>>(&self, particle: V) -> Result<V::Vetted, IndexError> {
        particle.vet(self)
    }

    /// Vet an index for being valid, including the one-past-the-end index.
    pub fn vet_or_end(&self, particle: u32) -> Result<perfect::Index<'id, Unknown>, IndexError> {
        Ok(if particle == self.len() {
            self.end()
        } else {
            self.vet(particle)?.erased()
        })
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
        Container {
            array: self.untrusted().clone(),
            id: self.id(),
        }
    }
}
