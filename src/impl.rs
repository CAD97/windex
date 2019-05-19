use {
    crate::{particle::*, proof::*, traits::*, *},
    core::{convert::TryFrom, ops},
};

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
        u32::try_from(self.len()).unwrap()
    }

    unsafe fn get_unchecked(&self, i: u32) -> &T {
        let i = usize::try_from(i).unwrap();
        debug_assert!(i < self.len());
        self.get_unchecked(i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<u32>) -> &[T] {
        let r = usize::try_from(r.start).unwrap()..usize::try_from(r.end).unwrap();
        debug_assert!(r.start <= self.len());
        debug_assert!(r.end <= self.len());
        debug_assert!(r.start <= r.end);
        self.get_unchecked(r)
    }
}

unsafe impl<T> TrustedContainerMut for [T] {
    unsafe fn get_unchecked_mut(&mut self, i: u32) -> &mut T {
        debug_assert!(i < self.len());
        self.get_unchecked_mut(usize::try_from(i).unwrap())
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<u32>) -> &mut [T] {
        debug_assert!(r.start <= self.len());
        debug_assert!(r.end <= self.len());
        debug_assert!(r.start <= r.end);
        self.get_unchecked_mut(usize::try_from(r.start).unwrap()..usize::try_from(r.end).unwrap())
    }
}

unsafe impl<T> TrustedUnit<[T]> for T {}
unsafe impl<T> TrustedItem<[T]> for T {
    type Unit = T;
}

// ~~~ Strings ~~~ //

//#[inline]
//fn is_leading_byte(byte: u8) -> bool {
//    // We want to accept 0b0xxx_xxxx or 0b11xx_xxxx
//    // Copied from str::is_char_boundary
//    // This is bit magic equivalent to: b < 128 || b >= 192
//    (byte as i8) >= -0x40
//}

unsafe impl TrustedContainer for str {
    type Item = Character;
    type Slice = str;

    fn len(&self) -> u32 {
        u32::try_from(self.len()).unwrap()
    }

    unsafe fn get_unchecked(&self, i: u32) -> &Character {
        let i = usize::try_from(i).unwrap();
        debug_assert!(i < self.len());
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
        let r = usize::try_from(r.start).unwrap()..usize::try_from(r.end).unwrap();
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked(r)
    }
}

unsafe impl TrustedContainerMut for str {
    unsafe fn get_unchecked_mut(&mut self, i: u32) -> &mut Character {
        debug_assert!(i < self.len());
        let i = usize::try_from(i).unwrap();
        debug_assert!(self.is_char_boundary(i));
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
        let r = usize::try_from(r.start).unwrap()..usize::try_from(r.end).unwrap();
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked_mut(r)
    }
}

unsafe impl TrustedItem<str> for Character {
    type Unit = u8;
}
