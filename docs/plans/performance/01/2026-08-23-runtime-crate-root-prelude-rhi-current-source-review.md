---
title: Runtime Crate Root Prelude and RHI Current Source Review
date: 2026-08-23
scope:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/rhi.rs
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/RHI.Build.cs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/RenderCore.Build.cs
---

# Runtime Crate Root Prelude and RHI Current Source Review

## 1. Coverage

The current crate-root surface is **3/3 Rust files**, **161 physical / 151 non-empty lines**, **7,955 bytes**, and no test markers. Its workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `d7693a02765f6a7931fba6d5a381cecfa8e3e52d46ddbab5b4538e909b773ee4`. All three files, `zircon_runtime/Cargo.toml`, workspace consumers and the only product presenter factory caller were read directly. The three Rust files are clean in the shared worktree; the modified Runtime90 owner report is preserved and not claimed here.

## 2. Findings

1. None of the three files contains per-frame algorithms. `lib.rs` controls compile/module visibility, `prelude.rs` is compile-time re-export surface, and the only `rhi.rs` function constructs a boxed WGPU UI presenter during host presenter creation.
2. Graphics, Render Graph and the RHI facade are correctly gated by `graphics`; text/UI/dynamic API and other optional systems have explicit feature gates. Server workspace and App consumers use `default-features = false` before selecting `target-server`, so the current manifests do not accidentally union the default client graphics/window stack into that product.
3. Asset, scene, builtin, foundation, input, operation, platform and plugin modules remain unconditional crate modules. This is a compile time/binary-size and ownership question, not proof that they initialize at runtime. Any further gating requires profile reachability and binary/startup measurements; deleting module declarations by inspection would be unsafe.
4. The default runtime feature is `target-client`, while client and editor-host feature groups currently select nearly the same large subsystem set. The profile catalog must remain the runtime activation authority; Cargo feature presence alone must not be reported as a ready module.
5. `prelude.rs` is broad but has no runtime copy, lock or dispatch cost. Its risk is dependency/public-surface coupling and rebuild fan-out. Hot-path optimization belongs to the exported implementations, not to replacing these re-exports.
6. `create_default_ui_surface_presenter` directly names `zr_rhi_wgpu` and allocates a trait object. The call is not in a frame loop, so the box is not a performance bottleneck. The real issue is architectural: the editor GPU fallback uses it when a runtime-owned presenter is unavailable and may create an owned second Instance/Adapter/Device/Queue, bypassing the intended shared-device path. Runtime90 already owns that P0 and the hard cut to a production RHI device owner.

## 3. Unreal constraints

Unreal's `RHI.Build.cs` keeps the neutral RHI module separate from platform backends and dynamically selects Null, D3D11, D3D12, Vulkan or OpenGL by target; dedicated server skips every graphics backend except NullDrv. `RenderCore.Build.cs` depends publicly on RHI but keeps target-platform/editor/image tooling conditional or private. The relevant constraint is explicit target composition and one selected backend owner, not copying Unreal Build.cs or dynamic-module mechanics.

Zircon already meets the narrow server feature-isolation prerequisite. It does not yet meet the selected production backend/device ownership prerequisite because Runtime90 shows neutral `RenderDevice` has no production implementation and the editor fallback can create a second device.

## 4. Optimization plan

### M0: preserve current feature truth

Add static guards for server `default-features = false`, graphics-gated `rhi`/`render_graph`, and client/editor profile feature expectations. Keep Cargo feature, compiled module catalog, selected manifest and readiness report as separate facts.

### M1: measured target composition

For `target-server`, `target-client` and `target-editor-host`, record resolved features, compiled crate count, clean/incremental build duration, output size, startup wall time, loaded module count and idle RSS. Gate an unconditional module only when the product reachability and measured savings justify a hard cut.

### M2: production RHI owner

Runtime90 provides one device-generation owner and a production neutral RHI implementation. The editor obtains a shared presenter lease from that owner; remove the owned second-device fallback or make a separately measured recovery product with explicit device identity and lifetime.

### M3: narrow public surfaces

Keep `prelude` as convenience only. Internal modules import concrete owner paths, and public-surface changes use compile/consumer evidence. Measure compiler fan-out before splitting; do not claim runtime speedup from re-export churn.

## 5. Acceptance

1. Static feature matrix proves server has no `graphics`, `ui`, `text`, `winit`, `gilrs` or `zr_rhi_wgpu` activation and client/editor have exactly their declared product set.
2. Matched clean and incremental builds record duration, peak compiler RSS, artifact size and crate count for all three targets on the same toolchain/host. Runtime startup records wall time, loaded modules, threads and idle RSS.
3. Current-source editor GPU startup creates exactly one selected device generation in the normal path. Surface recreation does not create another adapter/device unless an explicit recovery generation is reported.
4. WPR/ETW measures startup CPU, module loads, thread creation, file I/O, RSS and energy. RenderDoc confirms the editor/runtime surfaces use the expected adapter/device and frame output; it cannot prove build-time or CPU-startup performance.

## 6. Current result

- Static current-source review is complete for **3/3** crate-root Rust files.
- No safe runtime optimization is present in these facade files; the second-device issue remains routed to Runtime90.
- No production or test source was changed by this slice.
- Cargo feature resolution, current-source product startup, WPR/ETW, GPU/device identity and RenderDoc remain dynamically pending.
