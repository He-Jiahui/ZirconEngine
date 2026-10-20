@group(0) @binding(0) var<storage, read> visible_candidate_indices: array<u32>;
@group(0) @binding(1) var<storage, read> source_words: array<u32>;

struct IndirectArguments {
    count: atomic<u32>,
    words: array<u32>,
}

struct AuthorityRecords {
    words: array<u32>,
}

@group(0) @binding(2) var<storage, read_write> indirect_arguments: IndirectArguments;
@group(0) @binding(3) var<storage, read_write> authority_records: AuthorityRecords;

const SOURCE_WORDS: u32 = 19u;
const AUTHORITY_WORDS: u32 = 14u;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let source_count = source_words[0];
    if global_id.x >= source_count {
        return;
    }

    let source_offset = 1u + global_id.x * SOURCE_WORDS;
    let candidate_index = source_words[source_offset + 1u];
    let visible_count = min(visible_candidate_indices[0], arrayLength(&visible_candidate_indices) - 1u);
    var is_visible = false;
    for (var index = 0u; index < visible_count; index += 1u) {
        if visible_candidate_indices[index + 1u] == candidate_index {
            is_visible = true;
            break;
        }
    }

    let args_offset = global_id.x * 4u;
    if is_visible && source_words[source_offset + 3u] != 0u {
        atomicAdd(&indirect_arguments.count, 1u);
        indirect_arguments.words[args_offset] = source_words[source_offset + 3u];
        indirect_arguments.words[args_offset + 1u] = 1u;
    } else {
        indirect_arguments.words[args_offset] = 0u;
        indirect_arguments.words[args_offset + 1u] = 0u;
    }
    indirect_arguments.words[args_offset + 2u] = source_words[source_offset + 2u];
    indirect_arguments.words[args_offset + 3u] = source_words[source_offset + 4u];

    let authority_offset = global_id.x * AUTHORITY_WORDS;
    for (var word = 0u; word < AUTHORITY_WORDS; word += 1u) {
        authority_records.words[authority_offset + word] =
            source_words[source_offset + 5u + word];
    }
}


// New producer-owned wire. Header [author_count, fixed_args_rows, resident_rows,
// paired_prepare_present], followed by twelve-word author rows. Row word 7 is
// the actual execution submission slot, independent of resident-page slot. This is not a
// reinterpretation of the old CPU tuple projection.
@group(0) @binding(4) var<storage, read_write> segment_output: array<u32>;
@group(0) @binding(5) var<storage, read> author_input: array<u32>;

fn reject_segments() { segment_output[0]=0xffffffffu; }
@compute @workgroup_size(1)
fn cs_segments() {
    let count=author_input[0];
    let resident=author_input[1];
    let request_count=author_input[3];
    if count>4096u || resident>4096u || request_count>4096u ||
        arrayLength(&author_input)<4u+count*12u+request_count*2u ||
        arrayLength(&segment_output)<4u+count*12u {
        reject_segments();return;
    }
    let request_start=4u+count*12u;
    var pending=0u;
    // Validate the complete frozen table before mutating args reservations.
    for(var row=0u;row<count;row++){
        let src=4u+row*12u;
        if author_input[src+8u]>2u || author_input[src+4u]==0u ||
           author_input[src+3u]>0xffffffffu-author_input[src+4u] ||
           author_input[src+3u]+author_input[src+4u]>author_input[src+5u] {
            reject_segments();return;
        }
        if author_input[src+8u]==1u {pending++;}
    }
    if resident+pending>4096u || resident+pending>arrayLength(&indirect_arguments.words)/4u {
        reject_segments();return;
    }
    var reserved=resident;
    for(var row=0u;row<count;row++){
        let src=4u+row*12u;
        var rank=author_input[src+11u];
        var found=false;
        for(var req=0u;req<request_count;req++){
            let at=request_start+req*2u;
            if author_input[at]==author_input[src+6u] {
                if found && rank!=author_input[at+1u] {reject_segments();return;}
                rank=author_input[at+1u];found=true;
            }
        }
        if author_input[src+8u]==1u && !found {reject_segments();return;}
        let dst=4u+row*12u;
        for(var word=0u;word<12u;word++){segment_output[dst+word]=author_input[src+word];}
        segment_output[dst+11u]=rank;
        if author_input[src+8u]==1u {
            let arg=reserved*4u;
            // A valid inactive draw reservation. The frontier-order token is
            // scheduling metadata only; neither count nor authority is issued.
            indirect_arguments.words[arg]=0u;
            indirect_arguments.words[arg+1u]=0u;
            indirect_arguments.words[arg+2u]=0u;
            indirect_arguments.words[arg+3u]=rank;
            // This is the slot of the reservation just encoded above, not a
            // missing resident-page slot reinterpreted as zero by a reader.
            if segment_output[dst+7u]==0xffffffffu {
                segment_output[dst+7u]=reserved;
            }
            reserved++;
        }
    }
    segment_output[1]=reserved;
    segment_output[2]=resident;
    segment_output[3]=author_input[2];
    segment_output[0]=count;
}
