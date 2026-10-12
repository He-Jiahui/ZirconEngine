/// Stateless facade over the process-lifetime native plugin discovery and load authorities.
///
/// Discovery mirrors Unreal's process-wide plugin manager and exists before any Core runtime
/// generation. Runtime-scoped dynamic-library lifetime remains owned by native host handles.
/// 可复制的调用入口，不拥有库；发现 authority 贯穿进程，运行时加载/卸载由 host handle 管理。
/// 普通加载入口使用 deny-all 授权；执行已签名或内置产物须选择带明确 authority 的调用路径。
#[derive(Clone, Debug, Default)]
pub struct NativePluginLoader;
