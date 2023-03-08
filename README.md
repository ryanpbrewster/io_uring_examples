# FAQ

### os error 22
```
Benchmarking sync_read_direct: Warming up for 3.0000 sthread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Invalid argument (os error 22)', benches/random_read_bench.rs:21:42
```

This can happen if you try to open a file with `O_DIRECT` when it isn't properly
aligned. O_DIRECT requires that files are some multiple of 512 bytes, and that
the memory you're reading into is aligned (print out its pointer value, it
should look like `7ffe4fc5e200`, and specifically SHOULD NOT look
like`7ffe4fc5e1d0`).

# Some benchmark results on AWS GP3 EBS volumes

### Tiny dataset (64 KiB)

```
read
p50=0.8 p99=17.2 p999=26.8 avg=2.7 total=1.139e7

pread
p50=0.7 p99=0.9 p999=1.6 avg=0.7 total=2.338e7

direct-pread
p50=1.3 p99=1.6 p999=3.9 avg=1.3 total=1.289e7

mmap
p50=0.0 p99=0.1 p999=0.1 avg=0.0 total=4.302e7
```

### Medium dataset (64 MiB)

```
read
p50=1.1 p99=19.1 p999=30.1 avg=3.6 total=1.030e7

pread
p50=0.9 p99=1.4 p999=5.5 avg=0.9 total=3.736e7

direct-pread
p50=1.2 p99=1.5 p999=3.6 avg=1.2 total=3.461e7

mmap
p50=0.2 p99=0.5 p999=0.6 avg=0.2 total=7.539e7
```

### Giant dataset (64 GiB)

64 GiB dataset (representing ~8 billion u64 entries), running at concurrency=2. All latency numbers in microseconds.

```
read
p50=5.1 p99=77.2 p999=133.7 avg=12.2 total=2.102e6

pread
p50=3.3 p99=11.9 p999=35.8 avg=5.3 total=1.680e7

direct-pread
p50=1.5 p99=5.5 p999=10.3 avg=2.1 total=1.824e7

mmap
p50=57.5 p99=890.5 p999=9139.5 avg=101.5 total=9.456e5
```

These mmap numbers are bad enough that I'm pretty sure I need to enable huge pages.

### mmap comparison

```
4 GiB
p50=0.3 p99=0.6 p999=4.8 avg=0.3 total=8.164e7

8 GiB
p50=0.3 p99=32.6 p999=460.2 avg=4.7 total=4.701e7
```

Yeah, there is some kind of hilarious discontinuity right around 4 GiB (which
is suspiciously u32::MAX). I did check and transparent hugepages are enabled:
```
admin@ip-172-31-24-253:~/io_uring_examples$ cat /sys/kernel/mm/transparent_hugepage/enabled
[always] madvise never
```
