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

## Tags

`baseline` - version that I wrote with stock libraries. Exactly the same algorithm and data structures as in Clojure version.
`contest`  - baseline + release build flags + FNV hash
`master`   - algorithmic further improvements

## Performance comparison

Compared to [github.com/tonsky/icfpc2019](https://github.com/tonsky/icfpc2019).

Test machine: MacBook Pro (15-inch, 2018), 2.6 GHz Core i7, 6 cores w/ Hyper-Threading, 16 GB 2400 MHz DDR4, macOS 10.14.5.

Baseline algorithm, utilizing twelve threads (`contest`):

| Solution                     | Time, min | Time, ms   | Relative speed |
|------------------------------|-----------|------------|----------------|
| Clojure w/ JDK 12.0.1+12     | 15.4 min  | 926698 ms  | x1 (baseline)  |
| Clojure w/ GraalVM EE 19.1.0 | 10.6 min  | 638876 ms  | x1.45          |
| GraalVM + Native Image       | 18.4 min  | 1105028 ms | x0.83          |
| Rust baseline                | 0.87 min  | 52460 ms   | x17.8          |
| Rust + LTO                   | 0.85 min  | 51154 ms   | x18            |
| Rust + FNV hash              | 0.48 min  | 28964 ms   | x32            |

Utilizing single thread:

| Solution                     | Time, min | Time, ms   | Relative speed |
|------------------------------|-----------|------------|----------------|
| Rust baseline                | 4.2 min   | 251625 ms  | x3.7           |
| Rust + FNV hash              | 2 min     | 124356 ms  | x7.45          |

Further improvements (`master`):

| Solution                     | Time, min | Time, ms   | Relative speed |
|------------------------------|-----------|------------|----------------|
| Rust + blockers vector       | 0.41 min  | 24892 ms   | x37.2          | 


(time: lower is better, speed: bigger is better)
