pub enum Either<T> {
    Left(T),
    Right(T),
}

impl<T> Either<T> {
    #[inline]
    pub fn left(t: T) -> Either<T> {
        Either::Left(t)
    }

    #[inline]
    pub fn right(t: T) -> Either<T> {
        Either::Right(t)
    }

    #[inline]
    pub fn is_left(&self) -> bool {
        match self {
            Either::Left(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_right(&self) -> bool {
        match self {
            Either::Right(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn get(&self) -> &T {
        match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        match self {
            Either::Left(t) => t,
            Either::Right(t) => t,
        }
    }

    pub fn map<U, F>(self, op: F) -> Either<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Either::Left(t) => Either::Left(op(t)),
            Either::Right(t) => Either::Right(op(t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_either() {
        let left = Either::left(1);
        let right = Either::right(-1);
        assert!(left.is_left());
        assert_eq!(left.get(), &1);
        assert!(right.is_right());
        assert_eq!(right.get(), &-1);
        let left = left.map(|i| if i > 0 { Some(i) } else { None });
        let right = right.map(|i| if i > 0 { Some(i) } else { None });
        assert!(left.is_left());
        assert_eq!(left.get(), &Some(1));
        assert!(right.is_right());
        assert_eq!(right.get(), &None);
    }
}
