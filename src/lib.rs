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
//! - The container and its particles are “branded” with a lifetime
//!   parameter `'id` which is an identity marker. Branded items
//!   can't leave their scope, and they tie the items uniquely to a particular
//!   container. This makes it possible to trust them.
//!
//! # Borrowing
//!
//! - Particles are freely copyable and do not track the backing data
//!   themselves. All access to the underlying data goes through the
//!   [`Container`] (e.g. by indexing the container with a trusted index).
//!
//!   [indexing]: <https://github.com/bluss/indexing>

#![no_std]
#![deny(rust_2018_idioms, unconditional_recursion)]
#![cfg_attr(feature = "doc", feature(doc_cfg))]

mod container;
mod r#impl;

pub mod particle;
pub mod proof;
pub mod traits;

use {crate::traits::TrustedContainer, core::ops, debug_unreachable::debug_unreachable};

pub use crate::container::Container;

/// Create an indexing scope for a borrowed container.
///
/// The indexing scope is a closure that is passed a unique lifetime for the
/// parameter `'id`; this lifetime brands the container and its indices and
/// ranges, so that they are trusted to be in bounds.
///
/// Indices and ranges branded with `'id` cannot leave the closure. The
/// container can only be trusted when accessed through the `Container`
/// wrapper passed as the first argument to the indexing scope.
pub fn scope<Array: ?Sized, F, Out>(array: &Array, f: F) -> Out
where
    Array: TrustedContainer,
    F: for<'id> FnOnce(&'id Container<'id, Array>) -> Out,
{
    generativity::make_guard!(guard);
    f(Container::new_ref(array, guard))
}

/// Create an indexing scope for a mutably borrowed container.
///
/// The indexing scope is a closure that is passed a unique lifetime for the
/// parameter `'id`; this lifetime brands the container and its indices and
/// ranges, so that they are trusted to be in bounds.
///
/// Indices and ranges branded with `'id` cannot leave the closure. The
/// container can only be trusted when accessed through the `Container`
/// wrapper passed as the first argument to the indexing scope.
pub fn scope_mut<Array: ?Sized, F, Out>(array: &mut Array, f: F) -> Out
where
    Array: TrustedContainer,
    F: for<'id> FnOnce(&'id Container<'id, Array>) -> Out,
{
    generativity::make_guard!(guard);
    f(Container::new_ref_mut(array, guard))
}

/// Create an indexing scope for an owned container.
///
/// The indexing scope is a closure that is passed a unique lifetime for the
/// parameter `'id`; this lifetime brands the container and its indices and
/// ranges, so that they are trusted to be in bounds.
///
/// Indices and ranges branded with `'id` cannot leave the closure. The
/// container can only be trusted when accessed through the `Container` wrapper
/// passed as the first argument to the indexing scope.
pub fn scope_val<Array, F, Out>(array: Array, f: F) -> Out
where
    Array: TrustedContainer,
    F: for<'id> FnOnce(Container<'id, Array>) -> Out,
{
    generativity::make_guard!(guard);
    f(Container::new(array, guard))
}

/// A utf8 string slice of exactly one codepoint.
///
/// This type is two `usize` large, so you'll probably want to read the
/// underlying `char` out with [`Character::as_char`] as soon as possible.
//
// Potential optimization: use a thin pointer to `u8` and determine the length
// from the leading UTF-8 byte. We don't do that now such that the structure
// is properly unsized, and there's no potential for misuse where `&Character`
// can be accidentally created other than unsafely casting from `&str`.
#[repr(transparent)]
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Character(str);

impl ops::Deref for Character {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl ops::DerefMut for Character {
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl Character {
    pub fn as_char(&self) -> char {
        self.chars()
            .nth(0)
            .unwrap_or_else(|| unsafe { debug_unreachable!() })
    }
}
