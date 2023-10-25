# Merlin Memory Model

## Overview
The Merlin programming language uses reference counting. It does **not** use a garbage collector! It uses reference counted, heap allocated objects to maintain memory safety. By leveraging Rust's memory safety, it is possible to safely ignore issues such as race conditions, memory corruption and leaks, and segmentation faults.

## Internal `Trc`
Internally a `Trc` is used to store data. Because a `.clone()` only creates a new data structure and does not copy the actual object data, this is a suitable option. The key to why `Trc` does not cause problems like memory leaks is because it is reference counted. See the [crates.io page](https://crates.io/crates/trc) for `Trc`,

## Benefits of `Trc` and comparison to `CPython`
`Trc` is a powerful data type. Besides it's atomic automatic memory management, it allows Merlin to be multithreaded! This is because the reference count is atomic. This is a contrast to `CPython`, which requires a `GIL` to provide a semblance of multithreading. The `Trc` data type gives Merlin inherent multithreading abilities. 
`Trc` implements biased reference counting, which allows it to remove the possibility of race conditions from the reference count - which is what prevents `CPython` from removing their
`GIL`.

## Footnote about systems lacking atomics
Merlin will automatically build to use a mutex instead. This incurrs a performance cost of around 200% on my machine, but allows Merlin to run.