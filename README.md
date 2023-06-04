# Merlin

![rustc 1.70 stable](https://img.shields.io/badge/rustc-1.70.0-brightgreen)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)

<h2><strong>Make your code <i>fly</i></strong></h2>

Merlin is a dynamically and strongly typed programming language written in Rust. It uses Rust's atomic data types to remove the need for a GIL.
In addition, it has a register-based interpreter (which mimics CPUs) and so has performance advantages over the Python stack-based interpreter.

### Current comparsion to Python:

With this code:
```Python
a=1
b=2
c=3
a/b+c
```
Total execution time:

Merlin 1.3 (release): 384.8 ns

`./merlin program.me -t 100000`

Python 3.10.6: 58.7 ns 

`python3 -m timeit -c "a=1;b=2;c=3;a/b+c"`

Merlin is: 6.56x slower

## Installation
To get started with Merlin:
- Download rust (preferrably with rustup command line tool)
- Execute `rustup default nightly` (reversion = default stable) if your system does not have atomics.
- Run `make release`
- Execute code using the generated binary!

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)