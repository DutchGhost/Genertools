use std::ops::{Generator, GeneratorState};
use std::pin::PinMut;
use std::ops::Deref;

pub trait GenTrait {
    type Yielding;
    type Returning;

    fn next(PinMut<Self>) -> Option<Self::Yielding>;

    unsafe fn resume(PinMut<Self>) -> GeneratorState<Self::Yielding, Self::Returning>;

    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Yielding) -> U
    {
        Map::new(self, f)
    }
}

impl <G> GenTrait for G
where
    G: Generator
{
    type Yielding = G::Yield;
    type Returning = G::Return;

    fn next(ptr: PinMut<Self>) -> Option<Self::Yielding> {
        match unsafe { ptr.resume() } {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }

    unsafe fn resume(ptr: PinMut<Self>) -> GeneratorState<Self::Yielding, Self::Returning> {
        <Self as Generator>::resume(PinMut::get_mut(ptr.reborrow()))
    }
}

pub struct Map<G, F> {
    generator: G,
    func: F
}

impl <G, F> Map<G, F> {
    pub fn new(generator: G, func: F) -> Self {
        Self {
            generator,
            func,
        }
    }
}

impl <U, G, F> Generator for Map<G, F>
where
    G: Generator,
    F: Fn(G::Yield) -> U
{
    type Yield = U;
    type Return = G::Return;
    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        match self.generator.resume() {
            GeneratorState::Yielded(y) => GeneratorState::Yielded((self.func)(y)),
            GeneratorState::Complete(r) => GeneratorState::Complete(r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_map() {
        let g = move || {
            yield 10;
            yield 20;
            yield 30;
        };

        let mut mapped = g.map(|item| item * 10);

        assert_eq!(mapped.next(), Some(100));
        assert_eq!(mapped.next(), Some(200));
        assert_eq!(mapped.next(), Some(300));
        assert_eq!(mapped.next(), None);
    }
}
