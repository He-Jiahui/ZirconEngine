const CULL_INPUT_FORCED_MIP_INDEX: u32 = 11u;
const GLOBAL_STATE_WORD_COUNT: u32 = 56u;
const DISPATCH_SETUP_WORD_COUNT: u32 = 7u;
const LAUNCH_HEADER_WORD_COUNT: u32 = GLOBAL_STATE_WORD_COUNT + DISPATCH_SETUP_WORD_COUNT;
const INSTANCE_SEED_WORD_COUNT: u32 = 7u;
const INSTANCE_WORK_ITEM_WORD_COUNT: u32 = 10u;
const DISPATCH_SETUP_INSTANCE_SEED_COUNT_INDEX: u32 = GLOBAL_STATE_WORD_COUNT;
const DISPATCH_SETUP_CLUSTER_BUDGET_INDEX: u32 = GLOBAL_STATE_WORD_COUNT + 1u;
const DISPATCH_SETUP_PAGE_BUDGET_INDEX: u32 = GLOBAL_STATE_WORD_COUNT + 2u;

@group(0) @binding(0)
var<storage, read> launch_worklist_words: array<u32>;

@group(0) @binding(1)
var<storage, read_write> instance_work_item_words: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let work_item_index = global_id.x;
    let instance_seed_count = launch_worklist_words[DISPATCH_SETUP_INSTANCE_SEED_COUNT_INDEX];
    if (work_item_index >= instance_seed_count) {
        return;
    }

    let seed_base = LAUNCH_HEADER_WORD_COUNT + work_item_index * INSTANCE_SEED_WORD_COUNT;
    let work_item_base = work_item_index * INSTANCE_WORK_ITEM_WORD_COUNT;

    instance_work_item_words[work_item_base] = launch_worklist_words[seed_base];
    instance_work_item_words[work_item_base + 1u] = launch_worklist_words[seed_base + 1u];
    instance_work_item_words[work_item_base + 2u] = launch_worklist_words[seed_base + 2u];
    instance_work_item_words[work_item_base + 3u] = launch_worklist_words[seed_base + 3u];
    instance_work_item_words[work_item_base + 4u] = launch_worklist_words[seed_base + 4u];
    instance_work_item_words[work_item_base + 5u] = launch_worklist_words[seed_base + 5u];
    instance_work_item_words[work_item_base + 6u] = launch_worklist_words[seed_base + 6u];
    instance_work_item_words[work_item_base + 7u] =
        launch_worklist_words[DISPATCH_SETUP_CLUSTER_BUDGET_INDEX];
    instance_work_item_words[work_item_base + 8u] =
        launch_worklist_words[DISPATCH_SETUP_PAGE_BUDGET_INDEX];
    instance_work_item_words[work_item_base + 9u] =
        launch_worklist_words[CULL_INPUT_FORCED_MIP_INDEX];
}
