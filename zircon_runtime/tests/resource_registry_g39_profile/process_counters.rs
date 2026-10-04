//! Windows process counters. Samples are process-wide, not query-owned peak RSS.
//! ABI follows the existing Interface text projection RSS probe and Win32 headers.

use std::ffi::c_void;
use std::io;
use std::mem::size_of;

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}
impl FileTime {
    fn ticks(&self) -> u64 {
        u64::from(self.low) | (u64::from(self.high) << 32)
    }
}

#[repr(C)]
#[derive(Default)]
struct IoCounters {
    read_operations: u64,
    write_operations: u64,
    other_operations: u64,
    read_bytes: u64,
    write_bytes: u64,
    other_bytes: u64,
}

#[repr(C)]
#[derive(Default)]
struct MemoryCounters {
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
    fn GetCurrentProcess() -> *mut c_void;
    fn GetProcessTimes(
        process: *mut c_void,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
    fn GetProcessIoCounters(process: *mut c_void, counters: *mut IoCounters) -> i32;
}
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(process: *mut c_void, counters: *mut MemoryCounters, size: u32) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Snapshot {
    pub(super) user_100ns: u64,
    pub(super) kernel_100ns: u64,
    pub(super) rss_bytes: usize,
    pub(super) lifetime_peak_rss_bytes: usize,
    pub(super) read_operations: u64,
    pub(super) write_operations: u64,
    pub(super) other_operations: u64,
    pub(super) read_bytes: u64,
    pub(super) write_bytes: u64,
    pub(super) other_bytes: u64,
}

impl Snapshot {
    pub(super) fn current() -> io::Result<Self> {
        let mut memory = MemoryCounters {
            cb: size_of::<MemoryCounters>() as u32,
            ..Default::default()
        };
        let mut io_counters = IoCounters::default();
        let mut creation = FileTime::default();
        let mut exit = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        // SAFETY: The current-process pseudo handle is valid and never closed here. All
        // repr(C) output structures are initialized, correctly sized and live through calls.
        unsafe {
            let process = GetCurrentProcess();
            if GetProcessMemoryInfo(process, &mut memory, size_of::<MemoryCounters>() as u32) == 0 {
                return Err(io::Error::last_os_error());
            }
            if GetProcessIoCounters(process, &mut io_counters) == 0 {
                return Err(io::Error::last_os_error());
            }
            if GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(Self {
            user_100ns: user.ticks(),
            kernel_100ns: kernel.ticks(),
            rss_bytes: memory.working_set_size,
            lifetime_peak_rss_bytes: memory.peak_working_set_size,
            read_operations: io_counters.read_operations,
            write_operations: io_counters.write_operations,
            other_operations: io_counters.other_operations,
            read_bytes: io_counters.read_bytes,
            write_bytes: io_counters.write_bytes,
            other_bytes: io_counters.other_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Delta {
    pub(super) user_100ns: u64,
    pub(super) kernel_100ns: u64,
    pub(super) read_operations: u64,
    pub(super) write_operations: u64,
    pub(super) other_operations: u64,
    pub(super) read_bytes: u64,
    pub(super) write_bytes: u64,
    pub(super) other_bytes: u64,
}
impl Delta {
    pub(super) fn between(before: Snapshot, after: Snapshot) -> Self {
        let difference =
            |first: u64, last: u64| last.checked_sub(first).expect("process counter regressed");
        Self {
            user_100ns: difference(before.user_100ns, after.user_100ns),
            kernel_100ns: difference(before.kernel_100ns, after.kernel_100ns),
            read_operations: difference(before.read_operations, after.read_operations),
            write_operations: difference(before.write_operations, after.write_operations),
            other_operations: difference(before.other_operations, after.other_operations),
            read_bytes: difference(before.read_bytes, after.read_bytes),
            write_bytes: difference(before.write_bytes, after.write_bytes),
            other_bytes: difference(before.other_bytes, after.other_bytes),
        }
    }
}
