# 1BRC

[**The One Billion Row Challenge**](https://www.morling.dev/blog/one-billion-row-challenge/)

Compute simple math over 1 billion rows, as fast as possible, without dependencies.

Modified, because this is really not a job for Java IMO. Let's Rust it up!

## Generate the data file

```console
$ cargo run --release --bin brc-generator 1000000000
    Finished release [optimized] target(s) in 24.18s
     Running `target/debug/brc-generator 1000000000`
```

It might take a few minutes to run. Make sure you have about 15 gigabytes free space.

## Run the reference implementation

```console
$ cargo build --release --bin brc-reference
   Compiling brc-reference v0.1.0 (1brc-reference)
    Finished release [optimized] target(s) in 0.62s

$ time target/release/brc-reference >/dev/null
real    0m9.737s
user    1m15.772s
sys     0m1.607s
```

### Optional Features

While the text of the challenge instructs us to use only the standard library, it's fun trying to eke out some extra performance by adding some dependencies.

- `fxhash`: replaces the hash algorithm with a non-cryptographically-safe one which might be quicker

## Write your own implementation

Create your solution in `solutions/yourname`, run it with:

```console
$ cargo build --release --bin brc-yourname
   Compiling brc-yourname v0.1.0 (1brc-yourname)
    Finished release [optimized] target(s) in 0.62s

$ time target/release/brc-yourname >/dev/null
```

When satisfied, submit a PR!

## Bench

Benchmark implementations on your hardware with [`hyperfine`](https://github.com/sharkdp/hyperfine):

```console
$ cargo build --release

$ hyperfine --warmup 1 'target/release/brc-yourname >/dev/null' 'target/release/1brc-reference >/dev/null'
Benchmark 1: target/release/brc-yourname >/dev/null
  Time (mean ± σ):     29.942 s ±  1.823 s    [User: 201.890 s, System: 3.708 s]
  Range (min … max):   28.056 s … 33.988 s    10 runs

Benchmark 2: target/release/brc-reference >/dev/null
  Time (mean ± σ):     43.444 s ± 18.045 s    [User: 203.612 s, System: 4.080 s]
  Range (min … max):   29.270 s … 77.972 s    10 runs

Summary
  'target/release/brc-yourname >/dev/null' ran
    1.45 ± 0.61 times faster than 'target/release/brc-reference >/dev/null'
```
