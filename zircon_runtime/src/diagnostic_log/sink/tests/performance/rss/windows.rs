use std::ffi::c_void;

type Handle = *mut c_void;

#[repr(C)]
#[derive(Default)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "GetCurrentProcess"]
    fn get_current_process() -> Handle;
    #[link_name = "K32GetProcessMemoryInfo"]
    fn get_process_memory_info(
        process: Handle,
        counters: *mut ProcessMemoryCounters,
        size: u32,
    ) -> i32;
}

pub(super) fn working_set_bytes() -> Option<u64> {
    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        ..ProcessMemoryCounters::default()
    };
    let size = counters.cb;
    // SAFETY: the pseudo-handle is valid for this process and counters describes writable memory.
    let succeeded = unsafe { get_process_memory_info(get_current_process(), &mut counters, size) };
    (succeeded != 0).then_some(counters.working_set_size as u64)
}
