//! Optional readback owned by the actual normal presenter and existing device timeline.
use super::{
    WgpuUiExternalImage, WgpuUiExternalImageAlphaMode, WgpuUiSurfaceDiagnosticCompletionOwner,
    WgpuUiSurfaceRenderer,
};
use std::sync::{Arc, Mutex};
use web_time::{Duration, Instant};
use zr_rhi::{
    DiagnosticReadbackAdmission, DiagnosticReadbackKind, DiagnosticReadbackReceipt,
    DiagnosticReadbackRequestId, DiagnosticReadbackTerminal, RenderDevice, RhiError,
    SubmissionTicket, UiSurfaceDrawList, UiSurfaceFramePixelFormat, UiSurfaceFrameReadback,
    UiSurfaceFrameReadbackRequest, UiSurfaceImageAlphaMode, UiSurfaceImageReadback,
    UiSurfaceImageReadbackRequest, UiSurfaceRect, UiSurfaceResolvedCommandKind,
};

type Delivery = Arc<Mutex<Option<(DiagnosticReadbackReceipt, Option<Vec<u8>>)>>>;
#[derive(Default)]
pub(super) struct ReadbackSlot {
    id: Option<DiagnosticReadbackRequestId>,
    rejected_id: Option<DiagnosticReadbackRequestId>,
    delivery: Delivery,
}
impl ReadbackSlot {
    pub(super) fn admit(&mut self, admission: DiagnosticReadbackAdmission) -> bool {
        match admission {
            DiagnosticReadbackAdmission::Admitted(id) => {
                self.id = Some(id);
                true
            }
            DiagnosticReadbackAdmission::Rejected(receipt) => {
                self.rejected_id = Some(receipt.request());
                self.deliver(receipt, None);
                false
            }
        }
    }
    pub(super) fn deliver(&self, receipt: DiagnosticReadbackReceipt, bytes: Option<Vec<u8>>) {
        *self.delivery.lock().unwrap_or_else(|p| p.into_inner()) = Some((receipt, bytes));
    }
    fn has_delivery(&self) -> bool {
        self.delivery
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_some()
    }
    pub(super) fn register(
        &self,
        owner: &dyn WgpuUiSurfaceDiagnosticCompletionOwner,
    ) -> Result<(), String> {
        let Some(id) = self.id else {
            return Ok(());
        };
        let delivery = Arc::clone(&self.delivery);
        match owner.register_receipt_callback(
            DiagnosticReadbackAdmission::Admitted(id),
            Box::new(move |receipt, bytes| {
                *delivery.lock().unwrap_or_else(|p| p.into_inner()) = Some((receipt, bytes));
            }),
        ) {
            Ok(true) => Ok(()),
            Ok(false) => Err("receipt route was not admitted".into()),
            Err(error) => Err(format!("receipt route failed: {error}")),
        }
    }
    fn cancel_route(&self, owner: Option<&dyn WgpuUiSurfaceDiagnosticCompletionOwner>) {
        if let (Some(owner), Some(id)) = (owner, self.id) {
            owner.cancel_receipt_callback(id);
        }
    }
    fn collect_local(&self, device: &crate::WgpuRenderDevice) {
        if !self.has_delivery() {
            if let Some(delivery) = self
                .id
                .and_then(|id| device.take_diagnostic_readback_delivery_for(id))
            {
                self.deliver(delivery.receipt(), delivery.into_bytes());
            }
        }
    }
    fn retire_local_rejection(&self, device: &crate::WgpuRenderDevice) {
        if let Some(id) = self.rejected_id {
            // Rejections are delivered before submitted maps. Remove only this owned terminal
            // entry so a rejected second request cannot block the first request's ordered ring.
            let _ = device.take_diagnostic_readback_delivery_for(id);
        }
    }
    fn take(&self) -> (Option<DiagnosticReadbackReceipt>, Option<Vec<u8>>) {
        let observed = self
            .delivery
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        let receipt = observed.as_ref().map(|(receipt, _)| *receipt);
        (receipt, observed.and_then(|(_, bytes)| bytes))
    }
}

pub(super) struct PendingImageReadback {
    pub(super) slot: ReadbackSlot,
    request: UiSurfaceImageReadbackRequest,
    command_frame: Option<UiSurfaceRect>,
    command_clip: Option<UiSurfaceRect>,
    format: Option<UiSurfaceFramePixelFormat>,
    alpha_mode: Option<UiSurfaceImageAlphaMode>,
    // Retain the exact provider generation, including its shared allocation pin, through delivery.
    source: Option<WgpuUiExternalImage>,
}
impl PendingImageReadback {
    pub(super) fn prepare(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        source: WgpuUiExternalImage,
    ) -> Result<(), String> {
        let (frame, clip) = image_command_geometry(draw_list, &self.request)?;
        let (format, alpha) = image_texture_format(&source, &self.request)?;
        self.command_frame = Some(frame);
        self.command_clip = clip;
        self.format = Some(format);
        self.alpha_mode = Some(alpha);
        self.source = Some(source);
        Ok(())
    }
}

pub(super) struct PendingFrameReadback {
    request: UiSurfaceFrameReadbackRequest,
    deadline: Instant,
    epoch: u64,
    present_index: u64,
    pub(super) window: ReadbackSlot,
    pub(super) image: Option<PendingImageReadback>,
    submission: Option<SubmissionTicket>,
    format: Option<UiSurfaceFramePixelFormat>,
    failure: Option<String>,
    cancelled: bool,
}
impl PendingFrameReadback {
    pub(super) fn new(request: UiSurfaceFrameReadbackRequest, epoch: u64) -> Self {
        let image = request.image.as_ref().map(|request| PendingImageReadback {
            slot: ReadbackSlot::default(),
            request: request.clone(),
            command_frame: None,
            command_clip: None,
            format: None,
            alpha_mode: None,
            source: None,
        });
        Self {
            deadline: Instant::now() + Duration::from_millis(request.deadline_ms),
            request,
            epoch,
            present_index: 0,
            window: ReadbackSlot::default(),
            image,
            submission: None,
            format: None,
            failure: None,
            cancelled: false,
        }
    }
    pub(super) fn fail(&mut self, reason: String) {
        self.failure.get_or_insert(reason);
    }
    pub(super) fn prepare_window(&mut self, format: UiSurfaceFramePixelFormat, present_index: u64) {
        self.format = Some(format);
        self.present_index = present_index;
    }
    pub(super) fn bind_submission(&mut self, ticket: SubmissionTicket) {
        // A first admitted copy still needs its real ticket when the second was rejected.
        if self.window.id.is_some()
            || self
                .image
                .as_ref()
                .is_some_and(|image| image.slot.id.is_some())
        {
            self.submission = Some(ticket);
        }
    }
    pub(super) fn ready(&self) -> bool {
        if self.cancelled {
            return true;
        }
        let admitted = self.window.id.is_some()
            || self
                .image
                .as_ref()
                .is_some_and(|image| image.slot.id.is_some());
        if !admitted {
            return self.failure.is_some();
        }
        self.submission.is_some()
            && (self.window.id.is_none() || self.window.has_delivery())
            && self
                .image
                .as_ref()
                .is_none_or(|image| image.slot.id.is_none() || image.slot.has_delivery())
    }
    fn cancel_routes(&self, owner: Option<&dyn WgpuUiSurfaceDiagnosticCompletionOwner>) {
        self.window.cancel_route(owner);
        if let Some(image) = &self.image {
            image.slot.cancel_route(owner);
        }
    }
    pub(super) fn cancel(
        &mut self,
        reason: &str,
        owner: Option<&dyn WgpuUiSurfaceDiagnosticCompletionOwner>,
    ) {
        self.fail(reason.to_owned());
        self.cancelled = true;
        // Detach consumers only; the diagnostic service still owns native copies to terminal.
        self.cancel_routes(owner);
    }
    pub(super) fn collect_local(&self, device: &crate::WgpuRenderDevice) {
        self.window.retire_local_rejection(device);
        if let Some(image) = &self.image {
            image.slot.retire_local_rejection(device);
        }
        // The device ring is ordered: drain this window before this image, never foreign IDs.
        self.window.collect_local(device);
        if let Some(image) = &self.image {
            image.slot.collect_local(device);
        }
    }
    pub(super) fn finish(
        mut self,
        surface_alpha_mode: String,
        native_target: Option<zr_rhi::RenderNativeSurfaceTarget>,
    ) -> UiSurfaceFrameReadback {
        let (receipt, mut bytes) = self.window.take();
        let opaque_pixels = bytes
            .as_ref()
            .map(|bytes| bytes.chunks_exact(4).all(|p| p[3] == 255));
        if self.failure.is_none() {
            if let Err(reason) = validate_delivery(
                &self.window,
                receipt,
                self.submission,
                self.request.physical_size,
                bytes.as_deref(),
            ) {
                self.fail(format!("normal {reason}"));
            }
        }
        let mut image_result = self.image.take().map(|image| {
            let (receipt, bytes) = image.slot.take();
            let mut failure = validate_delivery(
                &image.slot,
                receipt,
                self.submission,
                image.request.physical_size,
                bytes.as_deref(),
            )
            .err();
            if failure.is_none()
                && (image.command_frame.is_none()
                    || image.format.is_none()
                    || image.alpha_mode.is_none()
                    || image.source.is_none())
            {
                failure = Some("readback external resource was not prepared".into());
            }
            UiSurfaceImageReadback {
                request: image.request,
                command_frame: image.command_frame,
                command_clip: image.command_clip,
                receipt,
                pixel_format: image.format,
                alpha_mode: image.alpha_mode,
                opaque_pixels: bytes
                    .as_ref()
                    .map(|bytes| bytes.chunks_exact(4).all(|p| p[3] == 255)),
                color8: bytes,
                failure,
            }
        });
        if let Some(image) = &image_result {
            if let Some(reason) = &image.failure {
                self.fail(format!("image {reason}"));
            }
        }
        if let Some(reason) = &self.failure {
            bytes = None;
            if let Some(image) = &mut image_result {
                image.color8 = None;
                image.failure.get_or_insert_with(|| reason.clone());
            }
        }
        UiSurfaceFrameReadback {
            request: self.request,
            surface_epoch: self.epoch,
            present_index: self.present_index,
            submission: self.submission,
            receipt,
            pixel_format: self.format,
            color8: bytes,
            opaque_pixels,
            surface_alpha_mode,
            native_target,
            image: image_result,
            failure: self.failure,
        }
    }
}
fn validate_delivery(
    slot: &ReadbackSlot,
    receipt: Option<DiagnosticReadbackReceipt>,
    submission: Option<SubmissionTicket>,
    size: (u32, u32),
    bytes: Option<&[u8]>,
) -> Result<(), String> {
    if !receipt.is_some_and(|r| {
        Some(r.request()) == slot.id
            && slot.id.is_some()
            && submission.is_some()
            && r.kind() == DiagnosticReadbackKind::Texture
            && r.terminal() == DiagnosticReadbackTerminal::Succeeded
            && r.frame_key().map(|key| key.submission()) == submission
    }) {
        return Err("readback receipt does not match successful actual submission".into());
    }
    let expected = u64::from(size.0)
        .checked_mul(u64::from(size.1))
        .and_then(|pixels| pixels.checked_mul(4));
    if expected.is_none_or(|expected| {
        expected == 0 || bytes.is_none_or(|bytes| bytes.len() as u64 != expected)
    }) {
        return Err("readback payload extent invalid".into());
    }
    Ok(())
}

pub(super) fn image_command_geometry(
    draw_list: &UiSurfaceDrawList,
    request: &UiSurfaceImageReadbackRequest,
) -> Result<(UiSurfaceRect, Option<UiSurfaceRect>), String> {
    if request.resource_key.is_empty()
        || request.physical_size.0 == 0
        || request.physical_size.1 == 0
    {
        return Err("image readback resource identity or extent invalid".into());
    }
    let command = draw_list
        .commands
        .get(request.command_index)
        .ok_or_else(|| "image readback command index unavailable".to_owned())?;
    let Some(UiSurfaceResolvedCommandKind::Image { payload }) = draw_list.resolved_kind(command)
    else {
        return Err("image readback command is not an image".into());
    };
    if payload.resource_key != request.resource_key
        || payload.resource_generation != request.resource_generation
        || (payload.width, payload.height) != request.physical_size
        || payload.atlas_uv.is_some()
        || payload.rgba.is_some()
        || draw_list
            .image_resource(&request.resource_key, request.resource_generation)
            .is_some()
    {
        return Err("image readback command is not the exact whole external resource".into());
    }
    // Match image clipping without building a second draw plan or shaping unrelated text.
    let size = draw_list.projection_size();
    let mut visible = command.frame;
    for bound in [
        Some(UiSurfaceRect::new(0.0, 0.0, size.0 as f32, size.1 as f32)),
        command.clip,
        super::geometry::damage_with_analytic_coverage(draw_list.damage, size),
    ]
    .into_iter()
    .flatten()
    {
        if !visible.has_finite_positive_area() || !bound.has_finite_positive_area() {
            return Err("image readback command geometry invalid".into());
        }
        let left = visible.x.max(bound.x);
        let top = visible.y.max(bound.y);
        let right = (visible.x + visible.width).min(bound.x + bound.width);
        let bottom = (visible.y + visible.height).min(bound.y + bound.height);
        if right <= left || bottom <= top {
            return Err("image readback command has no visible submitted geometry".into());
        }
        visible = UiSurfaceRect::new(left, top, right - left, bottom - top);
    }
    Ok((command.frame, command.clip))
}
pub(super) fn image_texture_format(
    image: &WgpuUiExternalImage,
    request: &UiSurfaceImageReadbackRequest,
) -> Result<(UiSurfaceFramePixelFormat, UiSurfaceImageAlphaMode), String> {
    let texture = &image.texture;
    if !image.matches_generation(request.resource_generation)
        || (image.width, image.height) != request.physical_size
        || texture.width() != image.width
        || texture.height() != image.height
        || texture.depth_or_array_layers() != 1
        || texture.dimension() != wgpu::TextureDimension::D2
        || texture.sample_count() != 1
        || !texture.usage().contains(wgpu::TextureUsages::COPY_SRC)
    {
        return Err(
            "image readback external texture generation, extent or COPY_SRC invalid".into(),
        );
    }
    let format = frame_pixel_format(texture.format())
        .ok_or_else(|| "image readback external pixel format unsupported".to_owned())?;
    if request
        .expected_pixel_format
        .is_some_and(|expected| expected != format)
    {
        return Err("image readback external pixel format changed".into());
    }
    let alpha = match image.alpha_mode() {
        WgpuUiExternalImageAlphaMode::Opaque => UiSurfaceImageAlphaMode::Opaque,
        WgpuUiExternalImageAlphaMode::Premultiplied => UiSurfaceImageAlphaMode::Premultiplied,
    };
    Ok((format, alpha))
}
fn frame_pixel_format(format: wgpu::TextureFormat) -> Option<UiSurfaceFramePixelFormat> {
    Some(match format {
        wgpu::TextureFormat::Rgba8UnormSrgb => UiSurfaceFramePixelFormat::Rgba8Srgb,
        wgpu::TextureFormat::Bgra8UnormSrgb => UiSurfaceFramePixelFormat::Bgra8Srgb,
        wgpu::TextureFormat::Rgba8Unorm => UiSurfaceFramePixelFormat::Rgba8Linear,
        wgpu::TextureFormat::Bgra8Unorm => UiSurfaceFramePixelFormat::Bgra8Linear,
        _ => return None,
    })
}

impl WgpuUiSurfaceRenderer {
    pub(super) fn has_submitted_frame_readback(&self) -> bool {
        self.frame_readback
            .as_ref()
            .is_some_and(|pending| pending.submission.is_some())
    }
    pub(super) fn request_frame_readback(
        &mut self,
        request: UiSurfaceFrameReadbackRequest,
    ) -> Result<(), RhiError> {
        if self.frame_readback.is_some()
            || self.consumed_readback_nonces.len() >= 256
            || self.consumed_readback_nonces.contains(&request.nonce)
        {
            return Err(RhiError::SurfaceUnavailable(
                "normal readback request already pending or nonce consumed".into(),
            ));
        }
        if request.nonce.is_empty()
            || request.nonce.len() > 96
            || !request
                .nonce
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !(50..=10000).contains(&request.deadline_ms)
            || request.physical_size != (self.config.width, self.config.height)
        {
            return Err(RhiError::SurfaceUnavailable(
                "normal readback request identity, extent or deadline invalid".into(),
            ));
        }
        if !self.completion_owner.is_local() && self.diagnostic_completion_owner.is_none() {
            return Err(RhiError::SurfaceUnavailable(
                "external normal readback completion owner unavailable".into(),
            ));
        }
        self.consumed_readback_nonces.push(request.nonce.clone());
        self.frame_readback = Some(PendingFrameReadback::new(request, self.surface_epoch));
        Ok(())
    }
    pub(super) fn cancel_frame_readback(&mut self, reason: &str) {
        if let Some(pending) = self.frame_readback.as_mut() {
            pending.cancel(reason, self.diagnostic_completion_owner.as_deref());
        }
    }
    pub(super) fn prepare_frame_readback(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<crate::WgpuNativeDiagnosticReadbackFrame> {
        let pending = self.frame_readback.as_ref()?;
        if pending.failure.is_some() || pending.window.id.is_some() {
            return None;
        }
        if Instant::now() >= pending.deadline
            || pending.epoch != self.surface_epoch
            || pending.request.physical_size != draw_list.surface_size
            || pending.request.expected_draw_generation != draw_list.generation()
            || draw_list.is_target_only_resize()
        {
            self.cancel_frame_readback("normal readback deadline, extent or generation changed");
            return None;
        }
        let Some(cache) = self
            .retained_cache
            .as_ref()
            .filter(|cache| cache.matches(self.config.format, draw_list.surface_size))
        else {
            self.cancel_frame_readback(
                "surface does not support exact retained-cache COPY_DST presentation",
            );
            return None;
        };
        let texture = cache.texture().clone();
        let Some(format) = frame_pixel_format(texture.format()) else {
            self.cancel_frame_readback("surface pixel format unsupported");
            return None;
        };
        if let Some(request) = pending.request.image.as_ref() {
            let source = self.external_images.as_ref().and_then(|provider| {
                provider.resolve(&request.resource_key, request.resource_generation)
            });
            let failure = if !self
                .image_cache
                .is_resident(&request.resource_key, request.resource_generation)
            {
                Some("image readback command resource was not resident for this present".to_owned())
            } else if let Some(source) = source {
                self.frame_readback
                    .as_mut()
                    .unwrap()
                    .image
                    .as_mut()
                    .unwrap()
                    .prepare(draw_list, source)
                    .err()
            } else {
                Some("image readback exact external provider generation unavailable".to_owned())
            };
            if let Some(reason) = failure {
                self.cancel_frame_readback(&reason);
                return None;
            }
        }
        // Never consume or clear an already admitted scene batch. begin_frame rejects it.
        if let Err(error) = self
            .render_device
            .begin_diagnostic_readback_frame(self.present_index)
        {
            self.cancel_frame_readback(&format!("normal diagnostic batch unavailable: {error}"));
            return None;
        }
        let admission = self
            .render_device
            .enqueue_native_diagnostic_texture_color8_readback(
                &texture,
                draw_list.surface_size.0,
                draw_list.surface_size.1,
            );
        let pending = self.frame_readback.as_mut().expect("request remains owned");
        pending.prepare_window(format, self.present_index);
        match admission {
            Ok(admission) => {
                if !pending.window.admit(admission) {
                    pending.fail("normal readback quota rejected".into());
                } else if let Some(owner) = self.diagnostic_completion_owner.as_deref() {
                    if let Err(reason) = pending.window.register(owner) {
                        let reason = format!("normal {reason}");
                        if pending.image.is_none() {
                            pending.cancel(&reason, Some(owner));
                        } else {
                            pending.fail(reason);
                        }
                    }
                }
            }
            Err(error) => pending.fail(format!("normal color readback admission failed: {error}")),
        }
        if pending.failure.is_none() {
            if let Some(image) = pending.image.as_mut() {
                let source = image
                    .source
                    .as_ref()
                    .expect("external image validated before admission");
                let admission = self
                    .render_device
                    .enqueue_native_diagnostic_texture_color8_readback(
                        &source.texture,
                        image.request.physical_size.0,
                        image.request.physical_size.1,
                    );
                let failure = match admission {
                    Ok(admission) => {
                        if !image.slot.admit(admission) {
                            Some("image readback quota rejected".to_owned())
                        } else {
                            self.diagnostic_completion_owner
                                .as_deref()
                                .and_then(|owner| image.slot.register(owner).err())
                                .map(|reason| format!("image {reason}"))
                        }
                    }
                    Err(error) => Some(format!("image color readback admission failed: {error}")),
                };
                if let Some(reason) = failure {
                    pending.fail(reason);
                }
            }
        }
        // Seal once even on partial admission failure. The same normal encoder must submit all
        // admitted copies and bind the real ticket before their terminal deliveries are retired.
        match self
            .render_device
            .prepare_native_diagnostic_readback_frame("zircon-normal-surface-readback", encoder)
        {
            Ok(frame) => frame,
            Err(error) => {
                self.cancel_frame_readback(&format!("normal readback encoding failed: {error}"));
                None
            }
        }
    }
    pub(super) fn bind_frame_readback_submission(&mut self, ticket: SubmissionTicket) {
        if let Some(pending) = self.frame_readback.as_mut() {
            pending.bind_submission(ticket);
        }
    }
    pub(super) fn drain_frame_readback(
        &mut self,
    ) -> Result<Option<UiSurfaceFrameReadback>, RhiError> {
        let Some(pending) = self.frame_readback.as_ref() else {
            return Ok(None);
        };
        if Instant::now() >= pending.deadline {
            self.cancel_frame_readback("normal readback completion deadline expired");
        }
        if self.frame_readback.as_ref().is_some_and(|p| {
            p.epoch != self.surface_epoch
                || p.request.physical_size != (self.config.width, self.config.height)
                || (p.submission.is_some() && p.present_index != self.present_index)
        }) {
            self.cancel_frame_readback(
                "normal readback surface epoch, extent or actual present changed",
            );
        }
        // Soft failures still settle admitted work through the original completion owner.
        if self
            .frame_readback
            .as_ref()
            .is_some_and(|p| !p.cancelled && p.submission.is_some())
        {
            let polled = if self.completion_owner.is_local() {
                self.render_device.poll_submissions()
            } else {
                self.diagnostic_completion_owner
                    .as_ref()
                    .expect("external owner checked at admission")
                    .drain_once()
            };
            if let Err(error) = polled {
                self.cancel_frame_readback(&format!("normal completion owner failed: {error}"));
            }
        }
        if self.completion_owner.is_local() {
            self.frame_readback
                .as_ref()
                .unwrap()
                .collect_local(&self.render_device);
        }
        if !self.frame_readback.as_ref().unwrap().ready() {
            return Ok(None);
        }
        let pending = self.frame_readback.take().unwrap();
        pending.cancel_routes(self.diagnostic_completion_owner.as_deref());
        Ok(Some(pending.finish(
            format!("{:?}", self.config.alpha_mode),
            self.native_target,
        )))
    }
}
