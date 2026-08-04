#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderNativeSurfaceTarget {
    Win32 { hwnd: u64, hinstance: Option<u64> },
}
