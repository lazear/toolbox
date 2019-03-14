//! A space-efficient, generational Arena allocator.
//!
//! Invariants:
//!
//! - The first entry (index 0) will always be Vacant, and serves as the
//! head of the free list. This allows us to use an Option<NonZeroU32>
//! to save space
//!
//! - `Index` is also a NonZeroU32, where the highest 8 bits are used to
//! store the generation of the entry, and the low 24 bits are represent
//! the index of the `Entry` in the `Arena`. This puts a hard cap on the
//! number of generations (255) and items (2^24 - 2) that can be stored
//! in the `Arena`, but uses significantlly less space. This could be
//! tuned to use 16 bits for both generation and index if necessary.
#![forbid(unsafe_code)]
#![allow(dead_code)]
use std::num::NonZeroU32;

const MIN_CAPACITY: u32 = 16;

/// A generational arena allowing 255 generations, and 2^24 - 2 items
pub struct Arena<T> {
    data: Vec<Entry<T>>,
    len: u32,
}

/// Entry in an Arena<T>
enum Entry<T> {
    /// Vacant entry contains a pointer to the next free vacant entry
    Vacant(Option<NonZeroU32>),
    /// Occupied entry contains a generation count and a value
    Occupied(u8, T),
}

/// `Index` into an `Arena`, with bitpacked generation and index values
#[derive(Copy, Clone)]
pub struct Index(NonZeroU32);
impl Index {
    fn gen(&self) -> u8 {
        let mask = 0xFF00_0000;
        ((self.0.get() & mask) >> 24) as u8
    }

    fn pair(self) -> (u8, u32) {
        let mask = 0xFF00_0000;
        let gen = ((self.0.get() & mask) >> 24) as u8;
        let idx = self.0.get() & !mask;
        (gen, idx)
    }

    fn new(gen: u8, index: NonZeroU32) -> Index {
        Index(NonZeroU32::new(index.get() | ((gen as u32) << 24)).unwrap())
    }
}

impl<T> Arena<T> {
    fn with_capacity(n: u32) -> Arena<T> {
        assert!(n & 0xFF00_0000 == 0);
        let mut arena = Arena {
            data: vec![Entry::Vacant(None)],
            len: 0,
        };
        arena.reserve(n);
        arena
    }

    fn next_free(&self) -> Option<NonZeroU32> {
        match self.data.get(0) {
            Some(Entry::Vacant(ref next)) => *next,
            _ => None,
        }
    }

    fn set_free(&mut self, index: NonZeroU32) {
        self.data[0] = Entry::Vacant(Some(index))
    }

    fn get(&self, index: Index) -> Option<&T> {
        let (gen, idx) = index.pair();
        match self.data.get(idx as usize) {
            Some(Entry::Occupied(g, val)) if *g == gen => Some(val),
            _ => None,
        }
    }

    fn try_insert(&mut self, item: T) -> Option<Index> {
        let idx = self.next_free()?;
        let free = idx.get() as usize;
        match self.data[free] {
            Entry::Occupied(_, _) => panic!("Corrupted free list"),
            Entry::Vacant(next) => {
                self.data[0] = Entry::Vacant(next);
                self.data[free] = Entry::Occupied(0, item);
                Some(Index(idx))
            }
        }
    }

    fn reserve(&mut self, n: u32) {
        let start = self.data.len() as u32;
        let end = start + n;
        let free = self.next_free();

        self.data.reserve(n as usize);
        self.data.extend((start..end).map(|idx| {
            if idx == end - 1 {
                Entry::Vacant(free)
            } else {
                Entry::Vacant(Some(NonZeroU32::new(idx + 1).unwrap()))
            }
        }));
        self.set_free(NonZeroU32::new(start).unwrap());
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let a = Arena::<u32>::with_capacity(256);
        assert_eq!(a.next_free().map(|n| n.get()), Some(1))
    }
}
