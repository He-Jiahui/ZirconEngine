/// 当前安装的存储后端类别；Unavailable 不会退化为进程内持久化假象。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreferenceStorageBackendKind {
    Unavailable,
    AtomicFile,
    HostProvided,
}

impl PreferenceStorageBackendKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::AtomicFile => "atomic_file",
            Self::HostProvided => "host_provided",
        }
    }

    /// 表示具备持久化后端，不表示某次异步写入已经落盘；后者须看终态票据。
    pub const fn is_persistent(self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}
