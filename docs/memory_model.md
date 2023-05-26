# Merlin Memory Model

## Overview
The Merlin programming language uses reference counting. It does **not** use a garbage collector! It uses reference counted, heap allocated objects to maintain memory safety. By leveraging Rust's memory safety, it is possible to safely ignore issues such as race conditions, memory corruption and leaks, and segmentation faults.

## Internal `Arc`
Internally an `Arc` is used to store data. Because a `.clone()` only creates a new data structure and does not copy the actual object data, this is a suitable option. The key to why `Arc` does not cause problems (memory leak, race condition) is becaue it a) deallocates memory when it is not needed, and b) is *atomically* reference counted.

## Benefits of `Arc` and comparison to `CPython`
`Arc` is a powerful data type. Besides it's atomic automatic memory management, it allows Merlin to be multithreaded! This is because the reference count is atomic. This is a contrast to `CPython`, which requires a `GIL` to provide a semblance of multithreading. The `Arc` data type gives Merlin inherent multithreading abilities. 