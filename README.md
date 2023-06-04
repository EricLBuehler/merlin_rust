# Merlin

![Minimum rustc 1.72 nightly](https://img.shields.io/badge/rustc-1.72%20nightly-brightgreen)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)

<h2><strong>Make your code <i>fly</i></strong></h2>

Merlin is a dynamically and strongly typed programming language written in Rust. It uses Rust's atomic data types to remove the need for a GIL.

### Current comparsion to Python:

With this code:
```Python
a=1
b=2
c=3
a/b+c
```
Total execution time:

Merlin 1.2 (release): 554.4 ns

`./merlin program.me -t 100000`

Python 3.10.6: 59.9 ns 

`python3 -m timeit -c "a=1"`

Merlin is: 9.26x slower

## Installation
To get started with Merlin:
- Download rust (preferrably with rustup command line tool)
- Execute `rustup default nightly` (reversion = default stable)
- Run `make release`
- Execute code using the generated binary!

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)