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
is suspiciously right when the VM I'm benchmarking on runs out of RAM). I did
check and transparent hugepages are enabled:
```
admin@ip-172-31-24-253:~/io_uring_examples$ cat /sys/kernel/mm/transparent_hugepage/enabled
[always] madvise never
```

I think this is just what happens when you hit a bunch of page faults. Let's
confirm. I'll let both the 4 GiB and 8 GiB workloads run for 100 million
queries. I'm expecting the 8 GiB workload to hit way more page faults.
```
p50=0.3 p99=0.6 p999=4.5 avg=0.3 total=1.015e8
 Performance counter stats for './target/release/read_dataset --input /home/admin/big.dat --max-key 536870912 --concurrency 2 --variant mmap':

            107442      faults                                                      

      31.756305782 seconds time elapsed

      61.221542000 seconds user
       1.485039000 seconds sys
```
```
p50=0.3 p99=31.3 p999=450.8 avg=4.6 total=1.001e8
 Performance counter stats for './target/release/read_dataset --input /home/admin/big.dat --max-key 1073741824 --concurrency 2 --variant mmap':
          15658693      faults                                                      

     238.343423262 seconds time elapsed

      43.273421000 seconds user
     284.094372000 seconds sys
```

Ooof, look at the difference in system time! We hit 145x as many faults, and
spend nearly 200x as much time in the kernel.

By contrast, if we use `O_DIRECT` to avoid the page cache entirely, I'm expecting to see basically zero faults, and much less time in the kernel.
```
p50=1.2 p99=1.5 p999=4.0 avg=1.3 total=1.001e8
 Performance counter stats for './target/release/read_dataset --input /home/admin/big.dat --max-key 1073741824 --concurrency 2 --variant direct-pread':

               871      faults                                                      

      75.808721481 seconds time elapsed

      38.492406000 seconds user
     112.647794000 seconds sys
```

Hrm...well there aren't many faults, but that's still quite a lot of time in the kernel.

One kind of interesting thing is comparing `pread` to `direct-pread`:
```
p50=1.4 p99=11.3 p999=20.1 avg=2.2 total=1.004e8
 Performance counter stats for './target/release/read_dataset --input /home/admin/big.dat --max-key 1073741824 --concurrency 2 --variant pread':

              1120      faults                                                      

     124.951743039 seconds time elapsed

      33.861756000 seconds user
     180.965058000 seconds sys
```

Few faults, but overall notably slower, spending much more time in the kernel
(presumably uselessly pouplating the page cache).
