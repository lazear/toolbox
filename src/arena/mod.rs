pub mod generational;
pub mod typed;

pub trait Arena {
    type Index;
    type Item;

    fn new() -> Self;
    fn with_capacity(cap: usize) -> Self;
    fn capacity(&self) -> usize;
    fn try_insert(&mut self, item: Self::Item) -> Option<Self::Index>;
    fn insert(&mut self, item: Self::Item) -> Self::Index;
    fn remove(&mut self, index: Self::Index) -> Option<Self::Item>;
    fn get(&self, index: Self::Index) -> Option<&Self::Item>;
}
