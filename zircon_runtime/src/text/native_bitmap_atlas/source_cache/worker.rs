use super::*;

impl NativeBitmapAtlasSourceCache {
    pub(crate) fn request_worker_image(
        &mut self,
        font_database: &FontDatabase,
        worker_pool: Option<&TextRasterWorkerPool>,
        face_epoch: u64,
        cache_key: GlyphRasterKey,
    ) -> NativeBitmapAtlasWorkerRequestStatus {
        if self.entries.contains_key(&cache_key) {
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        }
        if self.pending_worker_work_ids.contains_key(&cache_key) {
            self.frame_report.worker_request_pending_count = self
                .frame_report
                .worker_request_pending_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Pending;
        }

        let Some(worker_pool) = worker_pool else {
            self.frame_report.worker_request_unavailable_count = self
                .frame_report
                .worker_request_unavailable_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };
        if self.frame_report.worker_request_submitted_count
            >= NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME
        {
            self.frame_report.worker_request_deferred_count = self
                .frame_report
                .worker_request_deferred_count
                .saturating_add(1);
            return NativeBitmapAtlasWorkerRequestStatus::DeferredByFrameBudget;
        }

        let Some(instance) = font_database.font_instance(cache_key.face) else {
            self.record_worker_font_missing();
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };
        let face = instance.face;
        let Ok(face_index) = font_database.face_index(face) else {
            self.record_worker_font_missing();
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };
        let Some(request) =
            SwashRasterRequest::native_bitmap_atlas_glyph(face_index as usize, cache_key)
        else {
            self.record_worker_font_missing();
            return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
        };
        let font_bytes = match self.raster_font_bytes_by_face.get(&face) {
            Some(bytes) => Arc::clone(bytes),
            None => {
                let Ok(bytes) = font_database.face_bytes(face) else {
                    self.record_worker_font_missing();
                    return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
                };
                self.frame_report.worker_request_font_copied_byte_count = self
                    .frame_report
                    .worker_request_font_copied_byte_count
                    .saturating_add(bytes.len());
                self.raster_font_resident_byte_count = self
                    .raster_font_resident_byte_count
                    .saturating_add(bytes.len());
                self.raster_font_bytes_by_face
                    .insert(face, Arc::clone(&bytes));
                bytes
            }
        };
        let variations = Arc::clone(
            self.raster_variations_by_instance
                .entry(cache_key.face)
                .or_insert_with(|| Arc::new(instance.variations.clone())),
        );
        let font_identity = self.raster_font_identity(face, face_epoch);
        let request = request
            .with_variations(variations)
            .with_font_identity(font_identity);
        let work_id = self.next_worker_id();
        let work = TextRasterWorkItem::new(work_id, face_epoch, font_bytes, request);

        match worker_pool.try_request(work) {
            Ok(()) => {}
            Err(
                TextRasterWorkerRequestError::QueueFull(_)
                | TextRasterWorkerRequestError::QueueBytesFull(_),
            ) => {
                self.frame_report.worker_request_backpressured_count = self
                    .frame_report
                    .worker_request_backpressured_count
                    .saturating_add(1);
                return NativeBitmapAtlasWorkerRequestStatus::DeferredByWorkerBackpressure;
            }
            Err(
                TextRasterWorkerRequestError::ChannelClosed(_)
                | TextRasterWorkerRequestError::DuplicateInFlight(_),
            ) => {
                self.frame_report.worker_request_failed_count = self
                    .frame_report
                    .worker_request_failed_count
                    .saturating_add(1);
                return NativeBitmapAtlasWorkerRequestStatus::Unavailable;
            }
        }

        self.register_worker_request(work_id, cache_key);
        self.frame_report.worker_request_submitted_count = self
            .frame_report
            .worker_request_submitted_count
            .saturating_add(1);
        NativeBitmapAtlasWorkerRequestStatus::Submitted(work_id)
    }

    pub(crate) fn apply_worker_completion_drain(
        &mut self,
        drain: TextRasterCompletionDrain,
    ) -> NativeBitmapAtlasSourceCacheFrameReport {
        let TextRasterCompletionDrain {
            accepted,
            face_invalidated_ids,
            face_invalidated_count,
            drained_bytes,
            byte_budget_deferred_count,
            oversized_accepted_count,
        } = drain;
        self.frame_report.worker_completion_drained_byte_count = self
            .frame_report
            .worker_completion_drained_byte_count
            .saturating_add(drained_bytes);
        self.frame_report
            .worker_completion_byte_budget_deferred_count = self
            .frame_report
            .worker_completion_byte_budget_deferred_count
            .saturating_add(byte_budget_deferred_count);
        self.frame_report.worker_completion_oversized_accepted_count = self
            .frame_report
            .worker_completion_oversized_accepted_count
            .saturating_add(oversized_accepted_count);
        self.frame_report.worker_completion_face_invalidated_count = self
            .frame_report
            .worker_completion_face_invalidated_count
            .saturating_add(face_invalidated_count);

        for work_id in face_invalidated_ids {
            self.remove_pending_worker_request(work_id);
        }
        for result in accepted {
            self.apply_worker_result(result);
        }
        self.frame_report()
    }

    fn apply_worker_result(&mut self, result: TextRasterWorkResult) {
        let Some(cache_key) = self.remove_pending_worker_request(result.id) else {
            self.frame_report.worker_completion_unknown_count = self
                .frame_report
                .worker_completion_unknown_count
                .saturating_add(1);
            return;
        };
        let Ok(bitmap) = result.result else {
            self.frame_report.worker_completion_failed_count = self
                .frame_report
                .worker_completion_failed_count
                .saturating_add(1);
            return;
        };
        let Some(image) = cached_glyph_image_from_worker_bitmap(bitmap) else {
            self.frame_report.worker_completion_invalid_bitmap_count = self
                .frame_report
                .worker_completion_invalid_bitmap_count
                .saturating_add(1);
            return;
        };
        let image_byte_count = image.bytes.len();
        let raster_key = native_bitmap_atlas_raster_key_for_content(cache_key, image.content);
        if self.insert(cache_key, image) {
            self.frame_report.worker_completion_insert_count = self
                .frame_report
                .worker_completion_insert_count
                .saturating_add(1);
            self.frame_report.worker_completion_applied_byte_count = self
                .frame_report
                .worker_completion_applied_byte_count
                .saturating_add(image_byte_count);
            if let Some(raster_key) = raster_key {
                let _ = self.bind_persistent_raster_key(cache_key, raster_key);
            }
        }
    }

    fn record_worker_font_missing(&mut self) {
        self.frame_report.worker_request_font_missing_count = self
            .frame_report
            .worker_request_font_missing_count
            .saturating_add(1);
    }

    pub(super) fn cancel_pending_worker_requests(
        &mut self,
        worker_pool: Option<&TextRasterWorkerPool>,
    ) {
        let cancelled_count = worker_pool
            .map(|worker_pool| {
                self.pending_worker_cache_keys
                    .keys()
                    .filter(|work_id| worker_pool.cancel(**work_id))
                    .count()
            })
            .unwrap_or(0);
        self.pending_worker_cache_keys.clear();
        self.pending_worker_work_ids.clear();
        self.pending_worker_cancelled_count = self
            .pending_worker_cancelled_count
            .saturating_add(cancelled_count);
    }

    fn remove_pending_worker_request(
        &mut self,
        work_id: TextRasterWorkId,
    ) -> Option<GlyphRasterKey> {
        let cache_key = self.pending_worker_cache_keys.remove(&work_id)?;
        self.pending_worker_work_ids.remove(&cache_key);
        Some(cache_key)
    }

    fn next_worker_id(&mut self) -> TextRasterWorkId {
        let work_id = self.next_worker_id;
        self.next_worker_id = self
            .next_worker_id
            .saturating_add(1)
            .max(FIRST_NATIVE_BITMAP_ATLAS_WORK_ID);
        TextRasterWorkId::new(work_id)
    }

    fn raster_font_identity(&mut self, face: FontFaceId, face_epoch: u64) -> [u64; 2] {
        let next_raster_font_id = &mut self.next_raster_font_id;
        let raster_font_id = *self
            .raster_font_identity_by_face
            .entry(face)
            .or_insert_with(|| {
                let raster_font_id = *next_raster_font_id;
                *next_raster_font_id = next_raster_font_id
                    .saturating_add(1)
                    .max(FIRST_NATIVE_BITMAP_ATLAS_RASTER_FONT_ID);
                raster_font_id
            });
        [face_epoch, raster_font_id]
    }
}
