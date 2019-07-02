Re-implementaion of [github.com/tonsky/icfpc2019](https://github.com/tonsky/icfpc2019) in Rust to compare performance.

## Problem

[icfpcontest2019.github.io](https://icfpcontest2019.github.io)

## Building and running

```
RUST_BACKTRACE=1 cargo run problems/prob-049.desc --interactive
```

Running release version:

```
cargo run --release problems/prob-049.desc
```

Solve all:

```
./script/run_all
```