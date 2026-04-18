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
    frontier_rank: u32,
    assigned_slot: u32,
    recycled_page_id: u32,
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

fn request_already_completed(page_id: u32, completed_count: u32) -> bool {
    var completed_index = 0u;
    loop {
        if (completed_index >= completed_count) {
            break;
        }

        let output_index = completed_index * 2u + 1u;
        if (completed_pages[output_index] == page_id) {
            return true;
        }

        continuing {
            completed_index = completed_index + 1u;
        }
    }

    return false;
}

fn slot_owner_page_id(slot: u32, page_table_entry_count: u32) -> u32 {
    var entry_index = 0u;
    loop {
        if (entry_index >= page_table_entry_count) {
            break;
        }

        let entry_offset = entry_index * 2u;
        if (page_table_entries[entry_offset + 1u] == slot) {
            return page_table_entries[entry_offset];
        }

        continuing {
            entry_index = entry_index + 1u;
        }
    }

    return 0xffffffffu;
}

fn slot_for_page_id(page_id: u32, page_table_entry_count: u32) -> u32 {
    var entry_index = 0u;
    loop {
        if (entry_index >= page_table_entry_count) {
            break;
        }

        let entry_offset = entry_index * 2u;
        if (page_table_entries[entry_offset] == page_id) {
            return page_table_entries[entry_offset + 1u];
        }

        continuing {
            entry_index = entry_index + 1u;
        }
    }

    return 0xffffffffu;
}

fn slot_already_completed(slot: u32, completed_count: u32) -> bool {
    var completed_index = 0u;
    loop {
        if (completed_index >= completed_count) {
            break;
        }

        let output_index = completed_index * 3u + 1u;
        if (completed_pages[output_index + 1u] == slot) {
            return true;
        }

        continuing {
            completed_index = completed_index + 1u;
        }
    }

    return false;
}

fn request_matches_explicit_slot_contract(
    request: PendingRequestInput,
    page_table_entry_count: u32,
) -> bool {
    if (request.assigned_slot == 0xffffffffu) {
        return true;
    }

    let slot_owner = slot_owner_page_id(request.assigned_slot, page_table_entry_count);
    if (request.recycled_page_id != 0xffffffffu) {
        return slot_owner == request.recycled_page_id;
    }

    return slot_owner == 0xffffffffu;
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x != 0u) {
        return;
    }

    var remaining_available_slots = params.available_slot_count;
    var remaining_bytes = params.streaming_budget_bytes + params.reclaimable_bytes;
    var completed_count = 0u;
    var next_available_slot = 0u;
    var next_evictable_slot = 0u;
    var appended_page_table_entries = 0u;

    loop {
        if (completed_count >= params.page_budget) {
            break;
        }

        var selected_request_index = 0xffffffffu;
        var selected_frontier_rank = 0xffffffffu;
        var candidate_index = 0u;
        loop {
            if (candidate_index >= params.pending_count) {
                break;
            }

            let candidate = pending_requests[candidate_index];
            if (
                candidate.size_bytes <= remaining_bytes
                    && !request_already_completed(candidate.page_id, completed_count)
                    && request_matches_explicit_slot_contract(
                        candidate,
                        params.resident_count + appended_page_table_entries,
                    )
                    && (
                        selected_request_index == 0xffffffffu
                            || candidate.frontier_rank < selected_frontier_rank
                            || (
                                candidate.frontier_rank == selected_frontier_rank
                                    && candidate_index < selected_request_index
                            )
                    )
            ) {
                selected_request_index = candidate_index;
                selected_frontier_rank = candidate.frontier_rank;
            }

            continuing {
                candidate_index = candidate_index + 1u;
            }
        }

        if (selected_request_index == 0xffffffffu) {
            break;
        }

        let request = pending_requests[selected_request_index];
        var assigned_slot = request.assigned_slot;
        if (assigned_slot != 0xffffffffu) {
        } else if (remaining_available_slots > 0u) {
            assigned_slot = available_slots[next_available_slot];
            next_available_slot = next_available_slot + 1u;
            remaining_available_slots = remaining_available_slots - 1u;
        } else {
            if (request.recycled_page_id != 0xffffffffu) {
                assigned_slot = slot_for_page_id(
                    request.recycled_page_id,
                    params.resident_count + appended_page_table_entries,
                );
            }

            if (assigned_slot == 0xffffffffu) {
                loop {
                    if (next_evictable_slot >= params.evictable_count) {
                        break;
                    }

                    let candidate_slot = evictable_slots[next_evictable_slot];
                    next_evictable_slot = next_evictable_slot + 1u;
                    if (slot_already_completed(candidate_slot, completed_count)) {
                        continue;
                    }

                    assigned_slot = candidate_slot;
                    break;
                }
            }
        }

        if (assigned_slot == 0xffffffffu) {
            break;
        }

        var recycled_page_id = request.recycled_page_id;
        if (recycled_page_id == 0xffffffffu) {
            recycled_page_id = slot_owner_page_id(
                assigned_slot,
                params.resident_count + appended_page_table_entries,
            );
        }

        let output_index = completed_count * 3u + 1u;
        completed_pages[output_index] = request.page_id;
        completed_pages[output_index + 1u] = assigned_slot;
        completed_pages[output_index + 2u] = recycled_page_id;

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

    completed_pages[0] = completed_count;
}
