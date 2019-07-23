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

Code versions:

- [Clojure](https://github.com/tonsky/icfpc2019): Original code submitted to contest.
- [JoinR Clojure](https://github.com/joinr/icfpc2019/tree/opt): Various source level optimizations.
- [Rust baseline](https://github.com/tonsky/icfpc2019-rust/tree/baseline): Version that I wrote with stock libraries. Exactly the same algorithm and data structures as in Clojure version.
- [Rust + LTO](https://github.com/tonsky/icfpc2019-rust/commit/7c6b8c2ac2efc85dd36a659e0c96decf990c5cbf): tweaked build options.
- [Rust + FNV hash](https://github.com/tonsky/icfpc2019-rust/commit/12318461092f7bcdae4d72fe40af7a43fbf585ed): faster hashes.
- [Rust + blockers vector](https://github.com/tonsky/icfpc2019-rust): datastructure improvements (not directly comparable as algorithm is different).

Test machine: MacBook Pro (15-inch, 2018), 2.6 GHz Core i7, 6 cores w/ Hyper-Threading, 16 GB 2400 MHz DDR4, macOS 10.14.5.

Baseline algorithm, utilizing twelve threads (`contest`):

| Solution                     | Time, min | Time, ms   | Relative speed |
|------------------------------|-----------|------------|----------------|
| Clojure w/ JDK 12.0.1+12     | 15.4 min  | 926698 ms  | x1 (baseline)  |
| Clojure w/ GraalVM EE 19.1.0 | 10.6 min  | 638876 ms  | x1.45          |
| Clojure w/ Native Image      | 18.4 min  | 1105028 ms | x0.83          |
| Rust baseline                | 0.87 min  | 52460 ms   | x17.8          |
| Rust + LTO                   | 0.85 min  | 51154 ms   | x18            |
| Rust + FNV hash              | 0.48 min  | 28964 ms   | x32            |

Utilizing single thread:

| Solution                     | Time, min | Time, ms   | Relative speed |
|------------------------------|-----------|------------|----------------|
| Rust baseline                | 4.2 min   | 251625 ms  | x3.7           |
| Rust + FNV hash              | 2 min     | 124356 ms  | x7.45          |

Further improvements:

| Solution                           | Time, min | Time, ms   | Relative speed |
|------------------------------------|-----------|------------|----------------|
| Rust + blockers vector             | 0.41 min  | 24892 ms   | x37.2          | 
| JoinR Clojure w/ JDK 12.0.1+12     |  1.9 min  | 114582 ms  | x8.1           |
| JoinR Clojure w/ GraalVM CE 19.1.1 |  1.6 min  | 93582 ms   | x9.9           |

(time: lower is better, speed: bigger is better)

## Some conclusions

Disclaimer:

The point of this experiment was to show how _average_ code in different languages perform. Not some perfect, pushed to the limit code that some genius spent infinite time polishing. No, regular code that we write every day under time and resource pressure.

I also don’t try to slander Clojure specifically. I didn’t try to write slow and inefficient Clojure code on purpose. I didn’t try to make Rust code to cheat. This is a fair and honest comparison of how things are if you are just using them, day to day.

Clojure is still a great language for quick hacking and interactive exploration. I was just curious how much am I missing performance-wise.

That said,

- Naive Rust wins over naive Clojure ~20x while doing _the exact same thing_.
- Rust on a single thread performs better than Clojure on 12 threads.
- GraalVM gives your JVM program 1.5x boost basically for free.
- Clojure can be pushed really hard to be just ~4x slower than Rust version written by a complete beginner.

My personal highlights from seeing [JoinR](https://github.com/joinr/icfpc2019/blob/opt/README.org) and [Serioga](https://github.com/serioga/icfpc2019) findings:

- Clojure defrecords are terribly slow at computing hashes. They treat it like a generic map even though list of all fields is known to defrecord macro in advance.
- `(< 1 x 2)` is still slow after [three years of reporting it](https://clojure.atlassian.net/browse/CLJ-2075)
- Destructuring is slow :(
- Most of `clojure.core` methods like `get`/`nth` are polymorphic hence slow. `.valAt` on maps is much faster if you know it’s a map. You usually do.
- Getting rid of boxing warnings is almost impossible in a big codebase. At least, if you plan to use functions. Or datatypes more compact than long and double.

At this point you’ll be writing Java code in Clojure syntax anyway. So just swith to Java and don’t try making Clojure performant by making it look and work like Java. There’s no point in writing something that is only nominally Clojure.