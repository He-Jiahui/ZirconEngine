/// Stateless facade over the process-lifetime native plugin discovery and load authorities.
///
/// Discovery mirrors Unreal's process-wide plugin manager and exists before any Core runtime
/// generation. Runtime-scoped dynamic-library lifetime remains owned by native host handles.
#[derive(Clone, Debug, Default)]
pub struct NativePluginLoader;
