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
cargo run --release problems/*.desc --threads=12
```

## Performance comparison

Compared to [github.com/tonsky/icfpc2019](https://github.com/tonsky/icfpc2019).

Test machine: MacBook Pro (15-inch, 2018), 2.6 GHz Core i7, 6 cores w/ Hyper-Threading, 16 GB 2400 MHz DDR4, macOS 10.14.5.

| Solution                     | No of Threads | Time, min | Time, ms  | Relative speed |
|------------------------------|---------------|-----------|-----------| ---------------|
| Clojure w/ JDK 12.0.1+12     | 12            | 15.4 min  | 926698 ms | x1 (baseline)  |
| Clojure w/ GraalVM EE 19.1.0 | 12            | 10.6 min  | 638876 ms | x1.45          |
| Rust                         | 12            | 0.87 min  | 52460 ms  | x17.8          |
| Rust                         | 1             | 4.2 min   | 251625 ms | x3.7           |

(time: lower is better, speed: bigger is better)