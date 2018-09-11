use std::ops::{Generator, GeneratorState};

pub trait GenTrait {
    type Yielding;
    type Returning;

    fn next(&mut self) -> Option<Self::Yielding>;

    fn resume(&mut self) -> GeneratorState<Self::Yielding, Self::Returning>;

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

    fn next(&mut self) -> Option<Self::Yielding> {
        match unsafe { self.resume() } {
            GeneratorState::Yielded(y) => Some(y),
            GeneratorState::Complete(_) => None,
        }
    }

    fn resume(&mut self) -> GeneratorState<Self::Yielding, Self::Returning> {
        unsafe { <Self as Generator>::resume(self) }
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
        let mut g = move || {
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
