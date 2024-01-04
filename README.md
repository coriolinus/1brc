# 1BRC

[**The One Billion Row Challenge**](https://www.morling.dev/blog/one-billion-row-challenge/)

Compute simple math over 1 billion rows, as fast as possible, without dependencies.

Modified, because this is really not a job for Java IMO. Let's Rust it up!

## Generate the data file

There is a feature-gated binary which can create the appropriate measurements list, as follows:

```sh
time cargo run --release --features generator --bin generate 1000000000
```

## Run the challenge

```sh
$ cargo build --release && time target/release/1brc >/dev/null
   Compiling one-billion-rows v0.1.0 (1brc)
    Finished release [optimized] target(s) in 0.62s

real    0m9.737s
user    1m15.772s
sys     0m1.607s
```

## Optional Features

While the text of the challenge instructs us to use only the standard library, it's fun trying to eke out some extra performance by adding some dependencies.

- `fxhash`: replaces the hash algorithm with a non-cryptographically-safe one which is approximately 1.2x quicker
