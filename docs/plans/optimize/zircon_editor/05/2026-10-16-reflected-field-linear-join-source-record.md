# Editor05 reflected-field pairing source record

Date: 2026-10-16. Owner: /root/runtime03_fence_core18.
Status: source implemented; scoped raw proof and independent core review complete.
Native, supported guard, measurements and product acceptance remain OPEN.

The inspector previously searched the returned reflection values once per visible
schema field. The production Scene-reading caller now uses a shared private
projection with a borrowed field-ID/value hash index. entry.or_insert preserves
the first returned duplicate. Schema order, hidden-field filtering, missing-value
skipping, existing conversion/drafts/read-only/editor behavior, and the original
stable name sort remain. Pairing is expected average O(R+F); the complete
projection still sorts in O(P log P). This closes only the field-pairing source
slice of E-INSP-P1-30.

## Exact source and evidence

- Builder: zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs.
  18511B/8237b3409913d051385e728cb5d7aea76705ed3f6d688bca16eeb476f826df96
  -> 19262B/78bdf3c24cf94f56e502999b4aefffd844d72c7469a7528dba3a3826ca09a131.
- New private child: zircon_editor/src/ui/workbench/snapshot/data/tests/reflected_field_join.rs.
  Absent -> 16996B/57b718b2be70851435ba30a124af6b848414f946b69cc34d48e6097fd3cf9ea8.
- Preserve-only public snapshot witness:
  zircon_editor/src/tests/editing/reflected_command/snapshot_projection.rs.
  Entire file remains 5551B/0808dbdeac0402298d8858140d2c1fca25c019f88a0ca6bbd6f3ac3d494e2eeb.

Fixed normal patch: editor05-reflected-field-linear-join-v1.patch,
20449B/c331e57091a7816690b7ac0d35d934531905a11f623a13f6637c1c148ee8d8e1.
Fixed binding: editor05-reflected-field-linear-join-v1-source-binding.json,
25902B/dc1653c50b416c208a3cd78306b8da87ea538e8451d99632c41cdcab8e4dc210.
Both are in .codex/outbox/reports/optimize-20261013-01a13451.
The BEFORE-source notice is
.jenkins/records/optimize-01a13451-20261016-editor05-source-activation.json,
6432B/ac9c5131bb66c7aed2a2c05b33d222df1e2bdd4662fe2e2defc00ae9aea38e26.

Source tool84/7f21ea actually exited0: Py7344/PS10248; Git9436,10268,28160,24604
all exited0 and were communicate/wait joined. Fresh before, new absence, both
packet pins and notice matched before writing only two files. After double raw,
5-hunk exact forward/reverse, unchanged witness, HEAD and index checks passed.
HEAD remains f28ba39a5095f4964c046ecc702778627f80cfd5; index remains
7478958B/8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893.
Independent Sol peer status is PASS_SCOPED_CORE_NOT_NATIVE, not compiler acceptance.

## Required execution identities: all UNRUN

New ordinary cases:

- ui::workbench::snapshot::data::editor_state_snapshot_build::reflected_field_join_tests::reflected_projection_joins_reordered_values_and_skips_missing_hidden_fields
- ui::workbench::snapshot::data::editor_state_snapshot_build::reflected_field_join_tests::reflected_projection_uses_first_duplicate_and_stable_name_ties
- ui::workbench::snapshot::data::editor_state_snapshot_build::reflected_field_join_tests::reflected_projection_preserves_drafts_read_only_and_value_formatting

Existing real Scene/public snapshot cases remain independent AND requirements:

- tests::editing::reflected_command::snapshot_projection::reflected_editor_command_snapshot_uses_reflection_schema_for_plugin_properties
- tests::editing::reflected_command::snapshot_projection::reflected_editor_command_snapshot_keeps_unloaded_dynamic_schema_protected
- tests::editing::reflected_command::snapshot_projection::reflected_editor_command_snapshot_marks_vector_and_entity_fields_editable

New ignored producer: ui::workbench::snapshot::data::editor_state_snapshot_build::reflected_field_join_tests::reflected_projection_linear_join_p95_1_100_10000

Existing command3 keeps zircon_editor/no-default-features/debug/lib and its old
167 subset independently; append the two test parent prefixes as OR selectors
and require each of the six ordinary names. Existing command12 keeps
zircon_editor/no-default-features/release/lib, all18 selectors, old12 subset and
--ignored --nocapture --test-threads=1; append the one producer independently.

## Additional unmeasured performance contract

Only the shared production projection (join + existing conversion + stable sort)
is compared with bench-only original repeated find. Input preparation, Scene
read/admission, world lock, full snapshot and GUI are excluded. Typed10k DTOs do
not prove a Scene admits10k fields. Both algorithms use the same black_box and
output lifetime boundary; correctness/checksum/drop are outside each timed call.

Scales1/100/10000 each have3 excluded warmup pairs and31 measured pairs,
alternating16 AB/15 BA. Emit both raw_ns values, order, seed, row count, complete
semantic equality/checksum and source/config identity for every pair. Each
algorithm/scale independently uses nearest-rank P95 at sorted index29 (30th/31).
Do not pool scales or compute P95 of paired ratios. At10000 only:
optimized_p95_ns*100 <= baseline_p95_ns*50 AND every paired output Eq.
Scales1/100 are report-only. New absolute and RSS budgets are null.
This engineering acceptance target is UNRUN and may not be relaxed after failure.

External ZIRCON_EDITOR05_SOURCE_IDENTITY must be64hex from the owner's actual
complete compiler-input source preseal canonical. ZIRCON_EDITOR05_CONFIG_IDENTITY
must identify its complete config/locks/features/toolchain/environment seal.
Missing values fail closed. Finite/virtual source hashes are not full seals.
Keep all original31Cargo+2Python arms,16 producers+legacy11 and all old
quantile/ratio/absolute/RSS/raw gates independently AND.

## Completion boundary

- [x] Apply the two frozen source changes; preserve the third file.
- [x] Exact raw/normal patch/source/index proof and independent core peer.
- [ ] Supported change guard and compiler/native ordinary execution.
- [ ] Ignored producer/raw evidence and the new relative P95 gate.
- [ ] Complete owner pre/post seals, real Scene10k admission and GUI evidence.
- [ ] Pane virtualization, full snapshot/frame budget and complete P1-30 acceptance.

No local build, native test, benchmark, guard, fixture or CI query was run.
No source copy/helper/temp was created, and no space release is claimed.
The packet-stage JS TextEncoder failure and the peer's actual1/null-name reader
are retained in the receipt. Earlier tool40 PID/session/terminal remain UNKNOWN;
later successful readers do not establish that all historical processes closed.

Post-output writer88 received a tool-level spawn_ready rejection; its PID, session
and terminal remain UNKNOWN. Bounded readback89/f8d338 actually exited0
(Py34260/PS33412, no children): four post targets absent, three frozen source hashes
and index unchanged. Missing outputs are created exclusively; no source is rewritten.
