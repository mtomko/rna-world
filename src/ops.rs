use std::ops::FnOnce;

pub trait ResultOps<T, E> {
    fn bi_map<A, B, D: FnOnce(E) -> B, F: FnOnce(T) -> A>(self, l: D, r: F) -> Result<A, B>;
}

impl<T, E> ResultOps<T, E> for Result<T, E> {
    fn bi_map<A, B, D: FnOnce(E) -> B, F: FnOnce(T) -> A>(self, l: D, r: F) -> Result<A, B> {
        match self {
            Ok(t) => Ok(r(t)),
            Err(e) => Err(l(e)),
        }
    }
}
