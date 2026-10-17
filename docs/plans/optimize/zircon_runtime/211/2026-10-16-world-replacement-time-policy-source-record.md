# Runtime211 world replacement time policy: source record

The actual checkout now has explicit Preserve, Reset and Restore for whole-World replacement. Legacy replacement keeps its existing Preserve behavior. The WorldDriver frame-begin path takes the time snapshot and schedule owner from the same World/time pair.

Fixed V2 patch: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime211-world-replacement-time-policy-v2.patch (71912 bytes, SHA256 e9131aafc2e206e13dcdd856c2d565126c7d1971d85984a66db9636d7fda17db).

Fixed V2 binding: E:\Git\ZirconEngine\.codex\outbox\reports\optimize-20261013-01a13451\runtime211-world-replacement-time-policy-v2-source-binding.json (93810 bytes, SHA256 88cfc910304c8b43996e1bb2a80136b1f694e449ee8620a35d341d6d32ce3c15).

The actual source process exited 0 (Py28836/PS35656; terminal e6536d). All eight Git children exited 0 and were joined. Ten after files exactly match V2, with 10-section/21-hunk forward/reverse raw proof. HEAD and index stayed unchanged; 12 read-only witnesses stayed at their bound pins. The earlier parser failure exited 1 before any source write and remains recorded.

Preserve retains elapsed time, fixed debt, pause/rate and local clock state while clearing the old transient interpolation/single-step state. Reset validates its explicit policy and starts local counters/debt at zero. Restore consumes the actual retired World/time token of this Level; it restores local counters, debt and configuration while keeping the live outer watermark/source generation and advancing checked epochs. Requests are returned on explicit validation/identity/quiet/overflow rejection. Legacy active-step settlement and saturating World generation are unchanged; its preexisting replacement-epoch MAX panic remains a limit.

Only the unique world_time export hunk was moved onto the current LF scene module. Its other foreign bytes were preserved. The unavailable complete V1 scene-before semantic delta and foreign actors remain UNKNOWN. V1 packets remain immutable.

## Actual source

- zircon_runtime/src/core/framework/time/clock.rs: 11493 bytes, SHA256 aecab264811fba85d5683f1f83c7afe08b7ecf150f1d959b4f972aaaa70e1b7c.
- zircon_runtime/src/core/framework/time/domain/stamp.rs: 3670 bytes, SHA256 fe37e0f0a4a5ca4f6da70617eebad9a8a120e494c5467845320d60bb7da947a1.
- zircon_runtime/src/scene/level_system.rs: 35732 bytes, SHA256 d0123af65336d8346264133e9a1510e02413398f7bceb354f2c152c9bb6e2a17.
- zircon_runtime/src/scene/world_time/controller.rs: 29957 bytes, SHA256 b8353b9a31cc5eac28fe71ae7d95f2f3797f20a2e660b197be953cffc0154a72.
- zircon_runtime/src/scene/world_time/mod.rs: 1388 bytes, SHA256 f293ad7181032c6ffde4782d8f88e0c31f8bc6ae89c368e4ba7f95d6504417df.
- zircon_runtime/src/scene/mod.rs: 8465 bytes, SHA256 0acca5968797c672d7062ae88d7a3381c75d7a9101ecd96849638951f2b68dd5.
- zircon_runtime/src/scene/module/world_driver.rs: 13928 bytes, SHA256 a073e04b12d09caad923ddaab03a8b7d3f4f761d7d5986fbbf778112f734f334.
- zircon_runtime/src/scene/world_time/replacement.rs: 10181 bytes, SHA256 350ab16f7482fe779296d98e067e44619e03a9caf758ba463c14610af42bee42.
- zircon_runtime/src/scene/level_system/tests/world_time_replacement.rs: 26340 bytes, SHA256 236669eea9f42f0617d84ce09b9282ad5a246c885714e7d1760c4a9d5a38fa2a.
- zircon_runtime/src/scene/world/generation.rs: 10695 bytes, SHA256 ead69f266dc29727d6daec870a58fede85df774fab16efd553c510d6cde31c71.

## Tests and route

New ordinary identities (source added; none executed):

- `scene::world_time::controller::replacement_policy_tests::replacement_epoch_exhaustion_rejects_without_mutating_clock_or_policy` - UNRUN.
- `scene::world_time::controller::replacement_policy_tests::restore_rebases_checkpoint_epochs_without_rewinding_live_outer_source` - UNRUN.
- `scene::level_system::world_time_replacement_tests::legacy_preserve_retains_time_debt_and_the_old_active_step_settlement_path` - UNRUN.
- `scene::level_system::world_time_replacement_tests::explicit_preserve_retains_the_actual_retired_world_and_full_time_state` - UNRUN.
- `scene::level_system::world_time_replacement_tests::reset_and_restore_rebase_domains_without_rewinding_the_live_outer_frame` - UNRUN.
- `scene::level_system::world_time_replacement_tests::rejected_owner_stale_and_invalid_requests_are_failure_atomic_and_retryable` - UNRUN.
- `scene::level_system::world_time_replacement_tests::quiet_rejections_preserve_the_request_and_original_fixed_step_transactions` - UNRUN.
- `scene::level_system::world_time_replacement_tests::epoch_and_incoming_generation_overflow_return_the_original_request` - UNRUN.
- `scene::level_system::world_time_replacement_tests::frame_begin_keeps_time_and_schedule_owner_paired_across_a_real_reset` - UNRUN.
- `scene::world::generation::replacement_generation_tests::checked_replacement_revision_counts_pending_once_and_rejects_exhaustion` - UNRUN.

Preserved ordinary identities (none executed here):

- `scene::world_time::controller::tests::worlds_keep_pause_scale_and_fixed_debt_independent` - UNRUN.
- `scene::world_time::controller::tests::policy_transactions_advance_only_the_changed_clock_domain_epoch` - UNRUN.
- `scene::world_time::controller::tests::fixed_steps_stay_uncommitted_until_each_explicit_commit` - UNRUN.
- `scene::world_time::controller::tests::aborting_a_fixed_step_preserves_debt_and_does_not_skip_its_tick_identity` - UNRUN.
- `scene::world_time::controller::tests::time_policy_changes_reject_while_a_fixed_step_is_active` - UNRUN.
- `scene::world_time::controller::tests::fixed_timestep_policy_changes_reject_while_fixed_debt_is_pending` - UNRUN.
- `scene::world_time::controller::tests::fixed_interpolation_preserves_committed_endpoints_when_elapsed_saturates` - UNRUN.
- `scene::world_time::controller::tests::fixed_interpolation_observes_only_committed_steps_and_actual_remaining_debt` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::worlds_derive_virtual_and_fixed_time_independently_from_one_outer_frame` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::paused_world_single_step_advances_one_fixed_tick_then_stays_paused` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::paused_single_step_consumes_only_one_preexisting_debt_step` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::single_step_rejects_unpaused_and_duplicate_requests` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_replacement_cancels_unconsumed_single_step` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::failed_single_step_retries_without_duplicating_fixed_debt` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::rejected_outer_frame_does_not_consume_pending_single_step` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_fixed_debt_uses_the_outer_budget_not_another_clock_step_count` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_rejects_duplicate_outer_frame_before_mutating_time` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_rejects_decreasing_outer_frame_before_mutating_time` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_rejects_skipped_outer_frame_without_discontinuity` - UNRUN.
- `scene::tests::ecs_schedule::world_time_controller::world_accepts_skipped_outer_frames_with_explicit_discontinuity` - UNRUN.
- `scene::world::generation::tests::world_generation_advances_only_for_successful_structural_mutations` - UNRUN.
- `scene::world::generation::tests::explicit_entity_spawn_advances_generation_exactly_once` - UNRUN.
- `scene::world::generation::tests::imported_node_record_spawn_advances_generation_exactly_once` - UNRUN.
- `scene::world::generation::tests::fixed_presence_rebuild_marks_derived_state_before_lifecycle_callbacks` - UNRUN.
- `scene::world::generation::tests::rejected_imported_node_record_does_not_mutate_or_advance_generation` - UNRUN.
- `scene::world::generation::tests::exhausted_imported_entity_id_is_rejected_before_world_mutation` - UNRUN.
- `scene::world::generation::tests::rejected_imported_node_record_batch_is_atomic` - UNRUN.
- `scene::world::generation::tests::component_replacement_advances_the_query_generation_revision` - UNRUN.
- `scene::world::generation::tests::failed_mutable_component_lookup_does_not_advance_generation` - UNRUN.
- `scene::world::generation::tests::world_generation_is_runtime_state_not_persisted_scene_data` - UNRUN.
- `scene::world::generation::tests::lifecycle_visibility_revision_is_runtime_state_rebuilt_once_per_world_reconstruction` - UNRUN.
- `scene::tests::level_system_frame_state::level_world_replacement_and_pose_publication_share_an_epoch_commit_order` - UNRUN.

Preserved ignored producer:

- `scene::tests::ecs_schedule::world_time_controller::runtime22_performance_outer_frame_snapshot_projection_profile` - UNRUN.

Reuse inherited command #2: zircon_runtime debug/lib, features target-editor-host,test-support, OR-add the exact selectors and independently AND the 42 literal names from /tests/proposed_routing. Retain old floor172. The one animation-conditioned old guard remains conditional on the real resolved features. Do not move the ignored producer to debug #2; retain the inherited Release/ignored route and all original performance criteria. No new Cargo arm/global filter is introduced.

## Completion limits

- [x] Apply fixed V2 to the registered ten actual source files.
- [x] Complete raw forward/reverse, identity, index preservation and scoped static peer evidence.
- [ ] Execute new and preserved ordinary tests.
- [ ] Execute the preserved ignored producer and inherited performance gates.
- [ ] Run supported whole change guard and obtain owner full compiler/config/environment pre/post seals.
- [ ] Complete full TIME-P1-019/G27, staged/archive/product acceptance.

- Supported Python>=3.11 whole change guard: OPEN_NOT_RUN.
- Native/Cargo/compiler/resolved features/config/toolchain/environment full pre/post seal: OPEN_NOT_RUN.
- All 10 new ordinary and 32 preserved ordinary plus 1 preserved ignored producer: UNRUN_THIS_SLICE.
- Old conditional animation source guard executes only when actual sealed features enable it; lexical inventory is not a native floor.
- All inherited 31 Cargo+2 Python commands, original floor172, old names/producers/performance/RSS/quantile gates remain independent AND obligations.
- Full TIME-P1-019/G27, staged scene/archive checkpoint policy, product-level restore and old clock-stamped downstream event filtering: OPEN.
- Foreign scene/module/lib attribution and unavailable complete V1 scene-before semantic delta: UNKNOWN; preservation is not semantic/native acceptance.
- Legacy saturating World generation remains; existing replacement-epoch MAX panic is a prior limit, not a new typed failure-atomic guarantee.
