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

#[allow(dead_code)]
trait Also: Sized {
    fn also<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }

    fn also_mut<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        f(&mut self);
        self
    }
}

// Implement it for all types
impl<T> Also for T {}
