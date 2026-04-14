use std::sync::{Arc, Mutex};

use slint::Image;
use zircon_core::CoreHandle;
use zircon_graphics::{
    create_shared_texture_render_service, EditorOrRuntimeFrame, GraphicsError,
    SharedTextureRenderService, ViewportFrameTextureHandle,
};

use crate::ViewportTextureBridge;

#[derive(Clone)]
pub(crate) struct SlintViewportController {
    shared: Arc<Mutex<ViewportState>>,
}

impl SlintViewportController {
    pub(crate) fn new(core: CoreHandle) -> Self {
        Self {
            shared: Arc::new(Mutex::new(ViewportState {
                core,
                service: None,
                pending_frame: None,
                latest_generation: None,
                latest_image: None,
                last_error: None,
            })),
        }
    }

    pub(crate) fn attach_renderer(
        &self,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<(), GraphicsError> {
        let mut shared = self.shared.lock().unwrap();
        if shared.service.is_some() {
            return Ok(());
        }
        let service = create_shared_texture_render_service(&shared.core, device, queue)?;
        if let Some(frame) = shared.pending_frame.take() {
            service.submit_frame(frame)?;
        }
        shared.service = Some(service);
        Ok(())
    }

    pub(crate) fn detach_renderer(&self) {
        let mut shared = self.shared.lock().unwrap();
        shared.service = None;
        shared.latest_generation = None;
        shared.latest_image = None;
    }

    pub(crate) fn submit_frame(&self, frame: EditorOrRuntimeFrame) -> Result<(), GraphicsError> {
        let mut shared = self.shared.lock().unwrap();
        if let Some(service) = shared.service.as_ref() {
            service.submit_frame(frame)
        } else {
            shared.pending_frame = Some(frame);
            Ok(())
        }
    }

    pub(crate) fn poll_image(&self) -> Option<Image> {
        let mut shared = self.shared.lock().unwrap();
        let Some(service) = shared.service.as_ref() else {
            return shared.latest_image.clone();
        };
        let Some(frame) = service.try_recv_latest_frame() else {
            return shared.latest_image.clone();
        };
        if shared.latest_generation == Some(frame.generation) {
            return shared.latest_image.clone();
        }
        match import_frame_image(frame) {
            Ok(image) => {
                shared.latest_generation = Some(image.0);
                shared.latest_image = Some(image.1.clone());
                shared.latest_image.clone()
            }
            Err(error) => {
                shared.last_error = Some(error);
                shared.latest_image.clone()
            }
        }
    }

    pub(crate) fn take_error(&self) -> Option<String> {
        self.shared.lock().unwrap().last_error.take()
    }
}

struct ViewportState {
    core: CoreHandle,
    service: Option<SharedTextureRenderService>,
    pending_frame: Option<EditorOrRuntimeFrame>,
    latest_generation: Option<u64>,
    latest_image: Option<Image>,
    last_error: Option<String>,
}

fn import_frame_image(frame: ViewportFrameTextureHandle) -> Result<(u64, Image), String> {
    ViewportTextureBridge::validate_metadata(frame.width, frame.height, frame.format, frame.usage)
        .map_err(|error| error.to_string())?;
    Image::try_from(frame.texture)
        .map(|image| (frame.generation, image))
        .map_err(|error| error.to_string())
}
