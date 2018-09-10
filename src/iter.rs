use geniter::GenIter;
use std::ops::{Generator, GeneratorState};

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
                    GeneratorState::Complete(_) => break,
                }
            }
        }
    };
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
    };
}

macro_rules! g {
    ($exp:expr, for $i:ident in $iter:expr) => {
        for $i in $iter {
            yield $exp
        }
    };
}

/// Creates an Iterator that only produces one item.
pub fn once<T>(x: T) -> impl Iterator<Item = T> {
    iter!(yield x)
}

/// Creates an Iterator that repeats the same item forever.
pub fn repeat<T: Copy>(x: T) -> impl Iterator<Item = T> {
    iter!(loop {
        yield x
    })
}

/// Creates an Iterator that repeats the same item for `n` times.
pub fn repeatn<T: Copy>(x: T, n: usize) -> impl Iterator<Item = T> {
    iter!(for _ in 0..n {
        yield x
    })
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
    I: DoubleEndedIterator,
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
    I: DoubleEndedIterator,
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

pub fn flatten<I, U>(iter: I) -> impl Iterator<Item = <I::Item as Iterator>::Item>
where
    I: Iterator,
    I::Item: Iterator,
{
    iter!(for it in iter {
        g!(x, for x in it)
    })
}

pub fn prepend<I>(value: I::Item, iter: I) -> impl Iterator<Item = I::Item>
where
    I: Iterator,
{
    ::std::iter::once(value).chain(iter)
}

pub fn islice<I>(
    iter: I,
    start: Option<usize>,
    stop: Option<usize>,
    step: Option<usize>,
) -> impl Iterator<Item = I::Item>
where
    I: Iterator,
{
    iter!(match (start, stop, step) {
        (None, None, None) => g!(x, for x in iter),

        (None, None, Some(it_step)) => g!(x, for x in iter.step_by(it_step)),

        (None, Some(it_stop), None) => g!(x, for x in iter.take(it_stop)),

        (None, Some(it_stop), Some(it_step)) => g!(x, for x in iter.step_by(it_step).take(it_stop)),

        (Some(it_start), None, None) => g!(x, for x in iter.skip(it_start)),

        (Some(it_start), None, Some(it_step)) => {
            g!(x, for x in iter.skip(it_start).step_by(it_step))
        }

        (Some(it_start), Some(it_stop), None) => g!(x, for x in iter.skip(it_start).take(it_stop)),

        (Some(it_start), Some(it_stop), Some(it_step)) => {
            g!(x, for x in iter.skip(it_start).step_by(it_step).take(it_stop))
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

    #[test]
    fn slicing() {
        let mut iter = islice(0..10, Some(3), None, None);

        assert_eq!(iter.next(), Some(3));

        // an Iterator over the characters, starting after 1 element, taking 3 elements of an iterator that steps by 2
        let mut chariter = islice("ABCDEFGHI".chars(), Some(1), Some(3), Some(2));

        assert_eq!(chariter.next(), Some('B'));
        assert_eq!(chariter.next(), Some('D'));
        assert_eq!(chariter.next(), Some('F'));
        assert_eq!(chariter.next(), None);

        let mut r = 1..=10;
        let list = islice(&mut r, None, Some(3), Some(2)).collect::<Vec<_>>();

        assert_eq!(list, [1, 3, 5]);

        assert_eq!(r.next(), Some(6));
    }
}
