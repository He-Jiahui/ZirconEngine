use crate::core::resource::{ResourceData, ResourceMarker};

/// 泛型资产 API 复用 ResourceManager 的 kind 路由；共享 marker 的资产仍须在读取时核对实际载荷类型。
/// Typed asset contract layered over the existing resource marker model.
pub trait Asset: ResourceData + Clone + 'static {
    type Marker: ResourceMarker;

    const LABEL: &'static str;
}
