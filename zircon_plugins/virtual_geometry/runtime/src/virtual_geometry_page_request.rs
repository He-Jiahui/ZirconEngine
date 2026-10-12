#[derive(Clone, Debug, PartialEq, Eq)]
/// 保存待上传页的大小和发起代次，供准备帧生成 GPU 请求。
pub(crate) struct VirtualGeometryPageRequest {
    page_id: u32,
    size_bytes: u64,
    generation: u64,
}

impl VirtualGeometryPageRequest {
    pub(in crate::virtual_geometry) fn new(page_id: u32, size_bytes: u64, generation: u64) -> Self {
        Self {
            page_id,
            size_bytes,
            generation,
        }
    }

    pub(crate) fn page_id(&self) -> u32 {
        self.page_id
    }

    pub(crate) fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}
