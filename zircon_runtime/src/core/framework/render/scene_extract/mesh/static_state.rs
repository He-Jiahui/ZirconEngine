#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderMeshStaticState {
    pub transform_static: bool,
    pub geometry_revision: u64,
    pub material_revision: u64,
}

impl RenderMeshStaticState {
    pub const fn new(
        transform_static: bool,
        geometry_revision: u64,
        material_revision: u64,
    ) -> Self {
        Self {
            transform_static,
            geometry_revision,
            material_revision,
        }
    }

    pub const fn from_transform_static(transform_static: bool) -> Self {
        Self {
            transform_static,
            geometry_revision: 0,
            material_revision: 0,
        }
    }

    pub const fn has_authoritative_revisions(self) -> bool {
        self.transform_static && self.geometry_revision != 0 && self.material_revision != 0
    }
}
