---
related_code:
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_many_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_unique_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/ecs/query/mod.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - zircon_runtime/src/scene/tests/ecs_query_state_structure.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_query_many.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
implementation_files:
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/ecs/query/query_access.rs
  - zircon_runtime/src/scene/ecs/query/query_many_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_many_unique_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs
  - zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs
  - zircon_runtime/src/scene/ecs/query/cached_query_iter.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - zircon_runtime/src/scene/tests/ecs_query_state_structure.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/bevy/crates/bevy_ecs/src/query/iter.rs
  - dev/bevy/crates/bevy_ecs/src/query/access.rs
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/cached_query_iter.rs zircon_runtime/src/scene/ecs/query/query_access.rs zircon_runtime/src/scene/ecs/query/query_many_iter.rs zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_many_unique_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_iter.rs zircon_runtime/src/scene/ecs/query/query_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_state/*.rs zircon_runtime/src/scene/ecs/system/query.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs zircon_runtime/src/scene/tests/mod.rs
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_many --locked --jobs 1 --message-format short
  - rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\query\query_many_iter.rs (2026-06-05 cached read-only many borrow fix: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\query\query_many_iter.rs zircon_runtime\src\scene\ecs\query\query_state\read_only.rs zircon_runtime\src\scene\tests\ecs_query_structure.rs (2026-06-05 cached many component-location path: passed)
  - cached-many component-location source guard for borrowed stable/component locations, `F::matches_component_locations(...)`, `D::fetch_with_component_locations(...)`, no uncached `D::fetch_with_ticks(...)`, and uncached many validation preservation (2026-06-05 cached many component-location path: passed)
  - cached get/contains component-location source guard for `cached_entity_location(...)`, `F::matches_component_locations(...)`, `D::fetch_with_component_locations(...)`, no uncached filter/fetch, and `NotSpawned` precheck preservation (2026-06-05 cached read-only get/contains component-location path: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\query\query_state\mod.rs zircon_runtime\src\scene\ecs\query\query_state\read_only.rs zircon_runtime\src\scene\ecs\query\query_state\read_only_cached.rs zircon_runtime\src\scene\tests\ecs_query_structure.rs (2026-06-05 cached read-only owner split: passed)
  - read-only cached owner source guard in `scene::tests::ecs_query_structure` for cached APIs staying out of `query_state/read_only.rs` and inside `query_state/read_only_cached.rs` (2026-06-05 cached read-only owner split: passed)
  - shared cache-slot source guard for `QueryState::cached_entity_location(...)` and cached owner usage in `cached_direct.rs` plus `read_only_cached.rs` (2026-06-05 shared cache slot helper: passed)
  - rustfmt --edition 2021 --check over all `zircon_runtime/src/scene/ecs/query/*.rs`, all `query_state/*.rs`, and `scene/tests/ecs_query_structure.rs` (2026-06-05 full ECS query static closeout: passed)
  - targeted `ecs_query_state_boundary_audit(...)` over the repository root (2026-06-05 full ECS query static closeout: passed, 7/7 owner modules, no oversized modules, no risks)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 full ECS query static closeout: started, timed out after 20 minutes with no visible result while compiling; rerun deferred because other sessions started Cargo lanes)
  - cached component-location fail-closed source guard in `scene::tests::ecs_query_structure` for `cached_query_iter.rs`, `query_iter.rs`, `query_many_iter.rs`, and `query_state/mod.rs` (2026-06-05 cache vector drift guard: passed)
  - flat cached component-location source guard in `scene::tests::ecs_query_structure` for `QueryState.cached_component_locations: Vec<ComponentStorageLocation>`, `cached_component_location_offsets: Vec<usize>`, and no `Vec<Vec<ComponentStorageLocation>>` in the state owner (2026-06-05 flat component-location cache: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/tests/ecs_query.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 flat component-location test adaptation: passed)
  - static old-cache access scan over zircon_runtime/src/scene/tests/ecs_query.rs for `cached_component_locations()[i][j]`, `.as_slice()`, and `map(|locations| locations[i])` (2026-06-05 flat component-location test adaptation: passed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 flat component-location test adaptation: compiled and ran; 8 passed, 2 source-guard assertions failed before guard text was updated)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 guard rerun: blocked during compile by unrelated active render-lane `zircon_runtime/src/core/framework/render/post_process/stack.rs` E0382 moved `effect_stack_after`)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 post-render-unblock rerun: passed, 10 passed; 0 failed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_many --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 flat component-location regression: passed, 9 passed; 0 failed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_scheduled_native_systems --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 flat component-location regression: passed, 7 passed; 0 failed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 flat component-location regression: passed, 44 passed; 0 failed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_many_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_many_unique_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_state/mutable.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 mutable cached component-location filters: passed)
  - static source scan over mutable query paths for `cached_query_component_locations(...)`, `F::matches_component_locations(...)`, and no `F::matches(world, entity, ...)` / `D::matches_data(world, entity)` in cached mutable paths (2026-06-05 mutable cached component-location filters: passed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 mutable cached component-location filters: passed, 10 passed; 0 failed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_many --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 mutable cached component-location behavior rerun: passed, 9 passed; 0 failed; existing warning noise only)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_scheduled_native_systems --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 mutable cached component-location behavior rerun: passed, 7 passed; 0 failed; existing warning noise only)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 mutable cached component-location behavior rerun: passed, 44 passed; 0 failed; existing warning noise only)
  - rustfmt --edition 2021 zircon_runtime\src\scene\ecs\query\query_state\mod.rs zircon_runtime\src\scene\ecs\query\query_state\stats.rs zircon_runtime\src\scene\ecs\query\mod.rs zircon_runtime\src\scene\ecs\mod.rs zircon_runtime\src\scene\tests\ecs_query_structure.rs zircon_runtime\src\scene\tests\ecs_performance_acceptance.rs (2026-06-13 Runtime 07 query cache telemetry slice: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\query\query_state\mod.rs zircon_runtime\src\scene\ecs\query\query_state\stats.rs zircon_runtime\src\scene\ecs\query\mod.rs zircon_runtime\src\scene\ecs\mod.rs zircon_runtime\src\scene\tests\ecs_query_structure.rs zircon_runtime\src\scene\tests\ecs_performance_acceptance.rs (2026-06-13 Runtime 07 query cache telemetry slice: passed)
  - query-state non-empty line budget check after telemetry split: `query_state/mod.rs` = 174 non-empty lines, below `QUERY_STATE_ROOT_NON_EMPTY_LINE_BUDGET = 180` (2026-06-13 Runtime 07 query cache telemetry slice: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never (2026-06-13 Runtime 07 query/change telemetry: passed with existing warning set)
  - cargo test -p zircon_runtime --lib query_state_cache_stats_record_reuse_and_rebuild_counts --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 07 query cache telemetry: passed, 1 passed; 0 failed)
  - cargo test -p zircon_runtime --lib query_state_reuses_archetype_matches_across_unchanged_frames --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-07-query-0613 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 07 M1.2 named assertion: pending after render-owned HZB compile blocker clears; source/rustfmt static checks passed)
  - ecs_query_state_boundary structural audit sync for `expected_module_count = 8`, `unexpected_modules = []`, and `risks = []` after accepting `query_state/stats.rs` as the Runtime 07 cache telemetry owner (2026-06-13 QueryState boundary audit: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs zircon_runtime/src/scene/ecs/query/query_state/mutable.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 cached combination component-location filters: passed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 cached combination component-location filters: passed, 10 passed; 0 failed; existing warning noise only)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_combinations --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 cached combination component-location filters: blocked by unrelated `zircon_runtime/src/ui/tests/runtime_drag_drop_component_state.rs` calls to missing `UiTree::insert_root` and `UiTree::insert_child`)
  - E:\cargo-targets\zircon-ecs-query-flat-cache-0605\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe scene::tests::ecs_query_combinations --test-threads=1 --nocapture (2026-06-05 cached combination component-location filters: passed, 4 passed; 0 failed; direct run from the freshly compiled structure-test binary)
  - E:\cargo-targets\zircon-ecs-query-flat-cache-0605\debug\deps\zircon_runtime-b34ee8d8fc52f1fd.exe scene::tests::ecs_query --test-threads=1 --nocapture (2026-06-05 cached combination component-location filters: passed, 44 passed; 0 failed; direct run from the freshly compiled structure-test binary)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs zircon_runtime/src/scene/ecs/query/query_state/read_only.rs zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs zircon_runtime/src/scene/ecs/system/query.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 cached combination borrowed cache-slot candidates: passed)
  - cached combination source guard for `QueryCombinationCandidates<'state>`, borrowed `EntityId`/stable/component-location cache slices, `cache_indices: Vec<usize>`, no `matched_entities` / `matched_stable_locations` / `matched_component_locations` / `extend_from_slice`, and cached fetch through `D::fetch_with_component_locations(...)` (2026-06-05 cached combination borrowed cache-slot candidates: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never (2026-06-05 cached combination borrowed cache-slot candidates: passed with existing warning noise only)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 cached combination borrowed cache-slot candidates: timed out after 7 minutes while compiling under concurrent external Cargo lanes; no result claimed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_state/read_only.rs zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs zircon_runtime/src/scene/ecs/query/query_state/mutable.rs zircon_runtime/src/scene/ecs/system/query.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 mutable cached combination cache-slot candidates: passed)
  - mutable cached combination source guard for `QueryCombinationMutCandidates<'state>`, borrowed `EntityId` cache slice, `cache_indices: Vec<usize>`, no `then_some(entity)` / copied matched buffers in the cached mutable constructor, and combination enumeration through `self.candidates.entity(...)` plus `self.candidates.len()` (2026-06-05 mutable cached combination cache-slot candidates: passed)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ecs-query-flat-cache-0605 --message-format short --color never (2026-06-05 mutable cached combination cache-slot candidates: blocked before ECS acceptance by unrelated `zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs` E0061: method takes 28 arguments but 26 were supplied)
  - git diff --check -- zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs zircon_runtime/src/scene/ecs/query/query_state/mutable.rs zircon_runtime/src/scene/ecs/system/query.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs docs/zircon_runtime/scene/ecs/query_state.md .codex/sessions/20260604-1232-runtime-architecture-review.md (2026-06-05 mutable cached combination cache-slot candidates: passed with expected LF-to-CRLF warnings only)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_combinations_mut_iter.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 mutable combination owned-branch hard cutover: passed)
  - mutable combination owned-branch source guard for `struct QueryCombinationMutCandidates<'state>`, cache-only `new_from_cached_entities(...)`, no `pub(crate) fn new<EntityList>`, no `QueryCombinationMutCandidates::Owned`, no mutable `D::matches_data(world, *entity)` constructor scan, and structure-test rejection of the removed branch (2026-06-05 mutable combination owned-branch hard cutover: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_combinations_iter.rs zircon_runtime/src/scene/ecs/query/query_state/read_only.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-08 uncached read-only combination vector sizing: passed)
  - uncached read-only combination source guard for `read_only_combination_candidate_count(...)`, `Vec::with_capacity(candidate_count)`, direct matched entity pushes, no owned constructor `.collect::<Vec<_>>()`, and slice-backed `QueryCombinationIter::new(...)` input (2026-06-08 uncached read-only combination vector sizing: passed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure::cached_combinations_trust_query_state_data_membership --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-query-combo-read-vector-0608 --message-format short --color never -- --exact --test-threads=1 --nocapture (2026-06-08 uncached read-only combination vector sizing: attempted; failed during unrelated `zircon_runtime` lib-test compilation before reaching the ECS guard because `SceneMeshInstanceAsset` initializers in asset/scene/graphics tests were missing `depth_bias`, `material_queue`, and `render_queue`; no ECS Cargo pass/fail claimed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_access.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs plus query-access sorted-insertion source guard, conflict-marker scan, trailing-whitespace scan, git diff --check, and audit_runtime_structure.py --json (2026-06-05 query-access binary-position insertion: passed; Cargo deferred because active external runtime/workspace compile lanes were present)
  - query-cache candidate-location visitor source guard in `scene::tests::ecs_query_structure` for `World::matching_query_archetypes(...)`, `matching_query_archetype_entity_count(...)`, `visit_entity_locations_matching_archetypes(...)`, `QueryState::update_cache(...)` direct visitor use, and rejection of the old `entity_locations_matching_query_archetypes(...)` candidate-vector handoff (2026-06-11 query-cache candidate-location visitor: static validation passed; Cargo deferred because unrelated UI interface and render-lane Cargo/rustc work was active during closeout)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs zircon_runtime/src/scene/ecs/query/query_state/read_only.rs zircon_runtime/src/scene/ecs/query/query_state/mutable.rs zircon_runtime/src/scene/tests/ecs_query_state_structure.rs zircon_runtime/src/scene/tests/mod.rs (2026-06-11 query-state item-fetch direct branches: static validation passed)
  - query-state item-fetch source guard in `scene::tests::ecs_query_state_structure` for direct `let Some(item) = ... else` fetch branches in cached direct, read-only cached, read-only, and mutable query-state entry points, mutable single-query matched-entity branch, and rejection of `.ok_or(...)` in targeted query-state files (2026-06-11 query-state item-fetch direct branches: static validation passed)
  - cargo validation for query-state item-fetch direct branches (2026-06-11: deferred to the M5 milestone testing stage after static validation; no Cargo command was started and no Cargo pass/fail is claimed)
  - python -m py_compile .codex\skills\zircon-project-skills\zr-runtime-interface-convergence\scripts\runtime_structure_audits\ecs_query_state_boundary.py (2026-06-05 cached read-only owner split: passed)
  - targeted `ecs_query_state_boundary_audit(...)` over the repository root (2026-06-05 cached read-only owner split: passed)
  - rustfmt --edition 2021 --check zircon_runtime\src\scene\ecs\query\query_iter.rs zircon_runtime\src\scene\tests\ecs_query_structure.rs (2026-06-05 cached full read-only QueryIter membership: passed)
  - cached QueryIter source guard for dynamic filter, cached fetch, no repeated `D::matches_component_locations`, and uncached `D::matches_data` validation (2026-06-05 cached full read-only QueryIter membership: passed)
  - targeted `ecs_query_state_boundary_audit(...)` over the repository root (2026-06-05 cached full read-only QueryIter membership: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/scene/ecs/query/query_state/mod.rs zircon_runtime/src/scene/ecs/query/query_mut_iter.rs zircon_runtime/src/scene/tests/ecs_query_structure.rs (2026-06-05 QueryMutIter cached-entity accessor compile unblock: passed)
  - cargo test -p zircon_runtime --lib scene::tests::ecs_query_structure::query_mut_iter_uses_borrowed_cached_entities_without_recollecting --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-slot-z-order-0605 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-05 QueryMutIter cached-entity accessor compile unblock: passed, 1 passed; 0 failed; 2797 filtered out)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short (2026-06-05 cached read-only many borrow fix: passed with existing warnings)
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-layout-slot-z-order-0605 --message-format short --color never (2026-06-05 QueryMutIter cached-entity accessor compile unblock: passed with existing warning noise only)
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py (2026-06-21 QueryState Markdown renderer split: passed)
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
doc_type: module-detail
---

# QueryState Owner Split

`zircon_runtime::scene::ecs::query::QueryState` owns ECS query access, matched entity cache state, and the `SystemParam` bridge used by runtime systems. The public type remains exported through `zircon_runtime::scene::ecs::QueryState`; only the source ownership changed.

The split follows the local query directory and Bevy's `bevy_ecs::query` precedent: keep query access, data, filters, iterators, and state in separate owner files instead of letting one state file own every read, mutable, and cached path.

## Owner Files

- `query_state/mod.rs` owns the `QueryState` struct, construction, access descriptors, cache fields, and cache telemetry fields.
- `query_state/cache.rs` owns cache rebuilds, cache-slot lookup, cached entity/component-location accessors, and cache metadata accessors.
- `query_state/cached_direct.rs` owns `CachedQueryData` and `CachedQueryFilter` paths that fetch directly from cached component storage locations.
- `query_state/read_only.rs` owns uncached non-mutating `QueryData` iteration, `get`, `many`, `contains`, and combination APIs.
- `query_state/read_only_cached.rs` owns cached non-mutating `QueryData` iteration, cached many, cached get/contains, and cached combination APIs.
- `query_state/stats.rs` owns the Runtime 07 cache telemetry snapshot API and change-detection telemetry accumulation.
- `query_state/mutable.rs` owns `QueryMutData` access, mutable alias validation, mutable many/combination iteration, and the narrow unsafe fetch used after duplicate checks.
- `cached_query_iter.rs` owns direct cached query iteration over precomputed component-storage locations.
- `query_many_iter.rs` owns caller-provided entity-list iteration for uncached and read-only cached many-query paths.
- `query_mut_iter.rs` owns full mutable query iteration over the cached entity slice.
- `query_many_mut_iter.rs` and `query_many_unique_mut_iter.rs` own many-target mutable iteration over caller-provided entity lists.
- `query_combinations_iter.rs` and `query_combinations_mut_iter.rs` own K-combination enumeration for read-only and mutable query data.
- `query_state/many_item_array.rs` owns shared many-query fixed-array collection helpers.
- `query_state/system_param.rs` owns the `SystemParam` implementation that turns query state into `Query<'world, D, F>`.

## Mutable Many Cache Membership

The M5 performance pass keeps `QueryState` as the cache authority and changes `QueryManyMutIter` plus `QueryManyUniqueMutIter` to borrow `cached_entity_indices` from the refreshed state. Membership checks now reuse `cached_query_entity_index`, so each requested entity uses the sorted cache's binary lookup instead of cloning `cached_entities` at iterator construction and then scanning it with `Vec::contains`.

This mirrors Bevy ECS's separation between `QueryState` and many-query iterator state while staying inside Zircon's current safety model. `QueryManyMutIter` still exposes `fetch_next` rather than implementing `Iterator`, because duplicate caller input can produce mutable aliases. `QueryManyUniqueMutIter` remains an `Iterator` because the `UniqueEntityArray` input proves the entity list has no duplicates.

The mutable cached-many iterators trust cache membership for entity existence and `QueryData` shape. After the binary cache lookup succeeds, they resolve the flat component-location slice for the cache slot and run `F::matches_component_locations(...)` for the current tick window before fetching the mutable item. This keeps `Added` and `Changed` filters correct without repeating structural `D::matches_data` checks or returning to world-level filter lookups that `QueryState::update_cache` already made unnecessary.

`for_each_mut`, `single_mut`, and mutable get validation now follow the same cache-authority rule for all-entity mutable paths: they refresh `QueryState`, walk `cached_entities` by cache slot, recover each slot's flat component-location slice, and only reapply the current filter window through `F::matches_component_locations(...)` before fetching. `for_each_mut` no longer allocates a cloned entity list before invoking callbacks. Mutable combinations still keep their dedicated iterator path, because they need tuple-combination ownership rather than a single callback item stream, but the cached branch now enumerates over matched cache-slot indices instead of copying a matched entity list.

## Full Mutable Cache Iteration

`QueryMutIter` also borrows the refreshed `QueryState.cached_entities` slice through the crate-internal `cached_entities()` accessor. The iterator stores the slice, the flat component-location buffer, component-location offsets, and an index cursor, so `QueryState::iter_mut_with_ticks(...)` no longer rebuilds a temporary `Vec<EntityId>` from cached locations and no longer performs world-level dynamic filter lookups before yielding mutable query items.

The iterator's state lifetime is explicit in `QueryMutIter<'world, 'state, D, F>`, and the system-facing `Query::iter_mut` return type preserves that borrow. Mutable safety still comes from the cache's unique entity ids plus the iterator's single-step fetch model: each yielded item is fetched from one stable entity, and the iterator only advances after the previous item is out of the callback/loop body. Because the entity slice is owned by the refreshed cache, the iterator skips redundant entity-existence and `D::matches_data` checks and only evaluates `F::matches(...)`.

## Full Read-Only Cache Iteration

`QueryIter::new_cached_locations(...)` is only constructed from `QueryState::iter_cached_with_ticks(...)`, after `QueryState::update_cache(...)` refreshes `cached_entities`, `cached_locations`, the flat `cached_component_locations` buffer, and `cached_component_location_offsets`. The cached branch therefore treats `QueryState` as the structural authority for `QueryData` shape and no longer repeats `D::matches_component_locations(...)` for each yielded entity.

The iterator still runs `F::matches_component_locations(...)` for the current tick window before fetching. This keeps dynamic `Added` and `Changed` filters correct while avoiding the repeated tuple-data component-location scans that cache construction already paid for. `D::fetch_with_component_locations(...)` remains the final stale-location guard and simply skips the entity if the cache invariant is ever invalidated.

## Read-Only Many Cache Membership

The read-only cached-many path now mirrors the mutable-many cache-membership model while using the same component-location path as full cached read-only iteration. `QueryManyCachedIter` borrows `QueryState.cached_entity_indices`, `cached_locations`, the flat `cached_component_locations` buffer, and `cached_component_location_offsets`, then walks the caller-provided entity iterator directly. Each requested entity uses `cached_query_entity_index` for binary membership lookup; the resulting cache slot supplies the stable location and component-location slice for dynamic filtering and fetch.

This removes the older `cached_many_entities` helper that allocated a filtered `Vec<EntityId>` for every `iter_many_cached` call. Duplicate requested entities still preserve their requested order because the iterator consumes the original caller sequence, not the sorted cache. Non-matching or missing entities are still skipped by the same query validation path that backed the previous `QueryManyIter` implementation.

`QueryManyCachedIter::next` must keep the entity iterator borrow separate from the read-only world/cache/tick inputs. The iterator copies `world`, `cached_entity_indices`, `cached_locations`, the flat component-location buffer, component-location offsets, and `ticks` into locals before entering `self.entities.by_ref()`, then performs the binary cache lookup, offset-slice resolution, `F::matches_component_locations(...)`, and `D::fetch_with_component_locations(...)` from those locals. That preserves request order and duplicate-request semantics while avoiding the uncached world lookup path after cache membership succeeds.

`contains_cached(...)`, `get_cached(...)`, and cached many get paths use the same slot-level helper after `QueryState::update_cache(...)`. The helper resolves one cache slot into a stable location plus component-location slice; `contains_cached(...)` then only runs the component-location filter, while get paths run the same filter and `D::fetch_with_component_locations(...)`. `get_cached(...)` still checks `World::contains_entity(...)` before cache membership so the public error stays `NotSpawned` for missing entities and `QueryDoesNotMatch` for spawned-but-filtered entities.

`QueryState::cached_entity_location(...)` is the shared cache-slot resolver for cached read-only and direct cached owners. It centralizes the invariant that `cached_entity_indices`, `cached_locations`, `cached_component_locations`, and `cached_component_location_offsets` are parallel cache views after a rebuild. Cached owners should call this helper rather than indexing those parallel cache vectors directly in get/contains paths.

Cached component-location paths fail closed when that parallel-vector invariant is broken. Iterators and cache-slot helpers now require `cached_component_location_offsets[index]` and `cached_component_location_offsets[index + 1]` to resolve a valid slice into the flat buffer instead of substituting an empty slice, so zero-component query data such as `EntityId`, `StableEntityLocation`, or `()` cannot accidentally yield from an inconsistent cache snapshot.

The component-location cache is flat by design. Earlier revisions retained one `Vec<ComponentStorageLocation>` per matched entity; the current cache stores all component locations in one buffer and uses per-entity offsets to recover slices. `QueryState::update_cache(...)` also reuses one scratch `Vec<ComponentStorageLocation>` while asking `World` for the current entity's component locations, so cache rebuilds avoid both retained per-entity Vecs and repeated scratch allocation. The rebuild path now asks `World` for the matched archetype list, derives an exact reserve bound through `matching_query_archetype_entity_count(...)`, and fills `QueryState` directly through `visit_entity_locations_matching_archetypes(...)`; it no longer receives a temporary `Vec<StableEntityLocation>` candidate buffer before building the real cache. The flat component-location reserve remains bounded by candidate count times the access read count.

The ECS query behavior tests now recover per-entity component-location slices through the same offset invariant instead of indexing the old nested Vec shape. That keeps tests aligned with the hot-path representation and prevents a compatibility accessor from reintroducing retained per-entity allocations just to preserve old assertions.

## Direct Item-Fetch Error Projection

Cached direct, read-only cached, read-only, and mutable query-state entry points all validate entity existence and filter membership before fetching `QueryData`. The M5 direct-branch pass keeps that behavior while making the final missing-item projection explicit: each targeted fetch path now uses `let Some(item) = ... else` and returns `QueryDoesNotMatch` or `NoEntities` from the branch that actually observed the missing data.

This is intentionally a control-flow cleanup rather than a semantic change. `QueryState` still treats cache membership as the structural authority for cached paths, `fetch_cached(...)` and `fetch_with_component_locations(...)` remain stale-location guards, and mutable `single_mut` still reports `NoEntities` when the matched entity can no longer produce a mutable item after validation. The source guard lives in `ecs_query_state_structure.rs` so the large legacy `ecs_query_structure.rs` file does not keep absorbing unrelated shape checks.

## Direct Cached Many Request Streams

The direct cached-many path also keeps the caller-provided entity stream inside `CachedQueryManyIter`. It no longer allocates a temporary `Vec<usize>` through an index-collection helper before iteration. On each requested entity, the iterator uses the borrowed `cached_entity_indices` binary lookup to recover the cache slot, then fetches from `cached_entities`, `cached_locations`, and the flat component-location slice resolved from `cached_component_locations` plus `cached_component_location_offsets`.

This preserves request order and duplicate-request semantics while keeping direct cached data access on the component-location path. The public `Query` helper returns `CachedQueryManyIter<'_, '_, D, F, EntityList::IntoIter>`, so the caller iterator type is retained instead of erased behind an owned index vector.

`CachedQueryData` no longer exposes a separate `matches_cached_data(...)` hook. `QueryState::update_cache` is the structural membership authority: it has already matched the access descriptor, archetype filters, and `D::matches_component_locations(...)` before an entity reaches a direct cached iterator or direct cached get path. Direct cached iteration, contains, and get paths therefore run only `F::matches_cached(...)` for the current tick window before `fetch_cached(...)`. If a component location is ever stale despite the cache invariant, `fetch_cached(...)` still returns `None` and the get path reports `QueryDoesNotMatch`.

## Cached Combination Candidate Filtering

Cached read-only and mutable combination queries now use cache-aware constructors on `QueryCombinationIter` and `QueryCombinationMutIter`. `QueryState::update_cache` already rebuilds `cached_entities` by checking the access descriptor's matched archetypes and `D::matches_component_locations`, so cached combination construction does not repeat `D::matches_data` for every candidate entity.

The cached constructors still run `F::matches_component_locations(...)` while collecting combination candidates. This is intentional: structural filters such as `With` and `Without` are already represented in the access descriptor, but tick-sensitive filters such as `Added` and `Changed` remain dynamic for each query window. The result follows Bevy's query-state split: state-owned membership is reused as the structural authority, while the iterator keeps the current filter window local without returning to world-level filter lookups.

Read-only cached combinations also fetch through `D::fetch_with_component_locations(...)` from the same cache-slot slices used by cached read-only iteration. The iterator no longer copies matched entities, stable locations, or component-location buffers after filtering. Instead, `QueryCombinationCandidates::Cached` borrows `QueryState.cached_entities`, `cached_locations`, `cached_component_locations`, and `cached_component_location_offsets`, while storing only a `Vec<usize>` of matched cache slots for the current tick window. Combination indices address that compact cache-slot list; fetch resolves the original cache slot and then uses `cached_query_component_locations(...)` against the borrowed flat buffer.

Mutable combinations use the same compact cache-slot rule as their only construction path. `QueryCombinationMutCandidates` borrows `QueryState.cached_entities` and stores only matched cache-slot indices after `F::matches_component_locations(...)`; combination enumeration resolves each compact candidate back to an entity id only when `fetch_next(...)` builds the current tuple. The old uncached full-world constructor and `Owned(Vec<EntityId>)` branch were removed once `QueryState::iter_combinations_mut_with_ticks(...)` became the authoritative entry point. Mutable fetch still uses `D::fetch_mut_with_ticks(...)`, because the returned items are mutable and the iterator's distinct combination indices are the aliasing proof.

This changes cached combination lifetime semantics to match other cached iterators: `iter_combinations_cached(...)` and `iter_combinations_mut(...)` borrow the refreshed `QueryState` cache for the iterator lifetime. The uncached read-only combination path still owns its filtered entity list through `QueryCombinationCandidates::Owned`, so it remains independent of QueryState cache storage. Mutable combinations no longer keep a separate uncached owned branch, because the public mutable entry points already route through `QueryState` cache refresh and alias validation.

The uncached read-only combination path still uses the original full-world validation and keeps both `D::matches_data` and `F::matches`, because it is not backed by a refreshed `QueryState.cached_entities` membership list. That owned path now receives the world entity-id slice directly, counts matching entities with the same predicate used for insertion, allocates `QueryCombinationCandidates::Owned` through `Vec::with_capacity(candidate_count)`, and then pushes each matching entity id. The iterator still owns the filtered entity list so callers remain independent of `QueryState` cache storage, but it no longer relies on iterator `collect()` growth behavior. Mutable combinations intentionally have no uncached owned branch after the cache-authority cutover.

## Cache Component Location Inputs

`QueryAccess::add_write` mirrors writable component IDs into both `writes` and `reads`. The cache rebuild path relies on that invariant and passes `access.reads()` directly to `World::component_storage_locations_for_internal(...)`, which fills a caller-provided scratch vector. That keeps the component-location list stable for read, mutable, and change-filtered query data without rebuilding a temporary `reads + writes` vector or retaining one component-location Vec per cached entity on every world revision.

`QueryAccess::insert_id(...)` keeps component IDs sorted by inserting new IDs at the `binary_search(...)` miss position instead of pushing and re-sorting the whole access list. The cached component-location slices produced from `access.reads()` are sorted by `ComponentId` as well. `query_data.rs`, `query_filter.rs`, and `cached_query_iter.rs` use `binary_search_by_key(...)` for component-location lookup instead of linear `iter().find(...)` scans. This keeps tuple query data and `Added`/`Changed` filters from repeatedly scanning the same per-entity location slice on cached paths.

Future access changes must preserve these invariants or update `QueryState::update_cache` and the structure guard together. A write-only component that appears in `writes` but not `reads` would lose its cached storage location on direct cached fetch paths. A location list that is no longer sorted by `ComponentId` would break binary component-location lookup.

## Runtime 07 Cache Telemetry

`QueryStateCacheStats` exposes the local query-cache counters needed by Runtime 07 and can project them into the existing runtime `DiagnosticStore`. The snapshot includes cache hits, misses, rebuilds, current cached revision, cached archetype/entity counts, and the last rebuild's candidate/matched entity counts.

The counters are updated only inside `QueryState::update_cache(...)`, so they observe the same structural revision boundary as the cache itself. A same-revision refresh increments `cache_hits` without rebuilding; a changed revision increments `cache_misses` and `cache_rebuilds`, then records the candidate entity reserve bound and matched entity count. `record_diagnostics(...)` writes the counter paths `ecs.query.archetype_cache_hits`, `ecs.query.archetype_cache_misses`, `ecs.query.archetype_cache_rebuilds`, `ecs.query.candidate_entities`, and `ecs.query.matched_entities` with the shared `ecs`/`query` tags. The behavior test `query_state_cache_stats_record_reuse_and_rebuild_counts` locks that unchanged frames reuse the cache, structural spawn invalidates it exactly once, and the diagnostic snapshot exposes the expected counter values.

`query_state_reuses_archetype_matches_across_unchanged_frames` is the Runtime 07 M1.2 named assertion for the same invariant. It repeats cache refreshes across unchanged world revisions and asserts that hits advance while misses, rebuilds, cached revision, and candidate/matched entity counts stay stable. This test is source and rustfmt verified, but its Cargo run is still pending behind the active render-owned HZB compile blocker.

## Query Access Conflict Checks

`QueryAccess::conflicts_with(...)` is the boolean scheduling/access compatibility check. It now follows the same split as Bevy's access model: the boolean compatibility path checks read/write intersections directly, while the detailed conflict-list path remains in `conflicting_components_with(...)`.

This avoids allocating a temporary `Vec<ComponentId>` when scheduling code only needs a yes/no answer. The detailed list remains available for diagnostics and tests that need to report the conflicting component IDs.

## Boundary Rules

Do not recreate `query_state.rs`.

New query-state APIs should land by behavior family:

- `QueryState` declaration, construction, access descriptors, and retained cache fields in `mod.rs`;
- cache rebuilds, cache-slot resolution, and cached metadata accessors in `cache.rs`;
- cache telemetry snapshots and change-detection telemetry accumulation in `stats.rs`;
- direct cached storage access in `cached_direct.rs`;
- uncached read-only entity access in `read_only.rs`;
- cached read-only entity access in `read_only_cached.rs`;
- mutable entity access and alias validation in `mutable.rs`;
- reusable fixed-size collection helpers in `helpers.rs`;
- scheduler/system parameter wiring in `system_param.rs`.

The structure guard in `scene::tests::ecs_query_structure` rejects a legacy `query_state.rs`, missing owner files, behavior impl families in the root file, and owner files above the current budget. The structural audit mirrors that contract as `ecs_query_state_boundary` with nine folder-backed owner modules, including the `cache.rs` cache behavior owner and the `stats.rs` telemetry sidecar, so CI or review automation can detect the same rollback without first running the Rust test binary. The 2026-06-21 renderer split keeps `ecs_query_state_boundary.py` as the 141-line audit/risk owner and moves `render_ecs_query_state_boundary_markdown(...)` into the 33-line `ecs_query_state_markdown.py` renderer owner. This keeps future ECS query/cache work from turning `QueryState` back into a mixed hot-path file or turning the audit owner back into a report-formatting module.

The 2026-06-17 editor UI validation pass exposed a compile-only owner-boundary drift in `query_state/mod.rs`: the root `QueryState` struct still owns `cached_entities` and `cached_entity_indices`, so the root module must import `EntityId` even though cache rebuild/accessor behavior lives in `query_state/cache.rs`. Restoring that import preserves the existing behavior split and keeps `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` from failing before editor UI changes compile.

## Validation Notes

This is a structural and hot-path refactor. It should preserve query behavior and public export shape, then reduce the `runtime-other` large-file and ECS query/cache pressure reported by the runtime architecture audit. Focused validation should cover the structure guard, representative many-query behavior tests, rustfmt, static source guards for the borrowed cache-index, direct cached-many request-stream, clone-free `for_each_mut`, allocation-free access-conflict boolean contracts, the Runtime 07 diagnostic-store projection, and the runtime structure audit before broad runtime validation.
