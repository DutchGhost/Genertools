use std::pin::PinMut;
use std::marker::Unpin;

pub trait AsPin: Unpin {
    fn as_pin(&mut self) -> PinMut<Self> {
        PinMut::new(self)
    }
}

impl <T> AsPin for T
where
    T: Unpin
{
    
}
