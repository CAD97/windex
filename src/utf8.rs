use {
    crate::{
        proof::{NonEmpty, Unknown},
        traits::{Idx, TrustedContainer, TrustedItem},
        Container, Index, IndexError,
    },
    core::ops,
    debug_unreachable::debug_unreachable,
};

#[allow(clippy::needless_return)] // alignment
pub(crate) fn is_leading_byte(byte: u8) -> bool {
    return byte & 0b1000_0000 == 0b0000_0000
        || byte & 0b1110_0000 == 0b1100_0000
        || byte & 0b1111_0000 == 0b1110_0000
        || byte & 0b1111_1000 == 0b1111_0000;
}

/// A utf8 string slice of exactly one codepoint.
#[repr(transparent)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Character(str);

impl ops::Deref for Character {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Character {
    pub fn as_char(&self) -> char {
        self.chars()
            .nth(0)
            .unwrap_or_else(|| unsafe { debug_unreachable!() })
    }
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
            .unwrap_or_else(|| self.len() - i);
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

unsafe impl TrustedItem<str> for Character {
    type Unit = u8;

    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, str>,
    ) -> Result<Index<'id, I, Unknown>, IndexError> {
        match idx.as_usize() {
            i if i < container.unit_len() => {
                let leading_byte = unsafe {
                    *container
                        .untrusted()
                        .as_bytes()
                        .get_unchecked(idx.as_usize())
                };
                if is_leading_byte(leading_byte) {
                    debug_assert!(container.untrusted().is_char_boundary(idx.as_usize()));
                    unsafe { Ok(Index::new(idx)) }
                } else {
                    Err(IndexError::Invalid)
                }
            }
            i if i == container.unit_len() => unsafe { Ok(Index::new(idx)) },
            _ => Err(IndexError::OutOfBounds),
        }
    }

    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, str>,
    ) -> Index<'id, I, Unknown> {
        let len = container[this].len();
        unsafe { Index::new(this.untrusted().add(len)) }
    }

    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, str>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        let next = Self::after(this, container);
        if next < container.end() {
            unsafe { Some(Index::new_nonempty(next.untrusted())) }
        } else {
            None
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::string::String;

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl TrustedContainer for String {
        type Item = Character;
        type Slice = str;

        fn unit_len(&self) -> usize {
            self.len()
        }

        unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
            <str as TrustedContainer>::get_unchecked(self, i)
        }

        unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
            <str>::get_unchecked(self, r)
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl TrustedItem<String> for Character {
        type Unit = u8;

        fn vet<'id, I: Idx>(
            idx: I,
            container: &Container<'id, String>,
        ) -> Result<Index<'id, I, Unknown>, IndexError> {
            Character::vet(idx, container.project())
        }

        fn after<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, String>,
        ) -> Index<'id, I, Unknown> {
            Character::after(this, container.project())
        }

        fn advance<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, String>,
        ) -> Option<Index<'id, I, NonEmpty>> {
            Character::advance(this, container.project())
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    impl<'id> Container<'id, String> {
        pub fn project(&self) -> &Container<'id, str> {
            unsafe { &*(&**self.untrusted() as *const str as *const Container<'id, str>) }
        }
    }
}
