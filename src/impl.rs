use {
    crate::{
        particle::{perfect::*, IndexError},
        proof::*,
        traits::*,
        *,
    },
    core::{convert::TryFrom, ops},
    debug_unreachable::debug_unreachable,
};

/// IMPORTANT safety note: `ix < self.len() as u32` is enough both when
/// `usize <= u32` and `usize > u32`. If `usize <= u32`, this is lossless.
/// If `usize > u32`, the worst that will happen is that the length checked
/// will be modulo u32::MAX, in which case a) we're already broken because we
/// assume u32 is enough, and b) this will only decrease inbounds length.
unsafe fn to_usize<Array: ?Sized>(ix: u32, container: &Array) -> usize
where
    Array: TrustedContainer,
{
    debug_assert!(ix <= container.len());
    usize::try_from(ix).unwrap_or_else(|_| debug_unreachable!())
}

// ~~~ References ~~~ //

// cannot name D::Target [rust-lang/rust#60871]
unsafe impl<D> TrustedContainer for D
where
    D::Target: TrustedContainer,
    D: ops::Deref,
{
    type Item = <D::Target as TrustedContainer>::Item;
    type Slice = <D::Target as TrustedContainer>::Slice;

    fn len(&self) -> u32 {
        <D::Target>::len(self)
    }

    unsafe fn get_unchecked(&self, i: u32) -> &Self::Item {
        <D::Target>::get_unchecked(&*self, i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<u32>) -> &Self::Slice {
        <D::Target>::slice_unchecked(self, r)
    }
}

// cannot name D::Target [rust-lang/rust#60871]
unsafe impl<D> TrustedContainerMut for D
where
    D::Target: TrustedContainerMut,
    D: ops::DerefMut + ops::Deref,
{
    unsafe fn get_unchecked_mut(&mut self, i: u32) -> &mut Self::Item {
        <D::Target>::get_unchecked_mut(self, i)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<u32>) -> &mut Self::Slice {
        <D::Target>::slice_unchecked_mut(self, r)
    }
}

unsafe impl<T: ?Sized, Array: ?Sized, D> TrustedItem<D> for T
where
    T: TrustedItem<Array>,
    Array: TrustedContainer<Item = T>,
    D: ops::Deref<Target = Array>,
{
    type Unit = T::Unit;

    fn vet<'id>(
        idx: u32,
        container: &Container<'id, D>,
    ) -> Result<Index<'id, Unknown>, IndexError> {
        T::vet(idx, container)
    }

    unsafe fn vet_inbounds<'id>(
        ix: u32,
        container: &Container<'id, D>,
    ) -> Option<Index<'id, NonEmpty>> {
        T::vet_inbounds(ix, container)
    }
}

unsafe impl<T, Array, D> TrustedUnit<D> for T
where
    T: TrustedUnit<Array>,
    Array: TrustedContainer<Item = T>,
    D: ops::Deref<Target = Array>,
{
}

// ~~~ Slices ~~~ //

unsafe impl<T> TrustedContainer for [T] {
    type Item = T;
    type Slice = [T];

    fn len(&self) -> u32 {
        self.len() as u32
    }

    unsafe fn get_unchecked(&self, ix: u32) -> &T {
        let i = to_usize(ix, self);
        debug_assert!(i < self.len());
        self.get_unchecked(i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<u32>) -> &[T] {
        let r = to_usize(r.start, self)..to_usize(r.end, self);
        debug_assert!(r.start <= r.end);
        self.get_unchecked(r)
    }
}

unsafe impl<T> TrustedContainerMut for [T] {
    unsafe fn get_unchecked_mut(&mut self, ix: u32) -> &mut T {
        debug_assert!(ix < self.len());
        let i = to_usize(ix, self);
        self.get_unchecked_mut(i)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<u32>) -> &mut [T] {
        let r = to_usize(r.start, self)..to_usize(r.end, self);
        debug_assert!(r.start <= r.end);
        self.get_unchecked_mut(r)
    }
}

unsafe impl<T> TrustedUnit<[T]> for T {}
unsafe impl<T> TrustedItem<[T]> for T {
    type Unit = T;

    fn vet<'id>(
        ix: u32,
        container: &Container<'id, [T]>,
    ) -> Result<Index<'id, Unknown>, IndexError> {
        if ix <= container.len() {
            Ok(unsafe { Index::new(ix, container.id()) })
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    unsafe fn vet_inbounds<'id>(
        ix: u32,
        container: &Container<'id, [T]>,
    ) -> Option<Index<'id, NonEmpty>> {
        debug_assert!(ix < container.len());
        Some(Index::new(ix, container.id()))
    }
}

// ~~~ Strings ~~~ //

#[inline]
fn is_leading_byte(byte: u8) -> bool {
    // We want to accept 0b0xxx_xxxx or 0b11xx_xxxx
    // Copied from str::is_char_boundary
    // This is bit magic equivalent to: b < 128 || b >= 192
    (byte as i8) >= -0x40
}

unsafe impl TrustedContainer for str {
    type Item = Character;
    type Slice = str;

    fn len(&self) -> u32 {
        self.len() as u32
    }

    unsafe fn get_unchecked(&self, ix: u32) -> &Character {
        let i = to_usize(ix, self);
        debug_assert!(self.is_char_boundary(i));
        let slice = self.get_unchecked(i..);
        let byte_count = slice
            .char_indices()
            .map(|(i, _)| i)
            .nth(1)
            .unwrap_or_else(|| slice.len());
        debug_assert!(slice.is_char_boundary(byte_count));
        let code_point = slice.get_unchecked(..byte_count);
        &*(code_point as *const str as *const Character)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<u32>) -> &str {
        let r = to_usize(r.start, self)..to_usize(r.end, self);
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked(r)
    }
}

unsafe impl TrustedContainerMut for str {
    unsafe fn get_unchecked_mut(&mut self, ix: u32) -> &mut Character {
        let i = to_usize(ix, self);
        let slice = self.get_unchecked_mut(i..);
        let byte_count = slice
            .char_indices()
            .map(|(i, _)| i)
            .nth(1)
            .unwrap_or_else(|| str::len(&slice));
        debug_assert!(slice.is_char_boundary(byte_count));
        let code_point = slice.get_unchecked_mut(..byte_count);
        &mut *(code_point as *mut str as *mut Character)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<u32>) -> &mut Self::Slice {
        let r = to_usize(r.start, self)..to_usize(r.end, self);
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked_mut(r)
    }
}

unsafe impl TrustedItem<str> for Character {
    type Unit = u8;

    unsafe fn vet_inbounds<'id>(
        ix: u32,
        container: &Container<'id, str>,
    ) -> Option<Index<'id, NonEmpty>> {
        let i = to_usize(ix, container.untrusted());
        let leading_byte = *container.untrusted().as_bytes().get_unchecked(i);
        if is_leading_byte(leading_byte) {
            debug_assert!(container.untrusted().is_char_boundary(i));
            Some(Index::new(ix, container.id()))
        } else {
            debug_assert!(!container.untrusted().is_char_boundary(i));
            None
        }
    }
}
