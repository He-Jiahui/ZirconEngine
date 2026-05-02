use crate::buffer::ZrByteSlice;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeTargetMode {
    ClientRuntime = 1,
    ServerRuntime = 2,
    EditorHost = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrPluginModuleKind {
    Runtime = 1,
    Editor = 2,
    Native = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrPluginModuleDescriptorV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub name: ZrByteSlice,
    pub crate_name: ZrByteSlice,
    pub target_modes: ZrByteSlice,
    pub capabilities: ZrByteSlice,
}

impl ZrPluginModuleDescriptorV1 {
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            kind: ZrPluginModuleKind::Runtime as u32,
            name: ZrByteSlice::empty(),
            crate_name: ZrByteSlice::empty(),
            target_modes: ZrByteSlice::empty(),
            capabilities: ZrByteSlice::empty(),
        }
    }
}
