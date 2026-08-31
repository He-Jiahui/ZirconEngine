use serde::{Deserialize, Serialize};

use super::{TextureHandle, TextureViewHandle};

/// Color value used when a render pass clears a color attachment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RenderClearColor {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderPassColorLoadOp {
    Load,
    Clear(RenderClearColor),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderPassDepthLoadOp {
    Load,
    Clear(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderPassStencilLoadOp {
    Load,
    Clear(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderPassStoreOp {
    Store,
    Discard,
}

/// Identifies the texture subresource used by a render-pass attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPassTextureViewDesc {
    pub texture: TextureHandle,
    pub mip_level: u32,
    pub array_layer: u32,
    /// Reuses a registered single-subresource D2 view. This lets transient
    /// surface frames expose their native default view without allocating a
    /// second native view while encoding the final render pass.
    #[serde(default)]
    pub registered_view: Option<TextureViewHandle>,
}

impl RenderPassTextureViewDesc {
    pub const fn new(texture: TextureHandle) -> Self {
        Self {
            texture,
            mip_level: 0,
            array_layer: 0,
            registered_view: None,
        }
    }

    pub const fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = mip_level;
        self
    }

    pub const fn with_array_layer(mut self, array_layer: u32) -> Self {
        self.array_layer = array_layer;
        self
    }

    pub const fn with_registered_view(mut self, registered_view: TextureViewHandle) -> Self {
        self.registered_view = Some(registered_view);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPassColorAttachmentDesc {
    pub view: RenderPassTextureViewDesc,
    pub resolve_target: Option<RenderPassTextureViewDesc>,
    pub load: RenderPassColorLoadOp,
    pub store: RenderPassStoreOp,
}

impl RenderPassColorAttachmentDesc {
    pub const fn new(
        texture: TextureHandle,
        load: RenderPassColorLoadOp,
        store: RenderPassStoreOp,
    ) -> Self {
        Self {
            view: RenderPassTextureViewDesc::new(texture),
            resolve_target: None,
            load,
            store,
        }
    }

    pub const fn with_view(mut self, view: RenderPassTextureViewDesc) -> Self {
        self.view = view;
        self
    }

    pub const fn with_resolve_target(mut self, resolve_target: TextureHandle) -> Self {
        self.resolve_target = Some(RenderPassTextureViewDesc::new(resolve_target));
        self
    }

    pub const fn with_resolve_view(mut self, resolve_target: RenderPassTextureViewDesc) -> Self {
        self.resolve_target = Some(resolve_target);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPassDepthStencilAttachmentDesc {
    pub view: RenderPassTextureViewDesc,
    pub depth_load: RenderPassDepthLoadOp,
    pub depth_store: RenderPassStoreOp,
    pub stencil_load: Option<RenderPassStencilLoadOp>,
    pub stencil_store: Option<RenderPassStoreOp>,
}

impl RenderPassDepthStencilAttachmentDesc {
    pub const fn depth(
        texture: TextureHandle,
        depth_load: RenderPassDepthLoadOp,
        depth_store: RenderPassStoreOp,
    ) -> Self {
        Self {
            view: RenderPassTextureViewDesc::new(texture),
            depth_load,
            depth_store,
            stencil_load: None,
            stencil_store: None,
        }
    }

    pub const fn with_stencil(
        mut self,
        stencil_load: RenderPassStencilLoadOp,
        stencil_store: RenderPassStoreOp,
    ) -> Self {
        self.stencil_load = Some(stencil_load);
        self.stencil_store = Some(stencil_store);
        self
    }

    pub const fn with_view(mut self, view: RenderPassTextureViewDesc) -> Self {
        self.view = view;
        self
    }
}
