use std::num::NonZeroU32;

/// Minimum capacity for an `Arena`
pub const MIN_CAPACITY: u32 = 16;

/// The `Arena`, an allocator 
pub struct Arena<T> {
    data: Vec<Entry<T>>,
}

/// An index into an `Arena`
#[derive(PartialEq, PartialOrd, Copy, Clone)]
pub struct Index(NonZeroU32);

#[derive(PartialEq, PartialOrd)]
enum Entry<T> {
    Vacant(Option<NonZeroU32>),
    Occupied(T),
}

use std::fmt;
impl<T: fmt::Debug> fmt::Debug for Entry<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Entry::Vacant(opt) => write!(f, "Vacant({:?})", opt),
            Entry::Occupied(item) => write!(f, "{:?}", item),
        }
    }
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena::with_capacity(MIN_CAPACITY)
    }

    /// Allocate an `Arena` capable of storing `n` items before re-allocating
    /// The mininimum capacity for an `Arena` is specified in MIN_CAPACITY, 
    /// which defaults to 16
    pub fn with_capacity(n: u32) -> Arena<T> {
        // Invariant that we have at least MIN_CAPACITY vacant entries when we
        // initialize the Arena, so setting the first element to point to
        // element 1 is safe.
        let mut arena = Arena {
            data: vec![Entry::Vacant(None)],
        };
        arena.reserve(n.max(MIN_CAPACITY));
        arena
    }

    /// Returns the number of items the `Arena` can store without allocating
    pub fn capacity(&self) -> u32 {
        self.data.capacity() as u32 - 1
    }

    #[inline]
    fn get_free(&self) -> Option<NonZeroU32> {
        match self.data.get(0) {
            Some(Entry::Vacant(next)) => *next,
            _ => None,
        }
    }

    #[inline]
    fn set_free(&mut self, next: Option<NonZeroU32>) {
        self.data[0] = Entry::Vacant(next);
    }

    fn reserve(&mut self, n: u32) {
        debug_assert!(n >= MIN_CAPACITY);
        
        let start = self.data.len() as u32 ;
        let end = start + n;
        let head = self.get_free();

        self.data.reserve(n as usize);

        // Subtract one for the free-list pointer
        for idx in start .. end - 1 {
            self.data.push(Entry::Vacant(Some(NonZeroU32::new(idx + 1).unwrap())));
        }

        // self.data.extend(
        //     (start..end).map(|idx| ))),
        // );

        *self.data.last_mut().unwrap() = Entry::Vacant(head);
        self.set_free(Some(NonZeroU32::new(start).unwrap()));

    }

    /// Attempt to insert an item into the `Arena` without performing any
    /// additional allocations.
    ///
    /// Will return an `Err` if the `Arena` has no remaining capacity
    ///
    /// # May panic
    ///
    /// In the event that the `Arena`'s free list is corrupted, this
    /// function will panic
    pub fn try_insert(&mut self, item: T) -> Result<Index, T> {
        match self.get_free() {
            None => Err(item),
            Some(free) => {
                let index = free.get() as usize;
                let old = std::mem::replace(&mut self.data[index], Entry::Occupied(item));
                match old {
                    Entry::Occupied(_) => panic!("Corrupted arena!"),
                    Entry::Vacant(next) => {
                        self.set_free(next);
                        Ok(Index(free))
                    }
                }
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, item: T) -> Index {
        match self.try_insert(item) {
            Ok(idx) => idx,
            Err(item) => self.reserve_insert(item),
        }
    }

    fn reserve_insert(&mut self, item: T) -> Index {
        self.reserve(self.capacity());
        self.try_insert(item)
            .map_err(|_| ())
            .expect("Out of memory")
    }

    pub fn remove(&mut self, index: Index) -> Option<T> {
        let i = index.0.get() as usize;
        let free = self.get_free();
        let prev = std::mem::replace(&mut self.data[i], Entry::Vacant(free));

        match prev {
            Entry::Occupied(item) => {
                self.set_free(Some(index.0));
                Some(item)
            }
            Entry::Vacant(_) => {
                std::mem::replace(&mut self.data[i], prev);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fill() {
        let mut arena = Arena::new();
        assert_eq!(arena.capacity(), MIN_CAPACITY);
        dbg!(&arena.data);
        for i in 0..15 {
            arena.insert(i);            
        }
        dbg!(&arena.data);
        assert_eq!(arena.data[0], Entry::Vacant(None));
        assert_eq!(arena.capacity(), MIN_CAPACITY);
    }

    #[test]
    fn sizeof() {
        assert_eq!(std::mem::size_of::<Option<NonZeroU32>>(), 4);
        assert_eq!(std::mem::size_of::<Index>(), 4);
    }
}
