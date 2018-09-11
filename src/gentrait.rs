use std::ops::{Generator, GeneratorState};
use std::pin::PinMut;
use std::marker::Unpin;
use aspin::AsPin;

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

    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Yielding) -> bool
    {
        Filter::new(self, f)
    }
}

impl <G> GenTrait for G
where
    G: Generator + Unpin
{
    type Yielding = G::Yield;
    type Returning = G::Return;

    fn next(mut ptr: PinMut<Self>) -> Option<Self::Yielding> {
        match unsafe { PinMut::get_mut(ptr.reborrow()).resume() } {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }

    unsafe fn resume(mut ptr: PinMut<Self>) -> GeneratorState<Self::Yielding, Self::Returning> {
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

impl <F, G: Unpin> AsPin<G> for Map<G, F> {
    fn as_pin(&mut self) -> PinMut<G> {
        PinMut::new(&mut self.generator)
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

pub struct Filter<G, F> {
    generator: G,
    pred: F
}

impl <G, F> Filter<G, F> {
    pub fn new(generator: G, pred: F) -> Self {
        Self {
            generator,
            pred
        }
    }
}

impl <F, G: Unpin> AsPin<G> for Filter<G, F> {
    fn as_pin(&mut self) -> PinMut<G> {
        PinMut::new(&mut self.generator)
    }
}

impl <G, F> Generator for Filter<G, F>
where
    G: Generator,
    F: Fn(&G::Yield) -> bool
{
    type Yield = G::Yield;
    type Return = G::Return;

    unsafe fn resume(&mut self) -> GeneratorState<Self::Yield, Self::Return> {
        loop {
            match self.generator.resume() {
                GeneratorState::Yielded(y) => {
                    if (self.pred)(&y) {
                        break GeneratorState::Yielded(y)
                    }
                    continue;
                }
                GeneratorState::Complete(r) => break GeneratorState::Complete(r)
            }
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

        let mut mapped = g.map(|item| item * 10).filter(|item| item > &199);

        let mut pin = mapped.as_pin();

        assert_eq!(GenTrait::next(pin.reborrow()), Some(200));
        assert_eq!(GenTrait::next(pin.reborrow()), Some(300));
        assert_eq!(GenTrait::next(pin.reborrow()), None);
    }
}
