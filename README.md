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

Merlin 1.1 (release): 200 ns

`./merlin program.me -t 1000000`

Python 3.10.6: 11.4 ns 

`python3 -m timeit -c "a=1"`

Merlin is: 17.54x slower

**Important note**: I am currently using functions that do **not** work well for accurate timing on my Windows machine running WSL. 
They have a 100 ns granularity, so I am working on a way to use more accurate functions.

## Docs
- [Keywords](docs/keywords.md)
- [Memory model reasoning and internals](docs/memory_model.md)