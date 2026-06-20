use super::super::super::paint_frame::HostPaintAtlasImage;

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct HostPaintImagePixels {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) resource_key: String,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) width: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) height: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) rgba: Vec<u8>,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) atlas:
        Option<HostPaintAtlasImage>,
}

impl HostPaintImagePixels {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn with_resource_key(
        mut self,
        resource_key: impl Into<String>,
    ) -> Self {
        self.resource_key = resource_key.into();
        self
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn with_atlas(
        mut self,
        atlas: Option<HostPaintAtlasImage>,
    ) -> Self {
        self.atlas = atlas;
        self
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_valid(
        &self,
    ) -> bool {
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }
}
