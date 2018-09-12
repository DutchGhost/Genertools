use aspin::AsPin;
use std::marker::Unpin;
use std::ops::{Generator, GeneratorState};
use std::pin::PinMut;

pub trait GenTrait {
    type Yielding;
    type Returning;

    fn next(PinMut<Self>) -> Option<Self::Yielding>;

    unsafe fn resume(PinMut<Self>) -> GeneratorState<Self::Yielding, Self::Returning>;

    #[inline]
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Yielding) -> U,
    {
        Map::new(self, f)
    }

    #[inline]
    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Yielding) -> bool,
    {
        Filter::new(self, f)
    }

    #[inline]
    fn iter(&mut self) -> Iter<Self>
    where
        Self: Sized + Unpin + AsPin<Self>,
    {
        Iter::new(self)
    }
}

impl<G> GenTrait for G
where
    G: Generator + Unpin,
{
    type Yielding = G::Yield;
    type Returning = G::Return;

    #[inline]
    fn next(mut ptr: PinMut<Self>) -> Option<Self::Yielding> {
        match unsafe { PinMut::get_mut(ptr.reborrow()).resume() } {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }

    #[inline]
    unsafe fn resume(mut ptr: PinMut<Self>) -> GeneratorState<Self::Yielding, Self::Returning> {
        <Self as Generator>::resume(PinMut::get_mut(ptr.reborrow()))
    }
}

pub struct Map<G, F> {
    generator: G,
    func: F,
}

impl<G, F> Map<G, F> {

    #[inline]
    pub fn new(generator: G, func: F) -> Self {
        Self { generator, func }
    }
}

impl<F, G: Unpin> AsPin<G> for Map<G, F> {

    #[inline]
    fn as_pin(&mut self) -> PinMut<G> {
        PinMut::new(&mut self.generator)
    }
}

impl <F: Unpin, G: Unpin> AsPin<Self> for Map<G, F> {
    #[inline]
    fn as_pin(&mut self) -> PinMut<Self> {
        PinMut::new(self)
    }
}

impl<U, G, F> Generator for Map<G, F>
where
    G: Generator,
    F: Fn(G::Yield) -> U,
{
    type Yield = U;
    type Return = G::Return;

    #[inline]
    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        match self.generator.resume() {
            GeneratorState::Yielded(y) => GeneratorState::Yielded((self.func)(y)),
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }
}

pub struct Filter<G, F> {
    generator: G,
    pred: F,
}

impl<G, F> Filter<G, F> {

    #[inline]
    pub fn new(generator: G, pred: F) -> Self {
        Self { generator, pred }
    }
}

impl<F, G: Unpin> AsPin<G> for Filter<G, F> {

    #[inline]
    fn as_pin(&mut self) -> PinMut<G> {
        PinMut::new(&mut self.generator)
    }
}

impl <F: Unpin, G: Unpin> AsPin<Self> for Filter<G, F> {
    #[inline]
    fn as_pin(&mut self) -> PinMut<Self> {
        PinMut::new(self)
    }
}

impl<G, F> Generator for Filter<G, F>
where
    G: Generator,
    F: Fn(&G::Yield) -> bool,
{
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        loop {
            match self.generator.resume() {
                GeneratorState::Yielded(y) => {
                    if (self.pred)(&y) {
                        break GeneratorState::Yielded(y);
                    }
                    continue;
                }
                GeneratorState::Complete(r) => break GeneratorState::Complete(r),
            }
        }
    }
}

/// An Iterator that wraps over a Generator.
/// Ensures that generator's resume never gets called once the generator completed.
pub struct Iter<'a, G: 'a>(Option<PinMut<'a, G>>);

impl<'a, G: Unpin + 'a + AsPin<G>> Iter<'a, G> {
    #[inline]
    pub fn new(reference: &'a mut G) -> Self {
        Iter(Some(reference.as_pin()))
    }
}

impl<'a, G> Iterator for Iter<'a, G>
where
    G: GenTrait + Unpin + 'a,
{
    type Item = G::Yielding;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut pin = self.0.take()?;

        GenTrait::next(pin.reborrow()).map(move |item| {
            self.0 = Some(pin);
            item
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_map() {
        let mut quick_iter = (move || {
            yield 10u32;
            yield 20;
            yield 30;
        }).map(|item| item * 10).filter(|item| item > &199);

        let mut iter = quick_iter.iter();

        assert_eq!(iter.next(), Some(200));
        assert_eq!(iter.next(), Some(300));
        assert_eq!(iter.next(), None);
    }
}
