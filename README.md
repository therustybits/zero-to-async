# Zero to Async

This is the companion repository for the YouTube video, where we walk through
the creation of a basic `async` runtime in embedded Rust from the ground up.
The code for the end of each chapter is in its respective directory.

Is it perfect? No! But some parts of it might be helpful to those (ok, me)
trying to learn about `async/await` or using Rust in embedded systems.

My apologies to Ferris for the use of `unsafe` in a few places: some were
avoidable (PAC access via raw pointer), while others were not (`no-std`
`Waker`/`RawWaker` stuff).

## Prerequisites

A burning desire to learn new things? And watching the two videos leading up to
this one, of course, which cover:
- [How to get setup for embedded development in Rust](https://youtu.be/TOAynddiu5M)
- [Blinking an LED & the embedded Rust ecosystem](https://youtu.be/A9wvA_S6m7Y)

Also: reading the [Rust book](https://doc.rust-lang.org/book/) is always a good idea

## Further Research

Can't get enough `async` embedded Rust? Then I'd encourage you to check out:
- [`embassy`](https://embassy.dev/)
- [Dario Nieuwenhuis' 2024 RustNL talk introducing `embassy`](https://youtu.be/H7NtzyP9q8E)
- [`lilos`](https://github.com/cbiffle/lilos/)
- [Cliff Biffle's 2023 OSFC talk introducing `lilos`](https://www.osfc.io/2023/talks/turn-your-code-inside-out-programming-and-debugging-bare-metal-with-async-rust/)
- [Cliff's blog](https://cliffle.com/blog)

If you're looking for a similar bottom-up exploration of `async` Rust but this
time using the standard library, check out the book ["Asynchronous Programming
in Rust" by Carl Fredrik Samson.](https://www.packtpub.com/en-us/product/asynchronous-programming-in-rust-9781805128137)

Any discussion of `async` Rust would be incomplete without mentioning 
[without.boats blog](https://without.boats/blog/), which is full of great
articles about the history of `async/await` within the Rust project, `async`
topics like Pinning, and possible future directions for `async`.
