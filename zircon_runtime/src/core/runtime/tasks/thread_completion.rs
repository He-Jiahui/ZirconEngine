use std::thread::JoinHandle;

/// Checks readiness without consuming the owner's permission to join.
///
/// Windows requires native thread termination, including TLS destruction. Other platforms
/// retain the standard library's main-function completion check; that check alone cannot
/// establish a hard deadline when thread-local destructors block.
pub fn thread_is_join_ready<T>(thread: &JoinHandle<T>) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
        use windows_sys::Win32::System::Threading::WaitForSingleObject;

        // The borrowed JoinHandle keeps the native handle open throughout this zero-time probe.
        unsafe { WaitForSingleObject(thread.as_raw_handle(), 0) == WAIT_OBJECT_0 }
    }
    #[cfg(not(windows))]
    {
        thread.is_finished()
    }
}

#[cfg(all(test, windows))]
#[path = "thread_completion/tests/cases.rs"]
mod tests;
