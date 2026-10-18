# Runtime/Editor cdylib 插件化候选里程碑存档

## Summary

本存档记录当前 `zircon_runtime` 与 `zircon_editor` 仍可继续插件化、并在后续具备条件后抽成 native `cdylib` 的候选范围。当前结论是：第一个里程碑必须先完成 Native ABI v2 行为插件边界；在该边界验收前，不启动 VG/GI、Runtime UI、Importer、Editor Contribution 等迁移里程碑的行为外移实现。Native ABI v2 通过后，后续迁移才按 linked-plugin extraction、editor contribution、低风险 native pilot、高风险 rendering native pilot 的顺序推进。

当前 native dynamic 闭环已经覆盖 descriptor、manifest、diagnostics、load-manifest、export packaging 与 entry report，但它还不是完整的行为 ABI。真实行为现在仍更适合通过 SourceTemplate / LibraryEmbed linked Rust crate 激活；只有当 Native ABI v2 提供稳定 host function table、opaque handles、versioned DTO、serialized command/event payload、plugin-owned allocation/free、panic/unload boundary、state save/restore 与 capability negotiation 后，才适合把行为本体放进 `cdylib`。

## Architecture Decision

- `zircon_runtime` 最小本体继续保留 core lifecycle、plugin loader、project manifest/export profile contracts、asset/resource identity、scene/ECS authority、base render bridge、RHI/render graph contracts、input/platform baseline 与 host-owned GPU/window handles。
- `zircon_editor` 最小本体继续保留 workbench shell、Slint host、layout/view registry、plugin manager、capability gate、project/session startup 与最小 scene editing host。
- `cdylib` 插件不得跨 ABI 暴露 Rust trait object、`Arc<dyn Trait>`、borrowed references、Slint object、`wgpu::*` concrete objects 或 host-owned raw resources。
- 可抽取行为必须通过 descriptor、stable handle、serialized DTO、host command、event payload、asset bytes、manifest 或 capability contract 进入 host。
- Native ABI v2 是所有行为迁移的硬性 M0 gate：后续迁移可以继续使用 linked crate 作为实现包装形式，但它们必须面向同一个 ABI/DTO/handle/command contract 设计，不能先把 Rust 对象共享路径迁出去再补 ABI。
- `NativeDynamic` 在 M0 前只作为 packaging/discovery/diagnostic 后端；M0 后才允许作为 behavior plugin 后端承载真实调用。

## M0 Hard Gate Requirements

后续迁移里程碑开始前，Native ABI v2 必须同时满足以下要求：

- **stable host function table**：host 暴露一张版本化函数表，插件只能通过函数表调用 host 能力，不能链接或持有 host Rust 对象。
- **opaque host handles**：runtime/editor/asset/scene/render/window 等宿主对象只以不可解引用 handle id 暴露，插件不能看到内部结构。
- **versioned descriptor/DTO**：descriptor、capability、command、event、diagnostic、asset/result DTO 均带 ABI 版本或 schema 版本，host 能拒绝未知版本。
- **serialized command/event payload**：editor operation、runtime command、UI/event payload、asset importer result 通过可序列化 bytes/JSON/TOML/bincode-like DTO 传输，而不是 Rust enum/trait object。
- **plugin-owned allocation/free ABI**：所有跨边界返回的 plugin-owned memory 必须由 plugin 提供 free 函数释放；host-owned memory 不得由 plugin 释放。
- **panic/unload boundary**：entry、callback、command invocation、unload 均必须把 panic 或错误转换为 diagnostics/status，不允许 unwinding 穿过 FFI 边界。
- **state save/restore**：行为插件必须能在 unload/reload 前后导出与恢复版本化 state blob；不支持 state 的插件必须显式声明 stateless。
- **capability negotiation**：host 与 plugin 在注册时协商 runtime/editor/render/importer/UI/viewport/export capabilities，未授予 capability 的函数调用必须被 host 拒绝并记录 diagnostics。
- **no Rust trait/object crossing ABI**：禁止跨 ABI 传递 Rust trait object、`Arc`、`Box<dyn Trait>`、borrowed references、Slint object、`wgpu` object、raw host-owned OS/resource pointer。

M0 验收失败时，M1-M7 只能继续做文档、协议草图和 owner-boundary 清理，不能开始新的行为迁移实现。

## Source Context

- `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`
- `.codex/plans/zircon_plugins 全量插件化收敛规划.md`
- `.codex/plans/全系统重构方案.md`
- `zircon_plugins/**/{runtime,editor}/Cargo.toml`
- `zircon_runtime/src/builtin/runtime_modules.rs`
- `zircon_runtime/src/plugin/native_plugin_loader/**`
- `zircon_runtime/src/plugin/export_build_plan/**`
- `zircon_editor/src/ui/host/**`
- `zircon_editor/src/ui/workbench/**`
- `zircon_editor/src/scene/viewport/**`

## Current State Snapshot

- 已存在 runtime plugin crates：`physics`、`sound`、`texture`、`net`、`navigation`、`particles`、`animation`、`virtual_geometry`、`hybrid_gi`。
- 已存在 editor plugin crates：`physics`、`sound`、`texture`、`net`、`navigation`、`particles`、`animation`、`virtual_geometry`、`hybrid_gi`、`runtime_diagnostics`、`ui_asset_authoring`、`native_window_hosting`。
- 当前唯一明确 `cdylib` fixture 是 `zircon_plugins/native_dynamic_fixture/native`，用于 ABI/package/export/loader 闭环验证。
- `RuntimePluginId::Ui` 仍通过 runtime feature `plugin-ui` 从 `zircon_runtime::ui::UiModule` 激活，说明 UI runtime 实现仍有进一步外移空间。
- VG/GI 已经通过 plugin render descriptors 接入，但 heavy runtime state、GPU resources、prepare/update/feedback/readback state 仍在 `zircon_runtime::graphics` 内持续切分。
- Editor 插件 crate 已经能注册 view/menu/operation/template/drawer contribution，但真实 UI 深度、operation payload 与工具链边界仍可继续收敛。

## Candidate Inventory

### Runtime: Highest Priority

#### Virtual Geometry

**Target plugin:** `zircon_plugins/virtual_geometry/runtime`

**Current runtime ownership to extract:**
- `zircon_runtime/src/graphics/runtime/virtual_geometry/**`
- Virtual Geometry residency, page table, hot frontier, page request generation, prepare-frame assembly, pending completion, normalized page table handling, automatic extract, Nanite-like CPU reference.
- Scene renderer VG GPU resources, pass outputs, readbacks, indirect args, last-state storage, debug snapshot enrichment, and render-path summaries currently under `zircon_runtime::graphics::scene::scene_renderer`.

**Host boundary to keep in runtime:**
- Neutral render DTOs under `zircon_runtime::core::framework::render`.
- Render graph/RHI contracts.
- Host-owned `wgpu` resources and frame submission ownership.
- Render feature descriptor/capability registry.

**cdylib readiness:** medium after linked-crate migration. It can become `cdylib` only if GPU objects are host handles and plugin returns command/descriptor DTOs rather than owning `wgpu` resources directly.

#### Hybrid GI

**Target plugin:** `zircon_plugins/hybrid_gi/runtime`

**Current runtime ownership to extract:**
- `zircon_runtime/src/graphics/runtime/hybrid_gi/**`
- Probe cache, scene representation, surface cache, voxel scene, scene prepare resources, runtime feedback, pending completion, prepare-frame generation, GPU readback completion normalization.
- Scene renderer HGI GPU prepare/readback/post-process resource ownership currently under `zircon_runtime::graphics::scene::scene_renderer`.

**Host boundary to keep in runtime:**
- Neutral lighting/render extract DTOs.
- Frame history and visibility snapshot contracts.
- Host-owned texture/buffer/readback handles.
- Render feature descriptor/capability registry.

**cdylib readiness:** medium after linked-crate migration. It needs the same host-owned GPU handle and serialized command model as Virtual Geometry.

#### Runtime UI Implementation

**Target plugin:** future `zircon_plugins/ui/runtime`

**Current runtime ownership to extract:**
- `zircon_runtime/src/ui/template/**`
- `zircon_runtime/src/ui/layout/**`
- `zircon_runtime/src/ui/event_ui/**`
- non-minimal UI surface/render components that are not required for core host boot.

**Host boundary to keep in runtime:**
- UI capability id, UI document/template schema, stable UI node/template DTOs, base event bridge, host integration point.
- Minimal `plugin-ui` selection policy until `UiModule` is fully externalized.

**cdylib readiness:** high for template/layout/data-only pieces once DTO serialization is stable; medium for rendering/event behavior because it currently shares runtime Rust types.

### Runtime: Good cdylib Fit

#### Asset Importers and Codecs

**Target plugins:** future importer-specific runtime crates such as `zircon_plugins/gltf_importer/runtime`, `zircon_plugins/obj_importer/runtime`, or texture codec subplugins.

**Current runtime ownership to extract:**
- `zircon_runtime/src/asset/formats/**`
- importer ingest helpers that consume bytes/paths and return neutral asset records.
- UI template compiler pieces that can consume serialized template documents and return serialized template diagnostics/records.

**Host boundary to keep in runtime:**
- Asset manager, resource identity, project asset database, resource handles, dependency graph, cache ownership.

**cdylib readiness:** high. Importers naturally cross ABI using file paths, byte slices, manifest metadata, diagnostics, and serialized asset records.

#### Script / VM Backend Adapter

**Target plugin:** future `zircon_plugins/script_vm_backend/native` or VM backend package.

**Current runtime ownership to extract:**
- Concrete VM backend adapter once `zr_vm` integration begins.

**Host boundary to keep in runtime:**
- `VmBackend`, `VmPluginPackage`, `VmPluginInstance`, `VmStateBlob`, `HostHandle`, capability negotiation, hot reload coordinator.

**cdylib readiness:** high for backend adapters if VM host protocol is stable. This aligns with the long-term direction that gameplay plugins should prefer VM-level hot replacement over Rust dynamic behavior plugins.

#### Platform / Input Backend Adapters

**Target plugins:** future platform/input backend packages.

**Current runtime ownership to extract:**
- Specific window/input backend implementations once host-owned handle rules are explicit.

**Host boundary to keep in runtime:**
- Platform and input contracts, event normalization, raw handle ownership, main-loop ownership.

**cdylib readiness:** medium. This is feasible only with strict host-owned window/raw-handle policy and no borrowed platform objects across ABI.

## Editor Candidate Inventory

### Editor Views and Panes

**Target plugins:** existing editor plugin crates plus future domain-specific editor plugins.

**Current editor ownership to extract:**
- Domain panes currently wired under `zircon_editor/src/ui/host/builtin_views/**` when they are not required for minimal editor boot.
- Runtime diagnostics panes, module plugin panes, UI asset editor panes, component showcase panes, VG/HGI debug panes.

**Host boundary to keep in editor:**
- Workbench shell, Slint host, pane host, view registry, layout persistence, capability gating, project/session state.

**cdylib readiness:** high for descriptor/template-driven panes; medium for panes that need live Rust object access until command/event payloads are serialized.

### Menu, Operation, and Command Payloads

**Target plugins:** existing editor domain crates such as `physics/editor`, `animation/editor`, `sound/editor`, `virtual_geometry/editor`, `hybrid_gi/editor`.

**Current editor ownership to extract:**
- Domain menu actions and operation handlers under `zircon_editor/src/ui/host/editor_event_execution/**` when they belong to optional features.
- Domain-specific payload builders that can be expressed as serialized command payloads.

**Host boundary to keep in editor:**
- `EditorOperation` routing, undo/redo policy, host capability checks, project mutation authority.

**cdylib readiness:** high after operation payloads become versioned serialized DTOs and plugin handlers call host function table methods instead of directly mutating editor state.

### Component Drawers and Inspector Extensions

**Target plugins:** per-domain editor plugins.

**Current editor ownership to extract:**
- Physics collider drawers, sound emitter drawers, animation clip/state drawers, texture settings drawers, particle emitter drawers, navigation mesh drawers, VG/HGI debug drawers.

**Host boundary to keep in editor:**
- Inspector host, selection model, draft editing state, undo command generation, drawer registry.

**cdylib readiness:** high if drawer output is a template/document descriptor plus serialized field-change commands.

### Viewport Tools and Gizmos

**Target plugins:** per-domain editor plugins and future viewport-tool packages.

**Current editor ownership to extract:**
- Specialized tools under `zircon_editor/src/scene/viewport/handles/**`, `pointer/**`, and `interaction/**` that are not the minimal move/rotate/scale baseline.
- Animation rig tool, physics collider edit tool, navigation mesh edit tool, particle emitter gizmo, VG/HGI debug visualization tool.

**Host boundary to keep in editor:**
- Viewport controller, input routing, transform math contracts, selection state, render overlay host, basic transform handles.

**cdylib readiness:** medium. Picking and overlay geometry can cross ABI as DTOs; direct camera/controller mutation must stay host-mediated.

### Project Templates and UI Template Contributions

**Target plugins:** editor-only template packages and domain editor crates.

**Current editor ownership to extract:**
- Optional welcome/new-project templates, pane templates, drawer templates, activity drawer entries, component showcase documents.

**Host boundary to keep in editor:**
- Template registry, template runtime projection, Slint host, document id namespace, template validation.

**cdylib readiness:** very high. Most of this can also be data-only package content rather than behavior `cdylib`.

### Export Strategy UI and Build Helpers

**Target plugins:** editor export-helper plugins.

**Current editor ownership to extract:**
- Per-plugin package preparation diagnostics, native build helper views, domain-specific export UI.

**Host boundary to keep in editor/runtime:**
- Export profile model, SourceTemplate/LibraryEmbed/NativeDynamic strategy contracts, sandbox/path validation, build-plan authority.

**cdylib readiness:** medium. Build helpers can be native, but path validation and final plan authority must remain host-owned.

## Non-Candidates for cdylib Extraction

- Runtime core lifecycle, module activation, plugin loader, project manifest, export profile, and native load-manifest authority.
- Asset manager, resource identity, stable handles, dependency graph, and project resource database.
- Scene authoritative world/ECS, hierarchy, transform propagation, and render-extract authority.
- Base render framework, render graph, RHI contracts, renderer host, frame submission, swapchain/window ownership.
- Editor workbench shell, Slint host, layout/view registry, plugin manager, capability gate, startup/project/session state.
- Any surface requiring direct sharing of Rust trait objects, `Arc`, borrowed references, Slint objects, `wgpu` objects, or host-owned OS handles.

## Milestones

### M0: Native ABI v2 Contract Design

**Goal:** Turn NativeDynamic from report-only package discovery into a safe behavior-plugin boundary.

**Files to design or modify later:**
- `zircon_runtime/src/plugin/native_plugin_loader/**`
- `zircon_runtime/src/plugin/package_manifest/**`
- `zircon_runtime/src/plugin/runtime_plugin/**`
- `zircon_editor/src/ui/host/editor_manager_plugins_export/**`
- `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`

**Acceptance:**
- ABI exposes a stable host function table, opaque host handles, versioned descriptor/DTO surfaces, serialized command/event payloads, plugin-owned allocation/free functions, panic-safe entry/callback status, unload callback, state save/restore hooks, capability negotiation, and an explicit ban on Rust trait/object crossing.
- Runtime/editor native entry reports can register callable operations, not only static diagnostics.
- Fixture plugin proves version mismatch, missing symbol, entry failure, denied capability call, allocation/free mismatch diagnostics, panic-to-diagnostic conversion, state save/restore, stateless plugin declaration, and safe unload.
- M0 must pass before M1-M7 migration implementation begins.

### Migration Start Rule

M1-M7 are blocked until M0 is accepted. After M0, every migration task must first decide its ABI-facing contract, even when the concrete implementation remains a linked Rust crate during that milestone. This prevents a temporary linked-crate API from becoming a second behavior model that later conflicts with NativeDynamic.

### M1: Finish VG Linked-Crate Migration

**Goal:** Move Virtual Geometry heavy runtime state behind `zircon_plugins/virtual_geometry/runtime` while runtime keeps only neutral contracts and host-owned GPU handles.

**Files to design or modify later:**
- `zircon_plugins/virtual_geometry/runtime/src/**`
- `zircon_runtime/src/graphics/runtime/virtual_geometry/**`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/**`
- `zircon_runtime/src/graphics/scene/scene_renderer/**virtual_geometry**`
- `docs/assets-and-rendering/virtual-geometry-nanite-foundation.md`
- `docs/assets-and-rendering/render-framework-architecture.md`

**Acceptance:**
- VG residency, prepare, feedback, and runtime snapshot logic live in plugin runtime package or behind plugin-owned contracts.
- Runtime frame submission consumes plugin outputs through descriptor/DTO methods, not concrete VG state fields.
- Base render without VG descriptor does not initialize VG heavy state.

### M2: Finish HGI Linked-Crate Migration

**Goal:** Move Hybrid GI heavy runtime state behind `zircon_plugins/hybrid_gi/runtime` while runtime keeps neutral lighting/render contracts.

**Files to design or modify later:**
- `zircon_plugins/hybrid_gi/runtime/src/**`
- `zircon_runtime/src/graphics/runtime/hybrid_gi/**`
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/**`
- `zircon_runtime/src/graphics/scene/scene_renderer/**hybrid_gi**`
- `docs/assets-and-rendering/hybrid-gi-lumen-foundation.md` if present or the current HGI render docs that own this subsystem.
- `docs/assets-and-rendering/render-framework-architecture.md`

**Acceptance:**
- Probe cache, scene representation, feedback, pending completion, and prepare-frame state are plugin-owned or exposed only through plugin-owned contracts.
- Base render without HGI descriptor does not initialize HGI heavy state.
- HGI editor/debug contribution reads stable inspection DTOs, not runtime internal fields.

### M3: Externalize Runtime UI Implementation

**Goal:** Split optional UI runtime behavior into `zircon_plugins/ui/runtime`, leaving runtime core with stable UI contracts and host bridge.

**Files to design or modify later:**
- `zircon_runtime/src/ui/**`
- `zircon_runtime/src/builtin/runtime_modules.rs`
- future `zircon_plugins/ui/runtime/Cargo.toml`
- future `zircon_plugins/ui/runtime/src/lib.rs`
- `docs/ui-and-layout/**`

**Acceptance:**
- `RuntimePluginId::Ui` resolves from plugin registration instead of direct `ui::UiModule` construction when linked.
- `core-min` remains UI-free.
- UI templates/layout/event payloads cross plugin boundary through stable descriptors and serialized DTOs.

### M4: Editor Contribution Deepening

**Goal:** Make editor plugins own real views, menus, operations, drawers, templates, and viewport tool contributions instead of only capability labels.

**Files to design or modify later:**
- `zircon_plugins/*/editor/src/lib.rs`
- `zircon_plugins/editor_support/src/**`
- `zircon_editor/src/ui/host/editor_extension_registration.rs`
- `zircon_editor/src/ui/host/editor_operation_dispatch.rs`
- `zircon_editor/src/ui/workbench/view/**`
- `zircon_editor/src/ui/template_runtime/**`
- `zircon_editor/src/scene/viewport/**`

**Acceptance:**
- Domain editor plugins register concrete operation descriptors and serialized payload schemas.
- Plugin-provided panes/templates disappear from registry when plugin is disabled.
- Optional domain drawers/tools are no longer hardcoded in editor minimal host.

### M5: Asset Importer and Template Compiler Plugins

**Goal:** Move importer/codec/template compiler behavior that naturally crosses bytes/paths into plugin packages.

**Files to design or modify later:**
- `zircon_runtime/src/asset/formats/**`
- `zircon_runtime/src/asset/importer/**`
- `zircon_runtime/src/ui/template/asset/compiler/**`
- future importer plugin crates under `zircon_plugins/**/runtime`

**Acceptance:**
- Importer plugins register format ids, extensions, diagnostics, and byte-to-asset conversion callbacks.
- Asset manager owns handles and dependency graph; plugin owns decode/import behavior only.
- Importers can be packaged as NativeDynamic once ABI v2 byte-slice/result ownership is in place.

### M6: Native cdylib Behavior Pilot

**Goal:** Convert one low-risk plugin from linked crate only to real native behavior plugin.

**Preferred pilot:** asset importer or editor template contribution, not VG/HGI.

**Files to design or modify later:**
- `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`
- one new native pilot package under `zircon_plugins/**/native`
- `zircon_runtime/src/plugin/native_plugin_loader/**`
- `zircon_editor/src/ui/host/editor_manager_plugins_export/**`

**Acceptance:**
- Host loads native plugin, negotiates ABI, registers a callable behavior contribution, invokes it through function table, receives serialized result, and unloads without retaining plugin-owned references.
- Failure modes produce diagnostics instead of panics.

### M7: High-Risk Native Rendering Pilot

**Goal:** After linked VG/HGI migration and ABI v2 are stable, evaluate whether selected render-feature policy logic can run from native plugin while GPU resources remain host-owned.

**Files to design or modify later:**
- `zircon_plugins/virtual_geometry/runtime/**`
- `zircon_plugins/hybrid_gi/runtime/**`
- `zircon_runtime/src/graphics/runtime/render_framework/**`
- `zircon_runtime/src/graphics/scene/scene_renderer/**`

**Acceptance:**
- Plugin can contribute render feature descriptors, prepare/update commands, debug policy, and residency decisions through stable DTOs.
- Host keeps GPU allocation, command encoding, frame submission, history, and readback ownership.
- No `wgpu` object, borrowed host pointer, or Rust trait object crosses native ABI.

## Validation Gates

### Per-slice gates

- `cargo check -p zircon_runtime --lib --locked`
- `cargo check -p zircon_editor --lib --locked`
- targeted `cargo check` / `cargo test` for affected plugin crate under `zircon_plugins`
- docs update under `docs/` for every subsystem boundary changed

### Milestone gates

- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`
- NativeDynamic export plan tests covering SourceTemplate, LibraryEmbed, NativeDynamic, duplicate package output directory, manifest id/path mismatch, missing symbol, ABI mismatch, and safe diagnostics.

## Risk Register

- `cdylib` ABI cannot safely expose Rust trait objects or borrowed runtime/editor references; all native behavior must use opaque handles and serialized DTOs.
- Rendering plugins are high risk because GPU objects, readbacks, and command encoders must remain host-owned.
- Slint UI objects must remain editor-host-owned; plugins may contribute templates and commands, not Slint object references.
- Hot reload for native Rust `cdylib` is limited by platform loader semantics; VM plugin hot replacement remains the preferred long-term gameplay/user-code model.
- Keeping linked crate and native dynamic paths simultaneously can create compatibility drift; each feature needs one authoritative contract and two packaging backends, not two behavior models.

## Archived Recommendation

优先执行顺序固定为：先完成 M0 Native ABI v2 行为插件边界，再开始 VG/GI linked-crate heavy-state 收敛、Runtime UI 外移、editor contribution 深化和 importer/template 低风险 native pilot，最后才评估 VG/HGI 这类高风险渲染插件的 native policy/command 化。不要先把 `wgpu`、Slint、scene world 或 editor state 直接跨 `cdylib` 边界暴露出去，也不要在 M0 之前启动新的行为迁移里程碑实现。
