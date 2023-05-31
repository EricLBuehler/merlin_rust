# Merlin

![Minimum rustc 1.69](https://img.shields.io/badge/rustc-1.69-brightgreen)
[![MIT License](https://img.shields.io/badge/License-MIT-informational)](LICENSE)

<h2><strong>Make your code <i>fly</i></strong></h2>

Merlin is a dynamically and strongly typed programming language written in Rust. It uses Rust's atomic data types to remove the need for a GIL.

### Current comparsion to Python:

With this code:
```Python
a=1
```
Total execution time:

Merlin 1.1 (release): 500 ns

`./merlin program.me -t 100000`

Python 3.10.6: 12.2 ns 

`python3 -m timeit -c "a=1"`

Merlin is: 41.50x slower

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)