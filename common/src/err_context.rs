/// A module for providing error context

/// An error (E), and some context (C)
pub struct ErrorContext<C, E>(pub C, pub E);

/// Creating a trait to extend an API by adding a context. method.
pub trait ErrorContextExt<T, E> {
    fn context<C>(self, c: C) -> Result<T, ErrorContext<C, E>>;
}

impl<T, E> ErrorContextExt<T, E> for Result<T, E> {
    fn context<C>(self, c: C) -> Result<T, ErrorContext<C, E>> {
        self.map_err(|e| ErrorContext(c, e))
    }
}
