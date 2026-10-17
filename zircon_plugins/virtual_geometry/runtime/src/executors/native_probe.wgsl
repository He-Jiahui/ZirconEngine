// Inputs are GPU-produced rows. Uniform values are allocation/ABI bounds only.
struct Words { words: array<u32>, }
struct Params {
    mode: u32,
    kind: u32,
    output_capacity: u32,
    first_capacity: u32,
    authority_capacity: u32,
    first_stride: u32,
    authority_stride: u32,
    reserved: u32,
}
@group(0) @binding(0) var<storage, read> first: Words;
@group(0) @binding(1) var<storage, read> authority: Words;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read_write> result: Words;

const INVALID: u32 = 0xffffffffu;

fn reject() {
    // An impossible GPU count is a fail-closed error at the existing mapped reader.
    result.words[0] = INVALID;
}

fn authority_matches_submission(row: u32, input: u32) -> bool {
    let a = row * 14u;
    let identity = input + 1u;
    let bits = first.words[identity + 4u];
    if (bits & 6u) != 6u || (bits & ~15u) != 0u
        || ((bits & 1u) == 0u && first.words[identity + 5u] != INVALID) { return false; }
    if authority.words[a] != first.words[identity + 5u]
        || authority.words[a + 1u] != first.words[identity + 6u]
        || authority.words[a + 2u] != first.words[identity + 7u]
        || authority.words[a + 3u] != first.words[identity + 8u]
        || authority.words[a + 6u] != first.words[identity + 1u]
        || authority.words[a + 7u] != first.words[identity]
        || authority.words[a + 8u] != first.words[identity + 9u]
        || authority.words[a + 9u] != first.words[identity + 10u]
    { return false; }
    let start = first.words[identity + 11u];
    let span = first.words[identity + 12u];
    if span == 0u || start > INVALID - span { return false; }
    let ordinal = authority.words[a + 5u];
    if ordinal < start || ordinal >= start + span { return false; }
    if (bits & 8u) != 0u {
        if authority.words[a + 10u] != first.words[identity + 13u] { return false; }
    } else if authority.words[a + 10u] != INVALID {
        return false;
    }
    // Stable key and option-presence provenance came from validated terminal
    // full identity; authority14 cannot independently express those fields.
    return first.words[input + 16u] == 1u;
}

@compute @workgroup_size(1)
fn gather() {
    result.words[0] = 0u;
    if params.mode == 2u {
        if (params.kind != 3u && params.kind != 4u) ||
            params.authority_stride!=12u || arrayLength(&authority.words)<4u {reject();return;}
        let author_count=authority.words[0];
        let fixed_rows=authority.words[1];
        let resident_rows=authority.words[2];
        if author_count>params.authority_capacity || author_count>(arrayLength(&authority.words)-4u)/12u ||
            resident_rows>fixed_rows || authority.words[3]>1u {
            reject();return;
        }
        if params.kind==3u && (params.first_stride!=4u || arrayLength(&first.words)<1u ||
            fixed_rows>params.first_capacity || fixed_rows>(arrayLength(&first.words)-1u)/4u) {
            reject();return;
        }
        var count=fixed_rows;
        var stride=4u;
        if params.kind==4u {count=author_count;stride=12u;}
        if count>params.output_capacity || count>(arrayLength(&result.words)-1u)/stride {reject();return;}
        for(var row=0u;row<count;row++){
            for(var word=0u;word<stride;word++){
                if params.kind==3u {result.words[1u+row*stride+word]=first.words[1u+row*4u+word];}
                else {result.words[1u+row*stride+word]=authority.words[4u+row*12u+word];}
            }
        }
        result.words[0]=count;return;
    }
    if params.kind > 2u || params.mode > 1u || params.authority_stride != 14u
        || arrayLength(&first.words) == 0u
    { reject(); return; }
    let count = first.words[0];
    if count > params.first_capacity || count > params.output_capacity
        || params.first_capacity > (arrayLength(&first.words) - 1u) / params.first_stride
        || params.authority_capacity > arrayLength(&authority.words) / 14u
    { reject(); return; }
    var stride = 5u;
    if params.kind == 1u { stride = 1u; }
    if params.kind == 2u { stride = 6u; }
    if params.output_capacity > (arrayLength(&result.words) - 1u) / stride {
        reject(); return;
    }
    var written = 0u;
    for (var row = 0u; row < count; row = row + 1u) {
        let input = 1u + row * params.first_stride;
        var source_row = row;
        var token = 0u;
        if params.mode == 0u {
            if params.first_stride != 4u || row >= params.authority_capacity {
                reject(); return;
            }
            let vertex_count = first.words[input];
            let instances = first.words[input + 1u];
            if vertex_count == 0u || instances == 0u { continue; }
            if instances != 1u { reject(); return; }
            token = first.words[input + 3u];
        } else {
            if params.first_stride != 17u { reject(); return; }
            source_row = first.words[input];
            if source_row >= params.authority_capacity
                || !authority_matches_submission(source_row, input)
            { reject(); return; }
            token = first.words[input + 15u];
        }
        let a = source_row * 14u;
        let submission_index = authority.words[a + 8u];
        let draw_ref_rank = authority.words[a + 9u];
        let packed_token = (min(submission_index, 65535u) << 16u) | min(draw_ref_rank, 65535u);
        if token != packed_token { reject(); return; }
        let out = 1u + written * stride;
        if params.kind == 1u {
            result.words[out] = authority.words[a + 6u];
        } else {
            var base = out;
            if params.kind == 2u {
                result.words[base] = authority.words[a + 6u];
                base = base + 1u;
            }
            result.words[base] = authority.words[a + 1u];
            result.words[base + 1u] = authority.words[a + 2u];
            result.words[base + 2u] = authority.words[a + 3u];
            result.words[base + 3u] = submission_index;
            result.words[base + 4u] = draw_ref_rank;
        }
        written = written + 1u;
    }
    result.words[0] = written;
}
