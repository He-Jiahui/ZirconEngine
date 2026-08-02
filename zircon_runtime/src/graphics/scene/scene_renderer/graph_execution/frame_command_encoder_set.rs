const SERIAL_ENCODER_LABEL: &str = "zircon-compiled-scene-serial-segment";

/// Owns the ordered command-buffer stream for one compiled scene submission.
///
/// Serial work is encoded lazily. A parallel graph stage flushes the current serial prefix,
/// appends its topology-ordered buffers, and leaves the next serial suffix unallocated until used.
pub(crate) struct FrameCommandEncoderSet {
    active_serial: Option<wgpu::CommandEncoder>,
    completed: Vec<wgpu::CommandBuffer>,
}

impl FrameCommandEncoderSet {
    pub(crate) fn new() -> Self {
        Self {
            active_serial: None,
            completed: Vec::new(),
        }
    }

    pub(crate) fn from_serial_encoder(encoder: wgpu::CommandEncoder) -> Self {
        Self {
            active_serial: Some(encoder),
            completed: Vec::new(),
        }
    }

    pub(crate) fn serial_encoder(&mut self, device: &wgpu::Device) -> &mut wgpu::CommandEncoder {
        self.active_serial.get_or_insert_with(|| {
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(SERIAL_ENCODER_LABEL),
            })
        })
    }

    pub(crate) fn flush_serial_prefix(&mut self) {
        if let Some(encoder) = self.active_serial.take() {
            self.completed.push(encoder.finish());
        }
    }

    pub(crate) fn append_parallel_buffers(
        &mut self,
        command_buffers: impl IntoIterator<Item = wgpu::CommandBuffer>,
    ) {
        debug_assert!(self.active_serial.is_none());
        self.completed.extend(command_buffers);
    }

    pub(crate) fn finish(mut self) -> Vec<wgpu::CommandBuffer> {
        self.flush_serial_prefix();
        self.completed
    }
}

impl Default for FrameCommandEncoderSet {
    fn default() -> Self {
        Self::new()
    }
}
