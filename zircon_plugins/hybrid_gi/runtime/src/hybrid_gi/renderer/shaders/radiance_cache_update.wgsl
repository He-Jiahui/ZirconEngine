struct RadianceCacheDispatchParams {
    update_count: u32,
    consume_count: u32,
    resident_probe_count: u32,
    stage: u32,
};

struct RadianceCacheUpdateInput {
    slot: u32,
    generation_low: u32,
    generation_high: u32,
    radiance_confidence: u32,
    reuse_committed_radiance: u32,
};

struct RadianceCacheConsumeInput {
    probe_id: u32,
    generation_low: u32,
    generation_high: u32,
    resident_probe_index: u32,
    slots: array<u32, 8>,
    weights_q16: array<u32, 8>,
};

struct RadianceCacheStorageEntry {
    radiance_confidence: u32,
    generation_low: u32,
    generation_high: u32,
    atlas_base: u32,
};

@group(0) @binding(0)
var<uniform> params: RadianceCacheDispatchParams;

@group(0) @binding(1)
var<storage, read> update_inputs: array<RadianceCacheUpdateInput>;

@group(0) @binding(2)
var<storage, read> consume_inputs: array<RadianceCacheConsumeInput>;

@group(0) @binding(3)
var<storage, read_write> cache_entries: array<RadianceCacheStorageEntry>;

@group(0) @binding(4)
var<storage, read_write> marked_slots: array<atomic<u32>>;

@group(0) @binding(5)
var<storage, read_write> trace_atlas: array<u32>;

@group(0) @binding(6)
var<storage, read_write> filtered_atlas: array<u32>;

@group(0) @binding(7)
var<storage, read_write> final_atlas: array<u32>;

@group(0) @binding(8)
var<storage, read_write> resident_probe_words: array<u32>;

const RADIANCE_CACHE_SLOT_CAPACITY: u32 = 32u;
const RADIANCE_CACHE_STAGE_MARK: u32 = 0u;
const RADIANCE_CACHE_STAGE_ALLOCATE: u32 = 1u;
const RADIANCE_CACHE_STAGE_TRACE: u32 = 2u;
const RADIANCE_CACHE_STAGE_FILTER: u32 = 3u;
const RADIANCE_CACHE_STAGE_BORDER_MIP: u32 = 4u;
const RADIANCE_CACHE_STAGE_CONSUME: u32 = 5u;
const RADIANCE_CACHE_DISPATCH_COUNTER_OFFSET: u32 = RADIANCE_CACHE_SLOT_CAPACITY;
const RESIDENT_PROBE_WORD_COUNT: u32 = 23u;
const RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET: u32 = 8u;
const RADIANCE_CACHE_PROBE_TILE_EXTENT: u32 = 4u;
const RADIANCE_CACHE_PROBE_TILE_TEXEL_COUNT: u32 = 16u;
const RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT: u32 = 4u;
const RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT: u32 = 1u;
const RADIANCE_CACHE_PROBE_MIP1_OFFSET: u32 = RADIANCE_CACHE_PROBE_TILE_TEXEL_COUNT;
const RADIANCE_CACHE_PROBE_MIP2_OFFSET: u32 =
    RADIANCE_CACHE_PROBE_MIP1_OFFSET + RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT;
const RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT: u32 =
    RADIANCE_CACHE_PROBE_MIP2_OFFSET + RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT;

fn probe_atlas_base(slot: u32) -> u32 {
    return slot * RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT;
}

fn unpack_rgba8(packed: u32) -> vec4<u32> {
    return vec4<u32>(
        packed & 0xffu,
        (packed >> 8u) & 0xffu,
        (packed >> 16u) & 0xffu,
        (packed >> 24u) & 0xffu,
    );
}

fn pack_rgba8(rgba: vec4<u32>) -> u32 {
    return rgba.x | (rgba.y << 8u) | (rgba.z << 16u) | (rgba.w << 24u);
}

fn average_cross_rgba8(
    center: u32,
    left: u32,
    right: u32,
    up: u32,
    down: u32,
) -> u32 {
    let accumulated =
        unpack_rgba8(center)
        + unpack_rgba8(left)
        + unpack_rgba8(right)
        + unpack_rgba8(up)
        + unpack_rgba8(down);
    return pack_rgba8((accumulated + vec4<u32>(2u)) / 5u);
}

fn average_four_rgba8(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let accumulated = unpack_rgba8(a) + unpack_rgba8(b) + unpack_rgba8(c) + unpack_rgba8(d);
    return pack_rgba8((accumulated + vec4<u32>(2u)) / 4u);
}

@compute @workgroup_size(64, 1, 1)
fn cs_update(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let update_index = global_id.x;
    if (update_index >= params.update_count) {
        return;
    }
    if (update_index == 0u) {
        atomicStore(
            &marked_slots[RADIANCE_CACHE_DISPATCH_COUNTER_OFFSET + params.stage],
            params.update_count,
        );
    }

    let update_input = update_inputs[update_index];
    if (update_input.slot >= RADIANCE_CACHE_SLOT_CAPACITY) {
        return;
    }

    if (params.stage == RADIANCE_CACHE_STAGE_MARK) {
        atomicStore(&marked_slots[update_input.slot], update_index + 1u);
        return;
    }
    if (atomicLoad(&marked_slots[update_input.slot]) != update_index + 1u) {
        return;
    }
    let atlas_base = probe_atlas_base(update_input.slot);
    if (params.stage == RADIANCE_CACHE_STAGE_ALLOCATE) {
        let previous_entry = cache_entries[update_input.slot];
        cache_entries[update_input.slot] = RadianceCacheStorageEntry(
            previous_entry.radiance_confidence,
            previous_entry.generation_low,
            previous_entry.generation_high,
            atlas_base,
        );
        return;
    }
    if (update_input.reuse_committed_radiance != 0u) {
        if (params.stage == RADIANCE_CACHE_STAGE_BORDER_MIP) {
            let previous_entry = cache_entries[update_input.slot];
            cache_entries[update_input.slot] = RadianceCacheStorageEntry(
                previous_entry.radiance_confidence,
                update_input.generation_low,
                update_input.generation_high,
                previous_entry.atlas_base,
            );
        }
        return;
    }

    let entry = RadianceCacheStorageEntry(
        update_input.radiance_confidence,
        update_input.generation_low,
        update_input.generation_high,
        atlas_base,
    );
    if (params.stage == RADIANCE_CACHE_STAGE_TRACE) {
        let atlas_base = cache_entries[update_input.slot].atlas_base;
        var row = 1u;
        loop {
            if (row >= RADIANCE_CACHE_PROBE_TILE_EXTENT - 1u) {
                break;
            }
            var column = 1u;
            loop {
                if (column >= RADIANCE_CACHE_PROBE_TILE_EXTENT - 1u) {
                    break;
                }
                trace_atlas[atlas_base + row * RADIANCE_CACHE_PROBE_TILE_EXTENT + column] =
                    entry.radiance_confidence;
                column = column + 1u;
            }
            row = row + 1u;
        }
        return;
    }
    if (params.stage == RADIANCE_CACHE_STAGE_FILTER) {
        let atlas_base = cache_entries[update_input.slot].atlas_base;
        var row = 1u;
        loop {
            if (row >= RADIANCE_CACHE_PROBE_TILE_EXTENT - 1u) {
                break;
            }
            var column = 1u;
            loop {
                if (column >= RADIANCE_CACHE_PROBE_TILE_EXTENT - 1u) {
                    break;
                }
                let center = atlas_base + row * RADIANCE_CACHE_PROBE_TILE_EXTENT + column;
                let left = atlas_base + row * RADIANCE_CACHE_PROBE_TILE_EXTENT
                    + max(column - 1u, 1u);
                let right = atlas_base + row * RADIANCE_CACHE_PROBE_TILE_EXTENT
                    + min(column + 1u, RADIANCE_CACHE_PROBE_TILE_EXTENT - 2u);
                let up = atlas_base + max(row - 1u, 1u) * RADIANCE_CACHE_PROBE_TILE_EXTENT
                    + column;
                let down = atlas_base
                    + min(row + 1u, RADIANCE_CACHE_PROBE_TILE_EXTENT - 2u)
                        * RADIANCE_CACHE_PROBE_TILE_EXTENT
                    + column;
                filtered_atlas[center] = average_cross_rgba8(
                    trace_atlas[center],
                    trace_atlas[left],
                    trace_atlas[right],
                    trace_atlas[up],
                    trace_atlas[down],
                );
                column = column + 1u;
            }
            row = row + 1u;
        }
        return;
    }
    if (params.stage == RADIANCE_CACHE_STAGE_BORDER_MIP) {
        let atlas_base = cache_entries[update_input.slot].atlas_base;
        var row = 0u;
        loop {
            if (row >= RADIANCE_CACHE_PROBE_TILE_EXTENT) {
                break;
            }
            var column = 0u;
            loop {
                if (column >= RADIANCE_CACHE_PROBE_TILE_EXTENT) {
                    break;
                }
                let source_row = min(max(row, 1u), RADIANCE_CACHE_PROBE_TILE_EXTENT - 2u);
                let source_column = min(max(column, 1u), RADIANCE_CACHE_PROBE_TILE_EXTENT - 2u);
                final_atlas[atlas_base + row * RADIANCE_CACHE_PROBE_TILE_EXTENT + column] =
                    filtered_atlas[
                        atlas_base
                        + source_row * RADIANCE_CACHE_PROBE_TILE_EXTENT
                        + source_column
                    ];
                column = column + 1u;
            }
            row = row + 1u;
        }

        var mip_row = 0u;
        loop {
            if (mip_row >= 2u) {
                break;
            }
            var mip_column = 0u;
            loop {
                if (mip_column >= 2u) {
                    break;
                }
                let source_row = mip_row * 2u;
                let source_column = mip_column * 2u;
                let source0 = atlas_base + source_row * RADIANCE_CACHE_PROBE_TILE_EXTENT + source_column;
                let source1 = source0 + 1u;
                let source2 = source0 + RADIANCE_CACHE_PROBE_TILE_EXTENT;
                let source3 = source2 + 1u;
                final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP1_OFFSET + mip_row * 2u + mip_column] =
                    average_four_rgba8(
                        final_atlas[source0],
                        final_atlas[source1],
                        final_atlas[source2],
                        final_atlas[source3],
                    );
                mip_column = mip_column + 1u;
            }
            mip_row = mip_row + 1u;
        }
        final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP2_OFFSET] = average_four_rgba8(
            final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP1_OFFSET],
            final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP1_OFFSET + 1u],
            final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP1_OFFSET + 2u],
            final_atlas[atlas_base + RADIANCE_CACHE_PROBE_MIP1_OFFSET + 3u],
        );
        cache_entries[update_input.slot] = entry;
    }
}

@compute @workgroup_size(64, 1, 1)
fn cs_consume(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let consume_index = global_id.x;
    if (consume_index >= params.consume_count) {
        return;
    }
    let consume = consume_inputs[consume_index];
    var weighted_rgb = vec3<u32>(0u);
    var total_weight = 0u;
    var corner_index = 0u;
    loop {
        if (corner_index >= 8u) {
            break;
        }
        let slot = consume.slots[corner_index];
        if (slot >= RADIANCE_CACHE_SLOT_CAPACITY) {
            return;
        }
        let entry = cache_entries[slot];
        if (entry.generation_low != consume.generation_low ||
            entry.generation_high != consume.generation_high) {
            return;
        }
        let radiance_confidence =
            final_atlas[entry.atlas_base + RADIANCE_CACHE_PROBE_MIP2_OFFSET];
        let confidence_q8 = radiance_confidence >> 24u;
        let corner_weight =
            (consume.weights_q16[corner_index] * confidence_q8 + 32767u) / 65535u;
        let rgb = vec3<u32>(
            radiance_confidence & 0xffu,
            (radiance_confidence >> 8u) & 0xffu,
            (radiance_confidence >> 16u) & 0xffu,
        );
        weighted_rgb = weighted_rgb + rgb * corner_weight;
        total_weight = total_weight + corner_weight;
        corner_index = corner_index + 1u;
    }
    if (total_weight == 0u) {
        return;
    }

    let resolved_rgb = min(
        vec3<u32>(255u),
        (weighted_rgb + vec3<u32>(total_weight / 2u)) / total_weight,
    );
    let packed_rgb = resolved_rgb.x | (resolved_rgb.y << 8u) | (resolved_rgb.z << 16u);
    if (consume.resident_probe_index >= params.resident_probe_count) {
        return;
    }
    let resident_base = consume.resident_probe_index * RESIDENT_PROBE_WORD_COUNT;
    if (resident_probe_words[resident_base] != consume.probe_id) {
        return;
    }
    resident_probe_words[resident_base + RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET] =
        packed_rgb;
    atomicAdd(
        &marked_slots[
            RADIANCE_CACHE_DISPATCH_COUNTER_OFFSET + RADIANCE_CACHE_STAGE_CONSUME
        ],
        1u,
    );
}
