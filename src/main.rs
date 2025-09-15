use libc::{c_uchar, mincore, size_t};
use madvise::{AccessPattern, AdviseMemory};
use memmap2::Mmap;
use std::{ffi::c_void, fs::File};

// TODO: implement rest of vmtouch from https://github.com/brk0v/sre-page-cache-article/blob/main/lru/main.go 

fn main() -> std::io::Result<()> {
    let path = "/var/tmp/file1.db";
    let page_size = page_size::get();
    let file = File::open(&path)?;
    let size = file.metadata()?.len();
    let pages = size.div_ceil(page_size as u64) as usize;

    // open mmap
    // NOTE: do we need this to get a ptr in our virtual address space ??
    // > "mincore() returns a vector that indicates whether pages of the calling process's vir‐
    // tual  memory  are  resident  in core (RAM)"
    let mmap = unsafe { Mmap::map(&file)? };
    let ptr = mmap.as_ptr() as *mut c_void;
    dbg!(ptr);
    mmap.advise_memory_access(AccessPattern::Random)?;

    let buf: Vec<u8> = vec![0; pages];
    unsafe {
        mincore(ptr, size as size_t, buf.as_ptr() as *mut c_uchar);
    }

    let count: usize = buf.iter().map(|item| (item % 2 == 1) as usize).sum();
    println!("Resident pages: {}/{}", count, pages);

    Ok(())
}
