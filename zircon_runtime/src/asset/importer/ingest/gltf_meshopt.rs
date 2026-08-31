use std::ffi::c_void;

use serde::Deserialize;

use super::gltf_decode::gltf_parse_error;
use crate::asset::AssetImportError;

const MESHOPT_EXTENSION: &str = "EXT_meshopt_compression";

pub(super) fn buffer_is_meshopt_fallback(
    buffer: &gltf::Buffer<'_>,
) -> Result<bool, AssetImportError> {
    let Some(value) = buffer.extension_value(MESHOPT_EXTENSION) else {
        return Ok(false);
    };
    value
        .get("fallback")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            gltf_parse_error(format!(
                "gltf Buffer{} has malformed {MESHOPT_EXTENSION} fallback metadata",
                buffer.index()
            ))
        })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum MeshoptMode {
    Attributes,
    Triangles,
    Indices,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
enum MeshoptFilter {
    #[default]
    None,
    Octahedral,
    Quaternion,
    Exponential,
    Color,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MeshoptView {
    buffer: usize,
    #[serde(default)]
    byte_offset: usize,
    byte_length: usize,
    byte_stride: usize,
    count: usize,
    mode: MeshoptMode,
    #[serde(default)]
    filter: MeshoptFilter,
}

#[derive(Clone, Debug)]
struct MeshoptDecodeJob {
    view_index: usize,
    source_buffer: usize,
    source_offset: usize,
    source_length: usize,
    destination_buffer: usize,
    destination_offset: usize,
    destination_length: usize,
    stride: usize,
    count: usize,
    mode: MeshoptMode,
    filter: MeshoptFilter,
}

pub(super) fn decode_meshopt_views(
    document: &gltf::Document,
    buffers: &mut [gltf::buffer::Data],
) -> Result<(), AssetImportError> {
    let fallback_buffers = document
        .buffers()
        .map(|buffer| buffer_is_meshopt_fallback(&buffer))
        .collect::<Result<Vec<_>, _>>()?;
    let views = document.views();
    let mut jobs = Vec::with_capacity(views.len());
    for view in views {
        let extension = view.extension_value(MESHOPT_EXTENSION);
        if fallback_buffers
            .get(view.buffer().index())
            .copied()
            .unwrap_or(false)
            && extension.is_none()
        {
            return Err(gltf_parse_error(format!(
                "gltf bufferView {} references a meshopt fallback buffer without {MESHOPT_EXTENSION}",
                view.index()
            )));
        }
        let Some(value) = extension else {
            continue;
        };
        let extension: MeshoptView = serde_json::from_value(value.clone()).map_err(|error| {
            gltf_parse_error(format!(
                "parse {MESHOPT_EXTENSION} for bufferView {}: {error}",
                view.index()
            ))
        })?;
        validate_meshopt_view(&view, &extension)?;
        jobs.push(MeshoptDecodeJob {
            view_index: view.index(),
            source_buffer: extension.buffer,
            source_offset: extension.byte_offset,
            source_length: extension.byte_length,
            destination_buffer: view.buffer().index(),
            destination_offset: view.offset(),
            destination_length: view.length(),
            stride: extension.byte_stride,
            count: extension.count,
            mode: extension.mode,
            filter: extension.filter,
        });
    }
    validate_non_overlapping_destinations(&jobs)?;
    for job in &jobs {
        decode_meshopt_view(buffers, job)?;
    }
    Ok(())
}

fn validate_meshopt_view(
    view: &gltf::buffer::View<'_>,
    extension: &MeshoptView,
) -> Result<(), AssetImportError> {
    let decoded_length = extension
        .count
        .checked_mul(extension.byte_stride)
        .ok_or_else(|| gltf_parse_error("meshopt decoded byte length overflow"))?;
    if extension.count == 0
        || extension.byte_stride == 0
        || extension.byte_length == 0
        || decoded_length != view.length()
    {
        return Err(gltf_parse_error(format!(
            "gltf bufferView {} has inconsistent {MESHOPT_EXTENSION} count/stride/length",
            view.index()
        )));
    }
    if view
        .stride()
        .is_some_and(|stride| stride != extension.byte_stride)
    {
        return Err(gltf_parse_error(format!(
            "gltf bufferView {} meshopt stride does not match its parent stride",
            view.index()
        )));
    }
    match extension.mode {
        MeshoptMode::Attributes => {
            validate_meshopt_attribute_filter(view.index(), extension.byte_stride, extension.filter)
        }
        MeshoptMode::Triangles => {
            if extension.count % 3 != 0
                || !matches!(extension.byte_stride, 2 | 4)
                || extension.filter != MeshoptFilter::None
            {
                return Err(gltf_parse_error(format!(
                    "gltf bufferView {} has invalid meshopt TRIANGLES count, stride, or filter",
                    view.index()
                )));
            }
            Ok(())
        }
        MeshoptMode::Indices => {
            if !matches!(extension.byte_stride, 2 | 4) || extension.filter != MeshoptFilter::None {
                return Err(gltf_parse_error(format!(
                    "gltf bufferView {} has invalid meshopt INDICES stride or filter",
                    view.index()
                )));
            }
            Ok(())
        }
    }
}

fn validate_meshopt_attribute_filter(
    view_index: usize,
    stride: usize,
    filter: MeshoptFilter,
) -> Result<(), AssetImportError> {
    let valid = match filter {
        MeshoptFilter::None => stride <= 256 && stride % 4 == 0,
        MeshoptFilter::Octahedral | MeshoptFilter::Color => matches!(stride, 4 | 8),
        MeshoptFilter::Quaternion => stride == 8,
        MeshoptFilter::Exponential => stride <= 256 && stride % 4 == 0,
    };
    if !valid {
        return Err(gltf_parse_error(format!(
            "gltf bufferView {view_index} has invalid meshopt ATTRIBUTES stride for {filter:?} filter"
        )));
    }
    Ok(())
}

fn validate_non_overlapping_destinations(
    jobs: &[MeshoptDecodeJob],
) -> Result<(), AssetImportError> {
    let mut ranges = Vec::with_capacity(jobs.len());
    for job in jobs {
        let end = job
            .destination_offset
            .checked_add(job.destination_length)
            .ok_or_else(|| gltf_parse_error("meshopt destination range overflow"))?;
        ranges.push((
            job.destination_buffer,
            job.destination_offset,
            end,
            job.view_index,
        ));
    }
    ranges.sort_unstable_by_key(|range| (range.0, range.1));
    for pair in ranges.windows(2) {
        if pair[0].0 == pair[1].0 && pair[0].2 > pair[1].1 {
            return Err(gltf_parse_error(format!(
                "gltf meshopt bufferViews {} and {} overlap in their decoded buffer",
                pair[0].3, pair[1].3
            )));
        }
    }
    Ok(())
}

fn decode_meshopt_view(
    buffers: &mut [gltf::buffer::Data],
    job: &MeshoptDecodeJob,
) -> Result<(), AssetImportError> {
    if job.source_buffer == job.destination_buffer {
        return Err(gltf_parse_error(format!(
            "gltf meshopt bufferView {} uses the same source and destination buffer",
            job.view_index
        )));
    }
    let source_end = job
        .source_offset
        .checked_add(job.source_length)
        .ok_or_else(|| gltf_parse_error("meshopt source range overflow"))?;
    let destination_end = job
        .destination_offset
        .checked_add(job.destination_length)
        .ok_or_else(|| gltf_parse_error("meshopt destination range overflow"))?;
    if job.source_buffer >= buffers.len() || job.destination_buffer >= buffers.len() {
        return Err(gltf_parse_error(format!(
            "gltf meshopt bufferView {} references a missing buffer",
            job.view_index
        )));
    }

    let (source, destination) = split_buffer_slices(
        buffers,
        job.source_buffer,
        job.source_offset,
        source_end,
        job.destination_buffer,
        job.destination_offset,
        destination_end,
        job.view_index,
    )?;
    let status = unsafe {
        // Bounds, non-aliasing, output size, stride and mode are validated above.
        match job.mode {
            MeshoptMode::Attributes => meshopt::ffi::meshopt_decodeVertexBuffer(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
                source.as_ptr(),
                source.len(),
            ),
            MeshoptMode::Triangles => meshopt::ffi::meshopt_decodeIndexBuffer(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
                source.as_ptr(),
                source.len(),
            ),
            MeshoptMode::Indices => meshopt::ffi::meshopt_decodeIndexSequence(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
                source.as_ptr(),
                source.len(),
            ),
        }
    };
    if status != 0 {
        return Err(gltf_parse_error(format!(
            "meshopt decode failed for gltf bufferView {} with status {status}",
            job.view_index
        )));
    }
    apply_meshopt_filter(destination, job);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn split_buffer_slices<'a>(
    buffers: &'a mut [gltf::buffer::Data],
    source_buffer: usize,
    source_offset: usize,
    source_end: usize,
    destination_buffer: usize,
    destination_offset: usize,
    destination_end: usize,
    view_index: usize,
) -> Result<(&'a [u8], &'a mut [u8]), AssetImportError> {
    let (source, destination) = if source_buffer < destination_buffer {
        let (left, right) = buffers.split_at_mut(destination_buffer);
        (&left[source_buffer].0, &mut right[0].0)
    } else {
        let (left, right) = buffers.split_at_mut(source_buffer);
        (&right[0].0, &mut left[destination_buffer].0)
    };
    let source = source.get(source_offset..source_end).ok_or_else(|| {
        gltf_parse_error(format!(
            "gltf meshopt bufferView {view_index} source range is out of bounds"
        ))
    })?;
    let destination = destination
        .get_mut(destination_offset..destination_end)
        .ok_or_else(|| {
            gltf_parse_error(format!(
                "gltf meshopt bufferView {view_index} destination range is out of bounds"
            ))
        })?;
    Ok((source, destination))
}

fn apply_meshopt_filter(destination: &mut [u8], job: &MeshoptDecodeJob) {
    unsafe {
        // The decoder initialized exactly count * stride bytes in destination.
        match job.filter {
            MeshoptFilter::None => {}
            MeshoptFilter::Octahedral => meshopt::ffi::meshopt_decodeFilterOct(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
            ),
            MeshoptFilter::Quaternion => meshopt::ffi::meshopt_decodeFilterQuat(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
            ),
            MeshoptFilter::Exponential => meshopt::ffi::meshopt_decodeFilterExp(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
            ),
            MeshoptFilter::Color => meshopt::ffi::meshopt_decodeFilterColor(
                destination.as_mut_ptr().cast::<c_void>(),
                job.count,
                job.stride,
            ),
        }
    }
}

#[cfg(test)]
mod plugins07_meshopt_collection_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const JOB_COUNT: usize = 65_536;
    const DESTINATION_BUFFERS: usize = 256;

    #[test]
    fn meshopt_collection_contract_rejects_same_buffer_overlap() {
        let jobs = vec![decode_job(3, 7, 0, 16), decode_job(9, 7, 8, 16)];

        let error = validate_non_overlapping_destinations(&jobs).unwrap_err();

        assert!(error.to_string().contains("bufferViews 3 and 9 overlap"));
    }

    #[test]
    fn meshopt_collection_contract_allows_cross_buffer_ranges() {
        let jobs = vec![decode_job(3, 7, 0, 16), decode_job(9, 8, 0, 16)];

        validate_non_overlapping_destinations(&jobs).unwrap();
    }

    #[test]
    #[ignore = "release performance gate"]
    fn meshopt_collection_performance_release_decode_jobs() {
        run_release_gate(
            "plugins07_meshopt_decode_job_collection",
            25,
            "workload_items=65536 legacy_initial_capacity=0 optimized_initial_capacity=65536",
            || measure_decode_job_collection(false),
            || measure_decode_job_collection(true),
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn meshopt_collection_performance_release_overlap_ranges() {
        let jobs = overlap_jobs();
        run_release_gate(
            "plugins07_meshopt_overlap_range_collection",
            40,
            "workload_items=65536 destination_buffers=256 legacy_range_containers=257 optimized_range_containers=1",
            || measure_grouped_range_collection(&jobs),
            || measure_flat_range_collection(&jobs),
        );
    }

    fn run_release_gate(
        marker: &str,
        threshold_percent: u128,
        structural_fields: &str,
        mut legacy: impl FnMut() -> u128,
        mut optimized: impl FnMut() -> u128,
    ) {
        for _ in 0..4 {
            black_box(legacy());
            black_box(optimized());
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (legacy(), optimized())
            } else {
                let optimized_ns = optimized();
                (legacy(), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT {marker} sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 {structural_fields}",
            csv(&legacy_samples),
            csv(&optimized_samples),
            threshold_percent = threshold_percent,
        );
        assert!(
            improvement_percent >= threshold_percent,
            "{marker} must improve P95 by at least {threshold_percent}%"
        );
    }

    fn measure_decode_job_collection(preallocate: bool) -> u128 {
        let started = Instant::now();
        let mut jobs = if black_box(preallocate) {
            Vec::with_capacity(JOB_COUNT)
        } else {
            Vec::new()
        };
        for index in 0..JOB_COUNT {
            jobs.push(decode_job(
                index,
                index % DESTINATION_BUFFERS,
                index * 16,
                8,
            ));
        }
        black_box(jobs);
        started.elapsed().as_nanos().max(1)
    }

    fn overlap_jobs() -> Vec<MeshoptDecodeJob> {
        (0..JOB_COUNT)
            .map(|index| {
                decode_job(
                    index,
                    index % DESTINATION_BUFFERS,
                    (index / DESTINATION_BUFFERS) * 16,
                    8,
                )
            })
            .collect()
    }

    fn measure_grouped_range_collection(jobs: &[MeshoptDecodeJob]) -> u128 {
        let started = Instant::now();
        let mut ranges_by_buffer: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
        for job in black_box(jobs) {
            ranges_by_buffer
                .entry(job.destination_buffer)
                .or_default()
                .push((
                    job.destination_offset,
                    job.destination_offset + job.destination_length,
                    job.view_index,
                ));
        }
        black_box(ranges_by_buffer);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_flat_range_collection(jobs: &[MeshoptDecodeJob]) -> u128 {
        let started = Instant::now();
        let mut ranges = Vec::with_capacity(jobs.len());
        for job in black_box(jobs) {
            ranges.push((
                job.destination_buffer,
                job.destination_offset,
                job.destination_offset + job.destination_length,
                job.view_index,
            ));
        }
        black_box(ranges);
        started.elapsed().as_nanos().max(1)
    }

    fn decode_job(
        view_index: usize,
        destination_buffer: usize,
        destination_offset: usize,
        destination_length: usize,
    ) -> MeshoptDecodeJob {
        MeshoptDecodeJob {
            view_index,
            source_buffer: destination_buffer + DESTINATION_BUFFERS,
            source_offset: 0,
            source_length: destination_length,
            destination_buffer,
            destination_offset,
            destination_length,
            stride: 4,
            count: destination_length / 4,
            mode: MeshoptMode::Attributes,
            filter: MeshoptFilter::None,
        }
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
