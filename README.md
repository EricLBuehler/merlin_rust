# Merlin

![rustc 1.70 stable](https://img.shields.io/badge/rustc-1.70.0-brightgreen)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)

<h2><strong>Make your code <i>fly</i></strong></h2>

Merlin is a dynamically and strongly typed programming language written in Rust. Merlin's threading system empowers programmers to write powerful code that leverages concurrency - regardless of whether your system has atomics.

Merlin uses Rust's atomic data types to remove the need for a GIL without sacrificing performance.
In addition, it has a register-based interpreter (like CPUs) which has large performance advantages over the Python stack-based interpreter that is simpler and needs to shuffle a lot of memory around.

### Current comparsion to Python:

With this code:
```Python
a=1
b=2
c=3
a/b+c
a/b+c
```
Total execution time:

Merlin 1.3 (release): 123.2 ns

`./merlin program.me -t 10000`

Python 3.10.6: 103 ns 

`python3 -m timeit -c "a=1;b=2;c=3;a/b+c;a/b+c"`

Merlin is: 19.61% slower

## Installation
To get started with Merlin:
- Download rust (preferrably with rustup command line tool)
- Run `make release`
- Execute code using the generated binary!

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)