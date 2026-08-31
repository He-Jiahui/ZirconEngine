use std::collections::HashSet;

use super::{ChromeCommandKind, ChromeCommandLayer, ChromeCommandStream};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStreamStats {
    pub(in crate::ui::retained_host::host_contract) command_count: usize,
    pub(in crate::ui::retained_host::host_contract) static_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) dynamic_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) text_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) image_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) clip_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) image_upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) draw_call_count: u64,
}

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn stats(&self) -> ChromeCommandStreamStats {
        let mut uploaded_image_versions: Option<HashSet<(&str, u64)>> = None;
        let mut stats = ChromeCommandStreamStats {
            command_count: self.commands().len(),
            ..ChromeCommandStreamStats::default()
        };
        for (_, _, resource) in self.image_resources().iter() {
            stats.image_upload_bytes = stats
                .image_upload_bytes
                .saturating_add(resource.upload_bytes);
        }
        for command in self.commands() {
            match command.layer {
                ChromeCommandLayer::Static => stats.static_command_count += 1,
                ChromeCommandLayer::Dynamic => stats.dynamic_command_count += 1,
                ChromeCommandLayer::Text => stats.text_command_count += 1,
                ChromeCommandLayer::Viewport => stats.dynamic_command_count += 1,
            }
            match &command.kind {
                ChromeCommandKind::Quad { .. }
                | ChromeCommandKind::Border { .. }
                | ChromeCommandKind::Text { .. } => stats.draw_call_count += 1,
                ChromeCommandKind::Image { payload } => {
                    stats.image_command_count += 1;
                    if payload.rgba.is_some()
                        && uploaded_image_versions
                            .get_or_insert_with(|| {
                                self.image_resources()
                                    .iter()
                                    .map(|(resource_key, generation, _)| (resource_key, generation))
                                    .collect()
                            })
                            .insert((payload.resource_key.as_str(), payload.resource_generation))
                    {
                        stats.image_upload_bytes = stats
                            .image_upload_bytes
                            .saturating_add(payload.upload_bytes);
                    }
                    stats.draw_call_count += 1;
                }
                ChromeCommandKind::Clip => stats.clip_command_count += 1,
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::super::{ChromeCommand, ChromeImagePayload};
    use super::*;
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn optimization_batch_dx_lazy_image_upload_dedup_preserves_upload_accounting() {
        let mut compacted = image_stream(2, true);
        compacted.compact_image_resources();
        assert_eq!(compacted.stats().image_upload_bytes, 32);

        let inline = repeated_inline_image_stream();
        assert_eq!(inline.stats().image_upload_bytes, 16);
    }

    #[test]
    fn optimization_batch_dx_image_upload_dedup_is_lazily_allocated() {
        let production = include_str!("stats.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("chrome command stats production source");

        assert!(production.contains("Option<HashSet<(&str, u64)>>"));
        assert!(production.contains("get_or_insert_with"));
        assert!(!production.contains("let mut uploaded_image_versions = self"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dx_lazy_image_upload_dedup_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const STATS_PER_SAMPLE: usize = 128;
        const IMAGE_RESOURCE_COUNT: usize = 1_024;

        let mut stream = image_stream(IMAGE_RESOURCE_COUNT, true);
        stream.compact_image_resources();
        assert!(stream.commands().iter().all(|command| matches!(
            &command.kind,
            ChromeCommandKind::Image { payload } if payload.rgba.is_none()
        )));
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_stats(&stream, STATS_PER_SAMPLE, false));
                optimized_samples.push(measure_stats(&stream, STATS_PER_SAMPLE, true));
            } else {
                optimized_samples.push(measure_stats(&stream, STATS_PER_SAMPLE, true));
                legacy_samples.push(measure_stats(&stream, STATS_PER_SAMPLE, false));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR360_LAZY_IMAGE_UPLOAD_DEDUP_BENCH_V1 stats_per_sample={STATS_PER_SAMPLE} image_resource_count={IMAGE_RESOURCE_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "lazy image upload dedup p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn image_stream(image_count: usize, unique: bool) -> ChromeCommandStream {
        let mut stream = ChromeCommandStream::full_rebuild((1_024, 1_024));
        for index in 0..image_count {
            let identity = if unique { index } else { 0 };
            stream.push_command_for_test(image_command(identity, true));
        }
        stream
    }

    fn repeated_inline_image_stream() -> ChromeCommandStream {
        image_stream(2, false)
    }

    fn image_command(identity: usize, with_pixels: bool) -> ChromeCommand {
        ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: identity as i32,
            frame: FrameRect::default(),
            clip: None,
            source: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: format!(
                        "image://{}/resource_{identity:04}",
                        "long_resource_segment/".repeat(8)
                    ),
                    resource_generation: identity as u64,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: with_pixels.then(|| vec![identity as u8; 16].into()),
                    atlas_uv: None,
                },
            },
        }
    }

    fn measure_stats(stream: &ChromeCommandStream, stats_count: usize, optimized: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..stats_count {
            let stats = if optimized {
                stream.stats()
            } else {
                legacy_stats(stream)
            };
            checksum = checksum.wrapping_add(stats.image_upload_bytes);
            black_box(stats);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn legacy_stats(stream: &ChromeCommandStream) -> ChromeCommandStreamStats {
        let mut uploaded_image_versions = stream
            .image_resources()
            .iter()
            .map(|(resource_key, generation, _)| (resource_key, generation))
            .collect::<HashSet<_>>();
        let mut stats = ChromeCommandStreamStats {
            command_count: stream.commands().len(),
            ..ChromeCommandStreamStats::default()
        };
        for (_, _, resource) in stream.image_resources().iter() {
            stats.image_upload_bytes = stats
                .image_upload_bytes
                .saturating_add(resource.upload_bytes);
        }
        for command in stream.commands() {
            match command.layer {
                ChromeCommandLayer::Static => stats.static_command_count += 1,
                ChromeCommandLayer::Dynamic => stats.dynamic_command_count += 1,
                ChromeCommandLayer::Text => stats.text_command_count += 1,
                ChromeCommandLayer::Viewport => stats.dynamic_command_count += 1,
            }
            match &command.kind {
                ChromeCommandKind::Quad { .. }
                | ChromeCommandKind::Border { .. }
                | ChromeCommandKind::Text { .. } => stats.draw_call_count += 1,
                ChromeCommandKind::Image { payload } => {
                    stats.image_command_count += 1;
                    if payload.rgba.is_some()
                        && uploaded_image_versions
                            .insert((payload.resource_key.as_str(), payload.resource_generation))
                    {
                        stats.image_upload_bytes = stats
                            .image_upload_bytes
                            .saturating_add(payload.upload_bytes);
                    }
                    stats.draw_call_count += 1;
                }
                ChromeCommandKind::Clip => stats.clip_command_count += 1,
            }
        }
        stats
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
