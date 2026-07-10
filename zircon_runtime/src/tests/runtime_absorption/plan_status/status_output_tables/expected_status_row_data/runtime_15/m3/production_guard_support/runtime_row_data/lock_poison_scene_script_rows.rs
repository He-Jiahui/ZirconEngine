type Slice = super::ExpectedStatusOutputSlice;

#[path = "lock_poison_scene_script_rows/lock_poison.rs"]
mod lock_poison;
#[path = "lock_poison_scene_script_rows/scene_script.rs"]
mod scene_script;

const COMBINED_ROWS: [Slice; 8] = [
    lock_poison::ROWS[0],
    lock_poison::ROWS[1],
    lock_poison::ROWS[2],
    lock_poison::ROWS[3],
    scene_script::ROWS[0],
    scene_script::ROWS[1],
    scene_script::ROWS[2],
    scene_script::ROWS[3],
];

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &COMBINED_ROWS;
