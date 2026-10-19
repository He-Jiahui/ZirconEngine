// Existing producer ABIs remain count+4 words and 14 authority words.
// Each author run gets its own indexed command, including duplicate candidates.
@group(0) @binding(0) var<storage, read> source_args: array<u32>;
@group(0) @binding(1) var<storage, read> source_authority: array<u32>;
@group(0) @binding(2) var<storage, read> author: array<u32>;
@group(0) @binding(3) var<storage, read_write> indexed_args: array<u32>;
@group(0) @binding(4) var<storage, read_write> sidecar: array<u32>;
@group(0) @binding(5) var<storage, read_write> status: array<u32>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let slot = id.x;
    if arrayLength(&author) < 4u { return; }
    let run_count = author[0];
    let source_count = author[1];
    let capacity = author[2];
    if slot >= capacity { return; }
    if slot >= arrayLength(&status) || slot >= arrayLength(&indexed_args) / 5u
        || slot >= arrayLength(&sidecar) / 32u { return; }
    for (var k = 0u; k < 5u; k++) { indexed_args[slot * 5u + k] = 0u; }
    for (var k = 0u; k < 32u; k++) { sidecar[slot * 32u + k] = 0u; }
    status[slot] = 0u;
    if slot >= run_count { return; }
    if source_count == 0u || source_count > 4096u || run_count > capacity
        || capacity > 65536u || arrayLength(&source_args) < 1u + source_count * 4u
        || arrayLength(&source_authority) < source_count * 14u
        || arrayLength(&author) < 4u + source_count * 14u + run_count * 25u {
        status[slot] = 1u; return;
    }
    let run = 4u + source_count * 14u + slot * 25u;
    let row = author[run];
    if row >= source_count { status[slot] = 1u; return; }
    let identity = 4u + row * 14u;
    if author[run + 10u] > 1u {status[slot] = 2u; return;}
    let projected = author[run + 10u] == 1u;
    for (var k = 0u; k < 14u; k++) {
        if author[run + 11u + k] != author[identity + k] &&
            !(projected && (k == 1u || k == 10u)) {
            status[slot] = 2u; return;
        }
    }
    let flags = author[identity + 4u];
    if (flags & 6u) != 6u || (flags & 0xfffffff0u) != 0u
        || ((flags & 1u) == 0u && author[identity + 5u] != 0xffffffffu) {
        status[slot] = 2u; return;
    }
    let authority = row * 14u;
    // Compare every available identity field against actual producer authority.
    if source_authority[authority] != author[identity + 5u]
        || source_authority[authority + 1u] != author[identity + 6u]
        || source_authority[authority + 2u] != author[identity + 7u]
        || source_authority[authority + 3u] != author[identity + 8u]
        || source_authority[authority + 4u] != author[run + 2u]
        || source_authority[authority + 5u] != author[run + 3u]
        || source_authority[authority + 6u] != author[identity + 1u]
        || source_authority[authority + 7u] != author[identity]
        || source_authority[authority + 8u] != author[identity + 9u]
        || source_authority[authority + 9u] != author[identity + 10u]
        || source_authority[authority + 10u] != author[identity + 13u]
        || source_authority[authority + 11u] != author[run + 1u] {
        status[slot] = 2u; return;
    }
    let ordinal = author[run + 3u];
    let cluster_start = author[identity + 11u];
    let cluster_count = author[identity + 12u];
    if cluster_count == 0u || ordinal < cluster_start || ordinal - cluster_start >= cluster_count {
        status[slot] = 2u; return;
    }
    let first_index = author[run + 4u];
    let index_count = author[run + 5u];
    let source_indices = author[run + 9u];
    if index_count == 0u || first_index % 3u != 0u || index_count % 3u != 0u
        || first_index > source_indices || index_count > source_indices - first_index {
        status[slot] = 4u; return;
    }
    let args = 1u + row * 4u;
    let vertices = source_args[args];
    let instances = source_args[args + 1u];
    if source_args[0] > source_count || instances > 1u
        || (instances == 0u && vertices != 0u) || (instances == 1u && vertices == 0u) {
        status[slot] = 3u; return;
    }
    for (var k = 0u; k < 14u; k++) {
        sidecar[slot * 32u + k] = source_authority[authority + k];
        sidecar[slot * 32u + 14u + k] = author[run + 11u + k];
    }
    // Only proven authored topology may project draw reference/rank; every
    // other canonical authority field is retained byte for byte.
    sidecar[slot * 32u + 6u] = author[run + 12u];
    sidecar[slot * 32u + 9u] = author[run + 21u];
    sidecar[slot * 32u + 28u] = row;
    sidecar[slot * 32u + 29u] = source_authority[authority + 11u];
    sidecar[slot * 32u + 30u] = author[run + 8u];
    sidecar[slot * 32u + 31u] = instances;
    if instances == 1u {
        indexed_args[slot * 5u] = index_count;
        indexed_args[slot * 5u + 1u] = instances;
        indexed_args[slot * 5u + 2u] = first_index;
        indexed_args[slot * 5u + 3u] = author[run + 6u];
        // This is the real GPUScene span, never the old debug visibility token.
        indexed_args[slot * 5u + 4u] = author[run + 7u];
    }
}
