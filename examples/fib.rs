#![feature(generators)]
#[macro_use]
extern crate genertools;

fn fib() -> impl Iterator<Item = usize> {
    iter!(
        let (mut current, mut nxt) = (0, 1);

        loop {
            nxt = std::mem::replace(&mut current, nxt) + current;
            yield current
        }
    )
}

fn main() {
    for item in fib().take(10) {
        println!("{}", item);
    }
}
