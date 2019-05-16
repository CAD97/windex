use {
    crate::{proof::*, traits::*, *},
    core::ops,
    debug_unreachable::debug_unreachable,
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

    fn unit_len(&self) -> usize {
        <D::Target>::unit_len(self)
    }

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
        <D::Target>::get_unchecked(&*self, i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
        <D::Target>::slice_unchecked(self, r)
    }
}

// cannot name D::Target [rust-lang/rust#60871]
unsafe impl<D> TrustedContainerMut for D
where
    D::Target: TrustedContainerMut,
    D: ops::DerefMut + ops::Deref,
{
    unsafe fn get_unchecked_mut(&mut self, i: usize) -> &Self::Item {
        <D::Target>::get_unchecked_mut(self, i)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<usize>) -> &Self::Slice {
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

    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, D>,
    ) -> Result<Index<'id, I, Unknown>, IndexError> {
        T::vet(idx, container)
    }

    unsafe fn vet_inbounds<'id, I: Idx>(
        idx: I,
        container: &Container<'id, D>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        T::vet_inbounds(idx, container)
    }

    fn align<'id, I: Idx>(idx: I, container: &Container<'id, D>) -> Index<'id, I, Unknown> {
        T::align(idx, container)
    }

    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, D>,
    ) -> Index<'id, I, Unknown> {
        T::after(this, container)
    }

    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, D>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        T::advance(this, container)
    }

    fn before<'id, I: Idx, P>(
        this: Index<'id, I, P>,
        container: &Container<'id, D>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        T::before(this, container)
    }
}

// ~~~ Slices ~~~ //

unsafe impl<T> TrustedContainer for [T] {
    type Item = T;
    type Slice = [T];

    fn unit_len(&self) -> usize {
        self.len()
    }

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
        debug_assert!(i < self.len());
        self.get_unchecked(i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
        debug_assert!(r.start <= self.len());
        debug_assert!(r.end <= self.len());
        debug_assert!(r.start <= r.end);
        self.get_unchecked(r)
    }
}

unsafe impl<T> TrustedContainerMut for [T] {
    unsafe fn get_unchecked_mut(&mut self, i: usize) -> &Self::Item {
        debug_assert!(i < self.len());
        self.get_unchecked_mut(i)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<usize>) -> &Self::Slice {
        debug_assert!(r.start <= self.len());
        debug_assert!(r.end <= self.len());
        debug_assert!(r.start <= r.end);
        self.get_unchecked_mut(r)
    }
}

unsafe impl<T> TrustedUnit<[T]> for T {}
trusted_item_forwarding!({T} [T] => T);

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

    fn unit_len(&self) -> usize {
        self.len()
    }

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
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

    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked(r)
    }
}

unsafe impl TrustedContainerMut for str {
    unsafe fn get_unchecked_mut(&mut self, i: usize) -> &Self::Item {
        debug_assert!(i < self.len());
        debug_assert!(self.is_char_boundary(i));
        let slice = self.get_unchecked_mut(i..);
        let byte_count = slice
            .char_indices()
            .map(|(i, _)| i)
            .nth(1)
            .unwrap_or_else(|| slice.len());
        debug_assert!(slice.is_char_boundary(byte_count));
        let code_point = slice.get_unchecked_mut(..byte_count);
        &mut *(code_point as *mut str as *mut Character)
    }

    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<usize>) -> &Self::Slice {
        debug_assert!(self.is_char_boundary(r.start));
        debug_assert!(self.is_char_boundary(r.end));
        debug_assert!(r.start < r.end);
        self.get_unchecked_mut(r)
    }
}

unsafe impl TrustedItem<str> for Character {
    type Unit = u8;

    unsafe fn vet_inbounds<'id, I: Idx>(
        idx: I,
        container: &Container<'id, str>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        let leading_byte = *container
            .untrusted()
            .as_bytes()
            .get_unchecked(idx.as_usize());
        if is_leading_byte(leading_byte) {
            debug_assert!(container.untrusted().is_char_boundary(idx.as_usize()));
            Some(Index::new_nonempty(idx))
        } else {
            None
        }
    }

    fn align<'id, I: Idx>(idx: I, container: &Container<'id, str>) -> Index<'id, I, Unknown> {
        let mut i = idx.as_usize();

        if idx.as_usize() >= container.unit_len() {
            return container.end();
        }

        // Hopefully LLVM will vectorize or at least unroll this
        // The maximum UTF8 length is 4 bytes, so only check four
        for _ in 0..3 {
            unsafe {
                let byte = *container.untrusted().as_bytes().get_unchecked(i);

                if is_leading_byte(byte) {
                    debug_assert!(container.untrusted().is_char_boundary(i));
                    return Index::new(idx);
                }

                // This cannot overflow as the first byte of a string is a leading byte
                i = i.checked_sub(1).unwrap_or_else(|| debug_unreachable!())
            }
        }

        unsafe { debug_unreachable!() }
    }

    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, str>,
    ) -> Index<'id, I, Unknown> {
        let len = container[this].len();
        unsafe { Index::new(this.untrusted().add(len)) }
    }
}
