---
title: Neural ONNX Editor Dist Current-Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/neural/editor/src
  - zircon_plugins/neural/dist/src
canonical_owners:
  - docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
status: static_complete_dynamic_pending
references:
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Public/NNERuntime.h
  - dev/UnrealEngine/Engine/Source/Runtime/NNE/Private/NNEModelData.cpp
  - dev/UnrealEngine/Engine/Source/Editor/NNEEditor/Private/NNEEditorModelDataFactory.cpp
  - dev/UnrealEngine/Engine/Source/Editor/NNEEditor/Private/NNEEditorOnnxFileLoaderHelper.cpp
---

# Neural ONNX Editor Dist Current-Source Performance Review

## 1. Coverage and product truth

Editor production scope is **8/8 Rust files**, **2,067 physical / 1,934 non-empty lines**, **69,108 bytes**, fingerprint `948d24490003c5730c3c2077ddbb1081c68b93fa17ecb7f346794cee29192f03`. Runtime/feature/Dist assembly is **7/7 production files**, **431 / 389 lines**, **15,483 bytes**, fingerprint `101fb2c137f612da6b4907c323caeda23577ad5676362cc0fcca489c2de5facb`.

The Editor plugin has a real operation factory and path-authority checks, and it can convert ONNX into `.znn`. The chain stops after file production: no package-external production consumer loads or executes the asset. Dist publishes a stateless manifest with no commands, events or bridge and explicitly says compute remains in the runtime module whose registration is empty.

## 2. Structural performance and integrity findings

### P0: the hand-written ONNX reader is unbudgeted and duplicates large payloads

`editor/src/onnx/reader.rs` has no file, field, string, node, tensor, dimension, attribute, raw-data, cumulative allocation or parse-time budget. Tensor raw bytes are copied to a `Vec<u8>` and then decoded into a second `Vec<f32>` (`:149-189`). Signed dimensions are read as varints and cast with `as u32` (`:157`, `:248`, `:263`), and names use lossy UTF-8 (`:280-282`). Opset/domain and external tensor data are not part of admission.

The converter then creates additional tree maps, cloned names, model vectors and a serialized output. Large imports can simultaneously retain source bytes, protobuf DTOs, decoded F32 weights, `.znn` weights, serialized output and undo bytes. This is the dominant memory algorithm; optimizing map lookup first would not change peak scale.

### P0: import/undo performs synchronous whole-file work and direct replacement

`editor/src/plugin.rs:291-349` reads the complete ONNX, parses, validates, converts and serializes inside `EditCommand::apply`. It reads the complete previous output into command history and directly calls `fs::write`; undo directly overwrites or removes the file. There is no background-job admission, byte/time budget, cancellation, progress, staging, digest validation, atomic replace, crash recovery or asset-index generation receipt.

This can stall the Editor owner thread and can destroy the last-good asset on partial writes. Undo memory grows with output size. The CLI repeats whole-file read/direct-write behavior.

### P0: authoring success is disconnected from runtime readiness

Import success proves only that a `.znn` file was written. It does not prove a selected target runtime can create model data, that the artifact matches target/backend/build/device identity, or that the exact asset generation reached the runtime. UI availability must derive from loader/provider/product readiness, not catalog presence.

### P1: Dist is metadata-only and must remain explicit

The native Dist entry exposes no command/event schemas or invocation bridge. This is acceptable only as a metadata-only distribution shape. It must not imply native-dynamic inference parity with the static runtime until lifecycle and behavior tests execute the same model generation through both paths.

## 3. Unreal source constraints

- `NNERuntime.h:62-86` makes runtime capability, runtime-specific model creation and cache identity explicit instead of treating ONNX conversion as universal execution readiness.
- `NNEModelData.cpp:588-623` caches created runtime-specific shared data by runtime; `:633-669` checks the selected runtime before creation.
- Unreal's NNE Editor factory/ONNX helper routes source model data into NNE asset ownership, while runtime data creation stays provider and target qualified. Zircon should follow this separation and integrate its existing Editor job/import transaction and Runtime resource owners rather than reproducing Unreal's UObject machinery.

## 4. Dependency-ordered optimization plan

### M0: bound and make source parsing truthful

Use a maintained ONNX/protobuf implementation or wrap the current reader in strict aggregate budgets. Validate file/field/string/count/depth/dimension/element/raw/external-data totals before allocation; represent dynamic/signed dimensions explicitly; reject invalid UTF-8 identity and unsupported opset/domain semantics with field paths and offsets.

### M1: move import into a cancellable background transaction

Preflight source revision, target, output and budgets on the owner thread; read/parse/convert/serialize on Editor09 jobs with progress and cancellation; stage on the same volume, reopen and validate digest, flush and atomically publish; commit asset-index generation on the owner lane.

### M2: remove whole-artifact undo ownership

Store content-addressed previous-artifact identity or transaction receipt in command history. Undo/redo should swap validated generations, not keep arbitrary model bytes in memory or directly overwrite files.

### M3: cook per runtime/target

Preserve source identity and import settings, then derive provider/target/build/precision-specific artifacts with cache keys and last-good generations. Runtime admission must prove loader and provider readiness before Editor actions advertise executable success.

### M4: qualify Editor impact

Measure source bytes, raw weight bytes, nodes/tensors, peak RSS, allocation bytes, job queue/wait/run/publish time, UI owner-thread blocked time, cancellation latency, output bytes and fault recovery across small/medium/large/malformed models. Separate cold cache miss from warm cache hit.

## 5. Acceptance gates

1. Malformed or over-budget ONNX produces bounded typed failure without changing the output.
2. Import parsing/conversion/serialization does not run on the Editor UI owner lane.
3. Publication is atomic and crash/fault injection preserves the previous generation.
4. Undo memory is O(receipt/identity), not O(model bytes).
5. Import success names the source, target/provider and installed artifact generation.
6. Dist capability remains metadata-only until static/native behavioral parity is executed.

## 6. Validation status

- Static Editor and Dist production review: complete for the captured fingerprints.
- Direct source optimization: deferred; the fix crosses bounded parsing, background jobs, transactional publication and resource generation ownership.
- Cargo/tests and Editor current-source timing: pending because the managed validator/current-source executable are unavailable.
- Protected ledgers, milestone commit and WeCom completion remain pending.
