use std::ops::{Generator, GeneratorState};

pub struct GenIter<G>(Option<G>);

impl<G> GenIter<G> {
    pub fn new(generator: G) -> Self {
        GenIter(Some(generator))
    }

    pub fn as_mut(&mut self) -> Option<&mut G> {
        self.0.as_mut()
    }

    pub fn take(&mut self) -> Option<G> {
        self.0.take()
    }
}

impl<G> Iterator for GenIter<G>
where
    G: Generator,
{
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        let mut gen = self.take()?;

        unsafe {
            match gen.resume() {
                GeneratorState::Yielded(y) => {
                    self.0 = Some(gen);
                    Some(y)
                }
                GeneratorState::Complete(_) => None,
            }
        }
    }
}
