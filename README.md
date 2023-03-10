# restate

[![ci-badge]][ci] [![Latest Version]][crates.io] [![docs-badge]][docs]

[ci]: https://github.com/Neo-Ciber94/restate/actions/workflows/ci.yml
[ci-badge]: https://github.com/Neo-Ciber94/restate/actions/workflows/ci.yml/badge.svg
[latest version]: https://img.shields.io/crates/v/restate.svg
[crates.io]: https://crates.io/crates/restate
[docs]: https://docs.rs/restate
[docs-badge]: https://docs.rs/restate/badge.svg

`restate` is a Rust library that provides a simple way of defining and using finite state machines.

## Installation

Add the following to your Cargo.toml file:

```toml
[dependencies]
restate = "0.1.0-alpha"
```

Or use

```bash
cargo add restate
```

## Usage

restate can be used to define state machines that can transition between different states based on events. Here is an example:

```rust
use restate::blocking::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Active;

#[derive(PartialEq, Eq)]
enum CountEvent {
    Increment,
    Decrement,
}

let mut sm = Machine::with_context(0)
    .on_next(
        Builder::self_transition(Active, CountEvent::Increment).action(
            |cx: ContextMut<Active, CountEvent, i32>| {
                *cx.context += 1;
            },
        ),
    )
    .on_next(
        Builder::self_transition(Active, CountEvent::Decrement).action(
            |cx: ContextMut<Active, CountEvent, i32>| {
                *cx.context -= 1;
            },
        ),
    )
    .start(Active);

sm.send(CountEvent::Increment).unwrap();
sm.send(CountEvent::Increment).unwrap();
sm.send(CountEvent::Increment).unwrap();
sm.send(CountEvent::Decrement).unwrap();

assert_eq!(*sm.context(), 2);
```

This example creates a state machine with a single state Active and two events Increment and Decrement. It then adds a self-transition for each event that increments or decrements an integer in the machine's context. Finally, it starts the machine with the Active state, sends some events to it, and checks the final value of the context.

## License

This library is licensed under the MIT license. See the LICENSE file for more details.
