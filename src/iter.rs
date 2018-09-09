use geniter::GenIter;

#[macro_export]
macro_rules! iter {
    ($($b:tt)*) => {
        GenIter::new(move || { $($b)* })
    }
}

#[macro_export]
macro_rules! yield_from {
    ($gen:expr) => {
        loop {
            unsafe {
                match $gen.resume() {
                    GeneratorState::Yielded(y) => yield y,
                    GeneratoreState::Complete(_) => break,
                }
            }
        }
    }
}

/// A macro for yielding from an option.
/// Yields whenever the variant is Some, returns otherwise.
#[macro_export]
macro_rules! try_yield {
    ($option:expr) => {
        match $option {
            Some(x) => yield x,
            None => return,
        }
    }
}

/// Creates an Iterator that only produces one item.
pub fn once<T>(x: T) -> impl Iterator<Item = T> {
    iter!(yield x)
}

/// Creates an Iterator that repeats the same item forever.
pub fn repeat<T: Copy>(x: T) -> impl Iterator<Item = T> {
    iter!(loop { yield x })
}

/// Creates an Iterator that repeats the same item for `n` times.
pub fn repeatn<T: Copy>(x: T, n: usize) -> impl Iterator<Item = T> {
    iter!(for _ in 0..n { yield x })
}

/// Creates an Iterator that filters elements based on the predicate.
/// After `false` is returned, the rest of the items are yielded.
pub fn filter_while<F, I>(f: F, mut iter: I) -> impl Iterator<Item = I::Item>
where
    I: Iterator,
    F: Fn(&I::Item) -> bool,
{
    iter!(
        while let Some(item) = iter.next() {
            if f(&item) { yield item } else { break }
        }

        for item in iter {
            yield item;
        }
    )
}

/// Creates an Iterator that alternates between generating the item `at the front`,
/// and then generating the item `at the back`
pub fn alternate<I>(mut iter: I) -> impl Iterator<Item = I::Item>
where
    I: DoubleEndedIterator
{
    iter!(loop {
        try_yield!(iter.next());
        try_yield!(iter.next_back());
    })
}

/// Creates an Iterator that generates the first `n` items `at the front`,
/// and then generates `n` items `at the back`, repeating untill no items are left.
pub fn alternate_by<I>(mut iter: I, n: usize) -> impl Iterator<Item = I::Item>
where
    I: DoubleEndedIterator
{
    iter!(loop {
        for _ in 0..n {
            try_yield!(iter.next());
        }

        for _ in 0..n {
            try_yield!(iter.next_back());
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternating() {

        let mut alternate = alternate(0..10);

        assert_eq!(alternate.next(), Some(0));
        assert_eq!(alternate.next(), Some(9));
        assert_eq!(alternate.next(), Some(1));
        assert_eq!(alternate.next(), Some(8));
        assert_eq!(alternate.next(), Some(2));
        assert_eq!(alternate.next(), Some(7));
        assert_eq!(alternate.next(), Some(3));
        assert_eq!(alternate.next(), Some(6));
        assert_eq!(alternate.next(), Some(4));
        assert_eq!(alternate.next(), Some(5));
        assert_eq!(alternate.next(), None);
    }
}
