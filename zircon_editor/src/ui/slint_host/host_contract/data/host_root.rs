use super::{
    HostNativeFloatingWindowSurfaceData, HostWindowLayoutData, HostWindowSceneData,
    HostWindowShellData,
};

#[derive(Clone, Default)]
pub(crate) struct HostWindowPresentationData {
    pub host_shell: HostWindowShellData,
    pub host_layout: HostWindowLayoutData,
    pub host_scene_data: HostWindowSceneData,
    pub native_floating_surface_data: HostNativeFloatingWindowSurfaceData,
}
