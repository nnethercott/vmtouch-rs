# vmtouch-rs
Playing around with linux page cache and mmap. 

Rust implementation of the [SRE deep dive into Linux Page Cache](https://biriukov.dev/docs/page-cache/0-linux-page-cache-for-sre/) series.

```bash
> cargo build && sudo ./target/debug/vmtouch <your-file-here>
> Resident pages: 608/32768
> Active: 0
> Inactive: 16922
```
