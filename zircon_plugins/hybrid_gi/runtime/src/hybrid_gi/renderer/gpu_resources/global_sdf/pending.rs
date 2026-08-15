use zircon_runtime::graphics::{GraphicsError, RuntimeGpuReadback, RuntimePrepareCollectorContext};

use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfPageBuildRequest;

use super::packing::GlobalSdfGpuBuildStats;

pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuBuildDispatch {
    stats: GlobalSdfGpuBuildStats,
    pending: Option<GlobalSdfGpuPendingBuild>,
}

impl GlobalSdfGpuBuildDispatch {
    pub(super) fn without_pending(stats: GlobalSdfGpuBuildStats) -> Self {
        Self {
            stats,
            pending: None,
        }
    }

    pub(super) fn with_pending(pending: GlobalSdfGpuPendingBuild) -> Self {
        Self {
            stats: pending.stats(),
            pending: Some(pending),
        }
    }

    pub(in crate::hybrid_gi::renderer) fn stats(&self) -> GlobalSdfGpuBuildStats {
        self.stats
    }

    pub(in crate::hybrid_gi::renderer) fn encoded_gpu_work(&self) -> bool {
        self.pending.is_some()
    }

    pub(in crate::hybrid_gi::renderer) fn into_pending(self) -> Option<GlobalSdfGpuPendingBuild> {
        self.pending
    }
}

pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuPendingBuild {
    pub(super) requests: Vec<HybridGiGlobalSdfPageBuildRequest>,
    pub(super) completion_buffer: wgpu::Buffer,
    pub(super) stats: GlobalSdfGpuBuildStats,
}

impl GlobalSdfGpuPendingBuild {
    pub(in crate::hybrid_gi::renderer) fn stats(&self) -> GlobalSdfGpuBuildStats {
        self.stats
    }

    pub(in crate::hybrid_gi::renderer) fn request_count(&self) -> usize {
        self.requests.len()
    }

    pub(in crate::hybrid_gi::renderer) fn copy_completion_to(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        readback: &wgpu::Buffer,
    ) {
        encoder.copy_buffer_to_buffer(
            &self.completion_buffer,
            0,
            readback,
            0,
            self.completion_byte_count(),
        );
    }

    pub(in crate::hybrid_gi::renderer) fn completed_requests_from_words(
        &self,
        completion_words: &[u32],
    ) -> Result<Vec<HybridGiGlobalSdfPageBuildRequest>, GraphicsError> {
        completed_requests_from_words(&self.requests, completion_words)
    }

    fn completion_byte_count(&self) -> u64 {
        (self.requests.len() * std::mem::size_of::<u32>()) as u64
    }

    pub(in crate::hybrid_gi::renderer) fn enqueue(
        self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<GlobalSdfGpuReadbackFuture, GraphicsError> {
        let readback = context.request_gpu_readback(
            "zircon-hybrid-gi-global-sdf-page-completions",
            &self.completion_buffer,
            0..self.completion_byte_count(),
        )?;
        Ok(GlobalSdfGpuReadbackFuture {
            requests: self.requests,
            readback,
        })
    }
}

pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuReadbackFuture {
    requests: Vec<HybridGiGlobalSdfPageBuildRequest>,
    readback: RuntimeGpuReadback,
}

impl GlobalSdfGpuReadbackFuture {
    pub(in crate::hybrid_gi::renderer) fn requests(&self) -> &[HybridGiGlobalSdfPageBuildRequest] {
        &self.requests
    }

    pub(in crate::hybrid_gi::renderer) fn is_ready(&self) -> bool {
        self.readback.is_ready()
    }

    pub(in crate::hybrid_gi::renderer) fn try_collect(
        self,
    ) -> Option<Result<Vec<HybridGiGlobalSdfPageBuildRequest>, GraphicsError>> {
        let bytes = match self.readback.try_take()? {
            Ok(bytes) => bytes,
            Err(error) => return Some(Err(error)),
        };
        if bytes.len() != self.completion_byte_count() as usize {
            return Some(Err(GraphicsError::BufferMap(format!(
                "Global SDF completion readback returned {} bytes for {} pages",
                bytes.len(),
                self.requests.len()
            ))));
        }
        let words = bytes
            .chunks_exact(std::mem::size_of::<u32>())
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect::<Vec<_>>();
        Some(completed_requests_from_words(&self.requests, &words))
    }
}

fn completed_requests_from_words(
    requests: &[HybridGiGlobalSdfPageBuildRequest],
    completion_words: &[u32],
) -> Result<Vec<HybridGiGlobalSdfPageBuildRequest>, GraphicsError> {
    if completion_words.len() != requests.len() {
        return Err(GraphicsError::BufferMap(format!(
            "Global SDF completion readback returned {} words for {} pages",
            completion_words.len(),
            requests.len()
        )));
    }
    Ok(requests
        .iter()
        .copied()
        .zip(completion_words.iter().copied())
        .filter_map(|(request, complete)| (complete == 1).then_some(request))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_gpu_dispatch_retains_terminal_fallback_statistics() {
        let stats = GlobalSdfGpuBuildStats {
            candidate_overflow_page_count: 1,
            deferred_page_count: 3,
            ..GlobalSdfGpuBuildStats::default()
        };

        let dispatch = GlobalSdfGpuBuildDispatch::without_pending(stats);

        assert_eq!(dispatch.stats(), stats);
        assert!(!dispatch.encoded_gpu_work());
        assert!(dispatch.into_pending().is_none());
    }
}
