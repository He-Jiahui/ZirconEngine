mod documentation;
mod f12_current_state;
mod gate_wording;

use super::super::assert_contains_all;
use super::{read_repo, read_runtime_src};

const SLICE: &str = "Runtime 15 M3 runtime dead-code documentation anchor cleanup";
const STATUS: &str =
    "runtime_15_runtime_dead_code_documentation_anchor_cleanup_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_runtime_dead_code_documentation_anchors_use_folder_owner";
const CURRENT_ROOT_OWNER: &str = "structure_convention/runtime_dead_code/mod.rs";
const CURRENT_RUNTIME_UI_OWNER: &str = "structure_convention/runtime_dead_code/runtime_ui.rs";
const CURRENT_PRODUCTION_SCAN_OWNER: &str =
    "structure_convention/runtime_dead_code/production_scan.rs";
const STALE_FLAT_OWNER: &str = "structure_convention/runtime_dead_code.rs";
const MODULE_GATE_WORDING_SLICE: &str =
    "Runtime 15 M3 runtime dead-code module-gate status wording cleanup";
const MODULE_GATE_WORDING_STATUS: &str =
    "runtime_15_runtime_dead_code_module_gate_status_wording_static_passed_cargo_deferred";
const MODULE_GATE_WORDING_GUARD: &str =
    "runtime_15_runtime_dead_code_current_rows_keep_module_gate_audit_clear";
const CHILD_OWNER_SLICE: &str = "Runtime 15 M3 runtime dead-code guard child-owner split";
const MODULE_GATE_AUDIT_CLEAR: &str = "module_convention_gate audit clear";
const STALE_MODULE_GATE_PENDING: &str = "module_convention_gate` 与 full Cargo sweep 仍 pending";
const PRODUCTION_GATE_WORDING_SLICE: &str =
    "Runtime 15 M3 runtime dead-code production-gate status wording cleanup";
const PRODUCTION_GATE_WORDING_STATUS: &str =
    "runtime_15_runtime_dead_code_production_gate_status_wording_static_passed_cargo_deferred";
const PRODUCTION_GATE_WORDING_GUARD: &str =
    "runtime_15_runtime_dead_code_current_rows_use_production_gate_name";
const CURRENT_PRODUCTION_GATE: &str =
    "runtime_15_production_sources_do_not_allow_dead_code_suppression";
const STALE_PRODUCTION_GATE: &str =
    concat!("runtime_15_no_dead_code_", "suppression_in_production");
const F12_CURRENT_STATE_WORDING_SLICE: &str =
    "Runtime 15 F12 production dead-code current-state wording cleanup";
const F12_CURRENT_STATE_WORDING_STATUS: &str =
    "runtime_15_f12_production_dead_code_current_state_wording_static_passed_cargo_deferred";
const F12_CURRENT_STATE_WORDING_GUARD: &str =
    "runtime_15_f12_production_dead_code_current_state_is_zero_hit";
const CURRENT_F12_ZERO_HIT_WORDING: &str = "runtime production `allow(dead_code)` 零命中";
const STALE_OTHER_SUPPRESSION_SWEEP_PENDING: &str = "其他 suppression sweep 仍待 M5/T2 继续清理";
const STALE_FULL_CRATE_DEAD_CODE_SWEEP_PENDING: &str =
    "全 crate dead-code suppression sweep 仍待审计";

fn slice_entry<'a>(source: &'a str, slice: &str) -> Option<&'a str> {
    let start = source.find(slice)?;
    let rest = &source[start..];
    let end = ["\n## ", "\n| M"]
        .into_iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());
    Some(&rest[..end])
}
