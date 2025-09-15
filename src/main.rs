use libc::{c_uchar, mincore, size_t};
use madvise::{AccessPattern, AdviseMemory};
use memmap2::Mmap;
use std::{ffi::c_void, fs::File, os::unix::fs::FileExt};

// TODO: implement rest of vmtouch from https://github.com/brk0v/sre-page-cache-article/blob/main/lru/main.go

// https://www.kernel.org/doc/Documentation/vm/pagemap.txt
const PAGEMAP_LENGTH: u64 = 8;
const PFN_MASK: u64 = (1 << 55) - 1;
const KPF_LRU: u64 = 1 << 5;
const KPF_ACTIVE: u64 = 1 << 6;

#[cfg(target_os = "linux")]
fn main() -> std::io::Result<()> {
    let path = "/var/tmp/file1.db";
    let page_size = page_size::get();
    let file = File::options().read(true).open(&path)?;
    let size = file.metadata()?.len();
    let pages = size.div_ceil(page_size as u64) as usize;

    // open mmap
    let mmap = unsafe { Mmap::map(&file)? };
    let ptr = mmap.as_ptr() as *mut c_void;
    mmap.advise_memory_access(AccessPattern::Random)?;

    let buf: Vec<u8> = vec![0; pages];
    unsafe {
        mincore(ptr, size as size_t, buf.as_ptr() as *mut c_uchar);
    }
    let count: usize = buf
        .iter()
        .enumerate()
        .map(|(e, item)| {
            if item % 2 == 1 {
                // populate page table of curr process with entries
                _ = unsafe { std::ptr::read_volatile(mmap.as_ptr().add(e * page_size)) };
                return 1;
            }
            0
        })
        .sum();

    // hack to notify kernel not to update reference bits
    mmap.advise_memory_access(AccessPattern::Sequential)?;

    // active and inactive pages
    let (active, inactive) = get_pagemap_stats(ptr as u64, pages as u64, page_size as u64)?;

    println!("Resident Pages: {}/{}", count, pages);
    println!("Active: {}", active);
    println!("Inactive: {}", inactive);
    Ok(())
}

fn get_pagemap_stats(ptr: u64, pages: u64, page_size: u64) -> std::io::Result<(u64, u64)> {
    // This file lets a userspace process find out which physical frame each virtual page is mapped to.
    // It contains one 64-bit value for each virtual page.
    let pagemap = File::options().read(true).open("/proc/self/pagemap")?;
    // This file contains a 64-bit count of the number of times each page is mapped, indexed by PFN.
    let kpageflags = File::options().read(true).open("/proc/kpageflags")?;

    let mut buf = [0u8; PAGEMAP_LENGTH as usize];
    let offset = (ptr / page_size) * PAGEMAP_LENGTH;

    let mut active = 0;
    let mut inactive = 0;

    for i in 0..pages {
        pagemap.read_at(&mut buf, offset + i * PAGEMAP_LENGTH)?;

        let pfn = u64::from_le_bytes(buf) & PFN_MASK;
        if pfn == 0 {
            continue;
        }

        kpageflags.read_at(&mut buf, pfn * PAGEMAP_LENGTH)?;
        let flags = u64::from_le_bytes(buf);

        if flags & KPF_ACTIVE != 0 {
            active += 1;
            continue;
        };

        if flags & KPF_LRU != 0 {
            inactive += 1;
            continue;
        };
    }

    Ok((active, inactive))
}
