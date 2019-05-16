//! Sound unchecked indexing using the techniques from generative lifetimes,
//! extended to string slices and without pointer or mutability support.
//!
//! Major kudos go to Gankro and especially Bluss for the original [indexing]
//! crate, from which this crate blatantly steals all of its cleverness.
//!
//! # Basic Structure
//!
//! - A scope is created using the [`scope`] function; inside this scope,
//!   there is a [`Container`] object that has two roles: (1) it gives out or
//!   vets trusted indices, pointers and ranges (2) it provides access to the
//!   underlying data through these indices and ranges.
//!
//! - The container and its indices and ranges are “branded” with a lifetime
//!   parameter `'id` which is an identity marker. Branded items
//!   can't leave their scope, and they tie the items uniquely to a particular
//!   container. This makes it possible to trust them.
//!
//! - `Index<'id>` is a trusted index
//! - `Range<'id, Emptiness>` is a trusted range.
//!
//! - For a range, if the proof parameter `Emptiness` is `NonEmpty`, then the
//!   range is known to have at least one element. An observation: A non-empty
//!   range always has a valid front index, so it is interchangeable with the
//!   index representation.
//!
//! - Indices also use the same proof parameter. A `NonEmpty` index points to a
//!   valid element, while an `Unknown` index is an edge index (it can be used
//!   to slice the container, but not to dereference to an element).
//!
//! - All ranges have a `.first()` method to get the first index in the range,
//!   but it's only when the range is nonempty that the returned index is also
//!   `NonEmpty` and thus dereferenceable.
//!
//! # Borrowing
//!
//! - Indices and ranges are freely copyable and do not track the backing data
//!   themselves. All access to the underlying data goes through the
//!   [`Container`] (e.g. by indexing the container with a trusted index).
//!
//!   [indexing]: <https://github.com/bluss/indexing>

#![no_std]
#![deny(rust_2018_idioms, unconditional_recursion)]
#![cfg_attr(feature = "doc", feature(doc_cfg))]

#[cfg(feature = "std")]
extern crate std;

mod container;
mod r#impl;
mod index;
pub mod proof;
pub mod traits;

use {crate::traits::TrustedContainer, debug_unreachable::debug_unreachable, std::ops};

pub use crate::{
    container::Container,
    index::{Index, IndexError, Range},
};

/// Create an indexing scope for a container.
///
/// The indexing scope is a closure that is passed a unique lifetime for the
/// parameter `'id`; this lifetime brands the container and its indices and
/// ranges, so that they are trusted to be in bounds.
///
/// Indices and ranges branded with `'id` cannot leave the closure. The
/// container can only be accessed through the `Container` wrapper passed as
/// the first argument to the indexing scope.
pub fn scope<Array: TrustedContainer, F, Out>(array: Array, f: F) -> Out
where
    F: for<'id> FnOnce(Container<'id, Array>) -> Out,
{
    // This is where the magic happens. We bind the indexer and indices to the
    // same invariant lifetime (a constraint established by F's definition).
    // As such, each call to `indices` produces a unique signature that only
    // these two values can share.
    //
    // Within this function, the borrow solver can choose literally any
    // lifetime, including `'static`, but we don't care what the borrow solver
    // does in *this* function. We only need to trick the solver in the
    // caller's scope. Since borrowck doesn't do interprocedural analysis, it
    // sees every call to this function produces values with some opaque fresh
    // lifetime and can't unify any of them.
    //
    // In principle a "super borrowchecker" that does interprocedural analysis
    // would break this design, but we could go out of our way to somehow bind
    // the lifetime to the inside of this function, making it sound again.
    // Rustc will never do such analysis, so we don't care.
    f(unsafe { Container::new(array) })
}

/// [`scope`], but for a backing container behind a reference
/// (such as an unsized string slice).
pub fn scope_ref<Array: TrustedContainer, F, Out>(array: &Array, f: F) -> Out
where
    F: for<'id> FnOnce(&'id Container<'id, Array>) -> Out,
{
    f(unsafe { &*(array as *const Array as *const Container<'_, Array>) })
}

/// A utf8 string slice of exactly one codepoint.
///
/// This type is two pointers large, so you'll probably want to read the
/// underlying `char` out with [`Character::as_char`] as soon as possible.
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
