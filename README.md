# 1BRC

[**The One Billion Row Challenge**](https://www.morling.dev/blog/one-billion-row-challenge/)

Compute simple math over 1 billion rows, as fast as possible, without dependencies.

Modified, because this is really not a job for Java IMO. Let's Rust it up!

## Generate the data file

There is a feature-gated binary which can create the appropriate measurements list, as follows:

```sh
time cargo run --release --features generator --bin generate 1000000000
```
