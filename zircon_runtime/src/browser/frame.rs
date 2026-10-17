use super::*;
impl BrowserEngine {
    pub async fn frame(&mut self, timestamp_ms: f64) -> Result<BrowserFrameReceipt, String> {
        if self.closing || self.input.is_closed() {
            return Err("browser engine is closing".into());
        }
        if !timestamp_ms.is_finite() || timestamp_ms < 0.0 {
            return Err("invalid browser frame timestamp".into());
        }
        let generation = self.input.lifecycle_generation();
        if generation != self.lifecycle_generation {
            self.last_timestamp = None;
            self.lifecycle_generation = generation;
        }
        let mut receipt = BrowserFrameReceipt {
            frame: self.frame,
            presented: false,
            animation_poses: 0,
            ui_commands: 0,
            pending_assets: self.source.pending_requests(),
            omitted_input_events: self.input.omitted_events(),
            submitted_frames: 0,
            visible_instances: 0,
            texture_resident_bytes: 0,
            transient_texture_peak_bytes: 0,
            transient_buffer_peak_bytes: 0,
            staging_bytes: 0,
        };
        let size = UVec2::new(self.canvas.width(), self.canvas.height());
        if !self.input.is_visible() || size.x == 0 || size.y == 0 {
            self.last_timestamp = None;
            return Ok(receipt);
        }
        if size.x > 4096 || size.y > 4096 {
            return Err("browser viewport exceeds the 4096 pixel dimension limit".into());
        }
        if size != self.size {
            self.bind_surface(size)?;
            self.size = size;
        }
        let delta = self.last_timestamp.map_or(0.0, |previous| {
            (timestamp_ms - previous).max(0.0).min(250.0)
        });
        self.last_timestamp = Some(timestamp_ms);
        let core = self
            .core
            .as_ref()
            .ok_or_else(|| "browser engine is closed".to_string())?;
        let handle = core.handle();
        let input = resolve_manager_service(&handle, input_manager_handle(&handle).map_err(error)?)
            .map_err(error)?;
        input.begin_frame();
        self.input.drain(&mut self.frame_events);
        for event in self.frame_events.drain(..) {
            if let Some(ui) = &mut self.ui {
                ui.input(&event, timestamp_ms)?;
            }
            input.submit_event(event);
        }
        drop(input);
        let snapshot = core.advance_time_by(Duration::from_secs_f64(delta / 1000.0), 4);
        let level = self
            .level
            .as_ref()
            .ok_or_else(|| "browser level is closed".to_string())?;
        level.tick(&handle, snapshot).map_err(error)?;
        let context = RenderExtractContext::new(
            RenderWorldSnapshotHandle::from(level.handle())
                .with_generation(level.world_generation()),
            SceneViewportExtractRequest {
                viewport_size: Some(size),
                ..Default::default()
            },
        );
        // Uses the prepared LevelSystem extraction; never World::clone().
        let mut extract = level.build_render_frame_extract(&context);
        self.frame = self
            .frame
            .checked_add(1)
            .ok_or_else(|| "browser frame sequence exhausted".to_string())?;
        extract.set_timing(RenderFrameTiming::new(self.frame, delta as f32 / 1000.0));
        receipt.animation_poses = extract.animation_poses.len();
        let ui = self
            .ui
            .as_mut()
            .map(|ui| ui.submission(size, timestamp_ms))
            .transpose()?
            .flatten();
        receipt.ui_commands = ui
            .as_ref()
            .map_or(0, |submission| submission.command_count());
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| "browser renderer is closed".to_string())?;
        let submitted = renderer.present_frame_extract_with_ui(
            self.viewport.expect("open viewport"),
            extract,
            ui,
        );
        let validated = renderer
            .finish_browser_frame_validation(submitted.is_ok())
            .await;
        submitted.map_err(error)?;
        validated.map_err(error)?;
        let stats = renderer.query_stats().map_err(error)?;
        receipt.frame = self.frame;
        receipt.presented = true;
        receipt.submitted_frames = stats.submitted_frames;
        receipt.visible_instances = stats.last_visibility_visible_count;
        receipt.texture_resident_bytes = stats.last_frame_profile.persistent_texture_resident_bytes;
        receipt.transient_texture_peak_bytes =
            stats.last_frame_profile.transient_texture_peak_bytes;
        receipt.transient_buffer_peak_bytes = stats.last_frame_profile.transient_buffer_peak_bytes;
        receipt.staging_bytes = stats.last_frame_profile.staging_total_bytes;
        receipt.pending_assets = self.source.pending_requests();
        Ok(receipt)
    }
}
