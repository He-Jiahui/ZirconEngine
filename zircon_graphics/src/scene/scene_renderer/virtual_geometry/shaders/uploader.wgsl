struct VirtualGeometryUploaderParams {
    pending_count: u32,
    available_slot_count: u32,
    evictable_count: u32,
    page_budget: u32,
    streaming_budget_bytes: u32,
    reclaimable_bytes: u32,
    resident_count: u32,
    _padding1: u32,
};

struct PendingRequestInput {
    page_id: u32,
    size_bytes: u32,
};

@group(0) @binding(0)
var<uniform> params: VirtualGeometryUploaderParams;

@group(0) @binding(1)
var<storage, read> pending_requests: array<PendingRequestInput>;

@group(0) @binding(2)
var<storage, read> available_slots: array<u32>;

@group(0) @binding(3)
var<storage, read> evictable_slots: array<u32>;

@group(0) @binding(4)
var<storage, read_write> completed_pages: array<u32>;

@group(0) @binding(5)
var<storage, read_write> page_table_entries: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x != 0u) {
        return;
    }

    var remaining_available_slots = params.available_slot_count;
    var remaining_evictable_slots = params.evictable_count;
    var remaining_bytes = params.streaming_budget_bytes + params.reclaimable_bytes;
    var completed_count = 0u;
    var request_index = 0u;
    var next_available_slot = 0u;
    var next_evictable_slot = 0u;
    var appended_page_table_entries = 0u;

    loop {
        if (request_index >= params.pending_count) {
            break;
        }

        let request = pending_requests[request_index];
        if (request.size_bytes <= remaining_bytes) {
            var assigned_slot = 0xffffffffu;
            if (remaining_available_slots > 0u) {
                assigned_slot = available_slots[next_available_slot];
                next_available_slot = next_available_slot + 1u;
                remaining_available_slots = remaining_available_slots - 1u;
            } else if (remaining_evictable_slots > 0u) {
                assigned_slot = evictable_slots[next_evictable_slot];
                next_evictable_slot = next_evictable_slot + 1u;
                remaining_evictable_slots = remaining_evictable_slots - 1u;
            }

            if (assigned_slot == 0xffffffffu) {
                break;
            }

            let output_index = completed_count * 2u + 1u;
            completed_pages[output_index] = request.page_id;
            completed_pages[output_index + 1u] = assigned_slot;

            var resident_index = 0u;
            var replaced_existing_slot = false;
            loop {
                if (resident_index >= params.resident_count) {
                    break;
                }
                let resident_offset = resident_index * 2u;
                if (page_table_entries[resident_offset + 1u] == assigned_slot) {
                    page_table_entries[resident_offset] = request.page_id;
                    replaced_existing_slot = true;
                    break;
                }

                continuing {
                    resident_index = resident_index + 1u;
                }
            }

            if (!replaced_existing_slot) {
                let page_table_offset = (params.resident_count + appended_page_table_entries) * 2u;
                page_table_entries[page_table_offset] = request.page_id;
                page_table_entries[page_table_offset + 1u] = assigned_slot;
                appended_page_table_entries = appended_page_table_entries + 1u;
            }

            completed_count = completed_count + 1u;
            remaining_bytes = remaining_bytes - request.size_bytes;
        }

        continuing {
            request_index = request_index + 1u;
        }
    }

    completed_pages[0] = completed_count;
}
