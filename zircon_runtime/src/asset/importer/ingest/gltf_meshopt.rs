use std::collections::BTreeMap;
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
    let mut jobs = Vec::new();
    for view in document.views() {
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
    let mut ranges_by_buffer: BTreeMap<usize, Vec<(usize, usize, usize)>> = BTreeMap::new();
    for job in jobs {
        let end = job
            .destination_offset
            .checked_add(job.destination_length)
            .ok_or_else(|| gltf_parse_error("meshopt destination range overflow"))?;
        ranges_by_buffer
            .entry(job.destination_buffer)
            .or_default()
            .push((job.destination_offset, end, job.view_index));
    }
    for ranges in ranges_by_buffer.values_mut() {
        ranges.sort_unstable_by_key(|range| range.0);
        for pair in ranges.windows(2) {
            if pair[0].1 > pair[1].0 {
                return Err(gltf_parse_error(format!(
                    "gltf meshopt bufferViews {} and {} overlap in their decoded buffer",
                    pair[0].2, pair[1].2
                )));
            }
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
