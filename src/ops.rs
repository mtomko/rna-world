use std::ops::FnOnce;

pub trait ResultOps<T, E> {
    fn bi_map<A, B, D: FnOnce(E) -> B, F: FnOnce(T) -> A>(self, l: D, r: F) -> Result<A, B>;
    fn replace<A>(self, x: A) -> Result<A, E>;
}

impl<T, E> ResultOps<T, E> for Result<T, E> {
    fn bi_map<A, B, D: FnOnce(E) -> B, F: FnOnce(T) -> A>(self, l: D, r: F) -> Result<A, B> {
        match self {
            Ok(t) => Ok(r(t)),
            Err(e) => Err(l(e)),
        }
    }

    fn replace<A>(self, x: A) -> Result<A, E> {
        match self {
            Ok(_) => Ok(x),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::ops::ResultOps;

    #[test]
    fn replace_test() {
        let r: Result<&'static str, i32> = Ok("foo");
        let e: Result<&'static str, i32> = Err(3);
        assert_eq!(r.replace("bar"), Ok("bar"));
        assert_eq!(e.replace("bar"), Err(3));
    }
}
