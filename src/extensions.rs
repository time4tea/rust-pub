use std::iter::Filter;

pub trait FilterNotIterator: Iterator {
    fn filter_not<P>(self, predicate: P) -> Filter<Self, impl FnMut(&Self::Item) -> bool>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool;
}

impl<T: Iterator> FilterNotIterator for T {
    fn filter_not<P>(self, predicate: P) -> Filter<Self, impl FnMut(&Self::Item) -> bool>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        let mut predicate = predicate;
        self.filter(move |x| !predicate(x))
    }
}
