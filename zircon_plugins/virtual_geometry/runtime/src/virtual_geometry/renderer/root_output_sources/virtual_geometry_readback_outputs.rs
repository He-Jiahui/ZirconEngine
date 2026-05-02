use crate::virtual_geometry::renderer::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackCompletionParts,
};

#[derive(Default)]
pub(super) struct VirtualGeometryReadbackOutputs {
    gpu_readback: Option<VirtualGeometryGpuReadback>,
}

impl VirtualGeometryReadbackOutputs {
    pub(in crate::virtual_geometry::renderer) fn store_gpu_readback(
        &mut self,
        readback: Option<VirtualGeometryGpuReadback>,
    ) {
        self.gpu_readback = readback;
    }

    pub(in crate::virtual_geometry::renderer) fn has_gpu_readback(&self) -> bool {
        self.gpu_readback.is_some()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryGpuReadback> {
        self.gpu_readback.as_mut()
    }

    pub(in crate::virtual_geometry::renderer) fn gpu_readback(
        &self,
    ) -> Option<&VirtualGeometryGpuReadback> {
        self.gpu_readback.as_ref()
    }

    pub(in crate::virtual_geometry::renderer) fn take_gpu_completion_parts(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
        self.gpu_readback
            .take()
            .map(VirtualGeometryGpuReadback::into_completion_parts)
    }

    #[cfg(test)]
    pub(crate) fn take_gpu_readback(&mut self) -> Option<VirtualGeometryGpuReadback> {
        self.gpu_readback.take()
    }
}
