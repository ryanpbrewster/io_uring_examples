# FAQ

### os error 22
```
Benchmarking sync_read_direct: Warming up for 3.0000 sthread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Invalid argument (os error 22)', benches/random_read_bench.rs:21:42
```

This can happen if you try to open a file with `O_DIRECT` when it isn't properly
aligned. O_DIRECT requires that files are some multiple of 512 bytes.