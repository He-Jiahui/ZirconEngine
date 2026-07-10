# Editor Architecture Plan 01 M1

## Scope

- Plan: `docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- Layers: L1 typed message bus, editor context, event journal/listener service, UI shell state, interim editing/play/viewport owners, and editor-manager construction.
- Hard cutover: remove `Empty/Text`, `EditorEventRuntime`, `EditorEventRuntimeState`, `lock_inner`, and migration-only aliases.

## Baseline

- `EditorMessagePayload` contained only `Empty` and `Text`.
- `EditorMessageBus::request` invoked handlers while the outer aggregate lock remained held.
- `EditorEventRuntimeState` mixed event, UI, operation, play, viewport, and message state behind one mutex.
- `zircon_editor/src/tests/editor_event/runtime.rs` exceeded the repository test-file hard budget.
- The worktree contains unrelated active-session changes; this milestone owns only Plan 01 editor paths recorded in the coordination note.

## Test Inventory

- Four typed message families through publish, request, and broadcast.
- Exact-topic routing and all-subscriber broadcast behavior.
- Request handler re-entry through the same shared bus.
- Request target removal during the unlocked handler window.
- Dirty-mask merging and refresh drain behavior.
- Event sequence, revision, journal, and listener equivalence after owner separation.
- Existing editor-event behavior suite after its owner-based file split.
- Structure guards: old symbols and constructors absent; core has no UI imports; root wiring files are structural.

## Boundary And Failure Cases

- Unknown subscriber before a request.
- Subscriber removed while a request handler executes.
- Poisoned message-bus mutex recovery.
- Empty dirty masks.
- Failed event execution still records the same revision and listener delivery semantics.
- Replay preserves success/failure behavior and sequence ordering.

## Tooling Evidence

- Scoped `rustfmt` and `git diff --check` passed for the M1 implementation paths on 2026-07-10.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-architecture-0710` passed in 4m00s.
- The first `cargo test -p zircon_editor --lib --locked --jobs 1` attempt stopped on another session's runtime-animation E0502. After that owner fixed it, the build reached editor tests and found two stale play-backend imports in the split `stack_play.rs`; both now target `core::play`.
- The next test attempt stopped before editor test execution on another active runtime-asset cutover's unresolved `crate::asset::stage_environment_ibl_source` import.
- Once external compile blockers settled, the Windows test binary ran 2891 tests. The first independent inspector failure was a stale node-vs-entity error-text assertion; the focused regression passes 1/1 after aligning with the typed `missing parent 999999` scene error and still verifies atomic rollback.
- The shared test-environment mutex now recovers its poisoned guard and clears poison while preserving the existing `LockResult` call contract. Its focused regression passes 1/1 on Windows and WSL, so a first panic no longer converts unrelated tests into lock-poison failures.
- A serialized Windows run executed 2897 tests and reported 2681 passed, 184 failed, and 32 ignored. The remaining failures were independently reproducible shared-worktree baselines, including the external runtime-text HUD glyph-capture regression (`changed_pixels=0`), UI Asset V2 fixture/write-path drift, and editor-layout/native-painter expectation drift owned by the active layout session.
- Test fixture materialization now projects legacy test DTO inputs into schema-2 `.zui` at the test boundary; production code receives no legacy parser alias. The first fresh binary `ef16e53d3ac23fda` passed the reference/local-component/Slot/named-mount V2 projection test 1/1. Its 44-test manager UI Asset filter reached 33 passed / 11 failed, isolating component assets that incorrectly retained view `[root]`, external promotion outputs serialized as legacy schema, and a save test that injected legacy source into a V2 session.
- The follow-up production slice routes external widget/theme promotion, host initial external writes, external-source undo/redo restoration, and canonical V2 saves through one `serialize_v2_projection_document` boundary. The serializer validates its own output with `UiZuiAssetLoader` before returning. Component `.zui` projections omit view roots, ZUI tests use the profile-validating loader, and the session save test edits the canonical V2 source. Two additional direct tests lock component-root-without-view-root behavior and rejection of component assets with multiple components. The 1278-line reference/promotion test owner was split into an 821-line parent and 457-line `theme.rs` child.
- Production import hydration no longer relies on `EditorTemplateRuntimeService`'s test-only legacy fallback. `.zui` references, including `#Component` fragments, normalize to the asset path and load only through `UiZuiAssetLoader`; host hydration/refresh atomically replace legacy authoring projections and untouched V2 widget/style prototype maps through `replace_resolved_imports`. Non-`.zui` assets remain on the distinct legacy loader path.
- Latest `cargo test -p zircon_editor --lib --no-run --locked` did not reach editor compilation: active shared runtime work leaves untracked `zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs:144` with E0716. No pass is claimed for the latest V2 slice until a current binary executes both projection tests and the 44-test filter.
- M1-focused binaries pass: editor message 9/9, editor event 85/85, and hard-cut owner guard 1/1.
- WSL `cargo test -p zircon_editor --lib --locked --jobs 1 --no-run` completed. The message suite passes 9/9, event suite 85/85, hard-cut guard 1/1, inspector atomic rollback 1/1, both material `TextureDimensionMismatch` projections 1/1, and environment-lock poison recovery 1/1.
- Production searches report zero `EditorMessage::text/empty`, `EditorEventRuntime`, and `lock_inner` matches. The removed aggregate owner files are absent.
- The event test split has a maximum file size of 729 lines; shared imports live in the parent modules, and the old 3590-line and 1169-line files are deleted.
- Linux/WSL CI-parity no-run and focused owner tests are recorded above; no WSL failure remains in the M1-owned scope.

## Results

- M1.1, M1.2, and M1.3 implementation and test code are present.
- Full Cargo acceptance remains open because the prior shared editor baseline has 184 independent failures and the latest current-source compile is externally blocked before editor code, even though all original M1-owned Windows/WSL focused gates pass.

## Acceptance Decision

- Open. The implementation slices, structure scans, and focused Windows/WSL gates are complete, but the strict milestone policy forbids promotion while the declared full-crate gate has 184 independent shared-worktree failures.
