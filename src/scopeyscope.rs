pub trait Let {
    fn let_<R>(self, f: impl FnOnce(Self) -> R) -> R
    where
        Self: Sized,
    {
        f(self)
    }
}

// Implement it for all types
impl<T> Let for T {}
