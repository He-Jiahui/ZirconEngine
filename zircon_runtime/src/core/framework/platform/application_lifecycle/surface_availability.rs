/// 应用是否有可用原生表面的宿主观察；运行态本身不承诺可以呈现。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationSurfaceAvailability {
    Unknown,
    Available,
    #[default]
    Unavailable,
}
