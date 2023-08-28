/// A module for providing error context

/// An error (E), and some context (C)
pub struct ErrorContext<E>(pub String, pub E);

/// Creating a trait to extend an API by adding a context. method.
pub trait ErrorContextExt<T, E> {
    fn context<C: AsRef<str> + 'static>(self, c: C) -> Result<T, ErrorContext<E>>;
}

impl<T, E> ErrorContextExt<T, E> for Result<T, E> {
    fn context<C: AsRef<str> + 'static>(self, c: C) -> Result<T, ErrorContext<E>> {
        let s = c.as_ref();
        self.map_err(|e| ErrorContext(s.into(), e))
    }
}
