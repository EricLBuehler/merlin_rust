# Merlin

![rustc 1.70 stable](https://img.shields.io/badge/rustc-1.70.0-brightgreen)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)
![Build status](https://github.com/EricLBuehler/merlin/actions/workflows/build.yml/badge.svg)
![Tests status](https://github.com/EricLBuehler/merlin/actions/workflows/tests.yml/badge.svg)

<h2><strong>Make your code <i>fly</i></strong></h2>

Merlin is a dynamically and strongly typed programming language written in Rust. Merlin's threading system empowers programmers to write powerful code that leverages concurrency - regardless of whether your system has atomics.

Merlin uses Rust's atomic data types to remove the need for a GIL without sacrificing performance.
In addition, it has a register-based interpreter (like CPUs) which has large performance advantages over the Python stack-based interpreter that is simpler and needs to shuffle a lot of memory around.

### Current comparison to Python:

With this code:
```Python
a=1
b=2
a+b
a+b
a+b
```
Total execution time:

Merlin 1.3 (release): 20 ns

`./merlin program.me -t 10000`

Python 3.10.6: 54.9 ns 

`python3 -m timeit -c "a=1;b=2;a+b;a+b;a+b"`

Merlin is: 2.75x **faster**

***

With this code:
```Python
a=1
b=2
a+b
a+b
a+b
a+b
a+b
a+b
```
Total execution time:

Merlin 1.3 (release): 57.86 ns

`./merlin program.me -t 10000`

Python 3.10.6: 103 ns 

`python3 -m timeit -c "a=1;b=2;a+b;a+b;a+b;a+b;a+b;a+b"`

Merlin is: 1.78x **faster**

***

Merlin 1.3: 2.89x slower for 2x more.

Python 3.10.6: 1.87x slower for 2x more.

## Installation
To get started with Merlin:
- Download rust (preferably with rustup command line tool)
- Run `make release`
- Execute code using the generated binary!

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)