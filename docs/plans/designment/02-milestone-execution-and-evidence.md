---
related_code:
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime/src/asset/assets/ui/document_loader.rs
  - dev/penpot/plugins/libs/plugin-types/index.d.ts
  - dev/penpot/plugins/apps/table-plugin/src/plugin.ts
  - zircon_runtime_interface/src/ui/mod.rs
  - zircon_runtime_interface/src/ui/component/descriptor/slot_schema.rs
  - zircon_runtime_interface/src/ui/surface/render/batch/tests.rs
  - zircon_editor/src/core/gateway/mod.rs
  - zircon_editor/src/scene/selection/mod.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_editor/assets/ui/editor/layout/presets.toml
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/web/src/App.tsx
  - zircon_hub/package.json
  - tools/ui-profile-capture.ps1
  - tools/capture-editor-ui-visual.ps1
  - tools/ui-profile-scale-fixture.ps1
  - tools/performance-machine-manifest.ps1
  - tools/profile-capture-manifest.ps1
  - tools/ui-profile-scenarios.ps1
implementation_files:
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/zui-document.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-projection-model.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-projection.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-reconcile.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-asset.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/cli.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/plugin.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/app/app.component.ts
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/web/src/App.tsx
tests:
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/zui-document.spec.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-projection.spec.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-asset.spec.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui
  - zircon_runtime/tests/zui_penpot_bridge_contract.rs
  - zircon_runtime/tests/fixtures/ui/penpot_roundtrip.zui
  - zircon_editor/tests/integration_contracts.rs
  - zircon_editor/src/ui/layouts/views/asset_browser/tests/mod.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/web/tests/window_action_scheduler.test.mjs
  - tools/tests/ui-profile-capture-output-contract.Tests.ps1
  - tools/tests/capture-editor-ui-visual.Tests.ps1
  - tools/tests/ui-profile-scale-fixture.Tests.ps1
  - tools/tests/performance-machine-manifest.Tests.ps1
  - tools/tests/render-extract-performance-scenario.Tests.ps1
plan_sources:
  - "user: 2026-08-31 以 dev/penpot 作为界面设计参考，为 ZirconEngine 编写完整界面设计计划"
  - "user: 2026-08-31 先兼容当前 Penpot，实现 .zui 与 Penpot 资产互相转换，再推进 ZirconEngine 自举布局"
  - docs/plans/designment/01-penpot-inspired-interface-design.md
  - docs/plans/mvp/index.md
  - docs/plans/milestone-validation-policy.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_hub/01-action-dispatch-and-typed-payload.md
  - docs/plans/zircon_hub/02-background-task-framework-and-persistence.md
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
  - docs/plans/zircon_hub/06-layout-and-visual-standard.md
  - docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
doc_type: workflow-detail
status: in_progress
last_refined: 2026-08-31
---

# 界面设计里程碑执行与证据计划

本文件是 [01 主计划](./01-penpot-inspired-interface-design.md) 的执行 companion。01 是设计决策唯一事实源；本文件只规定 owner 路由、MVP 闸门、validation manifest、证据落点和可复现的验证方式。它不拥有 Runtime UI ABI、Editor SelectionModel、.zui catalog、Hub DTO 或业务实现。

## 1. 执行总则

### 1.1 闸门

| 闸门 | 含义 | 允许的界面工作 | 不能声称 |
|---|---|---|---|
| design-ready | schema、信息架构、fixture 和 owner 已评审 | M0、M1、直接支撑 F0-F4 的 M2/M3 契约、M6-PRE | 产品可用、F gate accepted、视觉 wave 通过 |
| f0..f4-accepted | MVP 对应 owner 计划已有当前源证据 | 下一层直接依赖的 UI/交互切片 | 上层高级能力已完成 |
| f5-accepted | MVP F0-F5 在 clean validation copy 上通过 | M4-M8 产品实现、完整 Hub polish、domain/plugin、视觉/性能 wave | 仍有未验收 failure 的发布候选 |
| accepted | 本里程碑证据齐全、owner review 完成 | 解锁表中指定的下一个里程碑 | 以窄测试或单张截图代表全量通过 |

docs/plans/mvp/index.md 的 blocked_by_* 状态优先于本文件。若 F gate 未满足，状态只能保持 blocked 或 design-ready，failure 必须回到原 owner 计划。

### 1.2 Penpot 到引擎的硬顺序

本次用户请求增加一个可逆的 authoring bootstrap 链。它与原有 M0-M9 产品里程碑相邻，但不改变 Runtime UI ABI 的 owner：`.zui` v2 仍是 ZirconEngine 的唯一运行时资产格式，Penpot 只提供一个 authoring adapter。三步必须按顺序完成，后一步不得用截图或手工复制替代前一步的 contract 证据。

| Bootstrap | 目标 | 入口/出口 | 必须保留 | 解锁 |
|---|---|---|---|---|
| A0 contract | 定义 `.zui` v2 <-> Penpot semantic bridge、metadata namespace、支持/警告/拒绝矩阵 | 真实 `.zui` fixture 可解析；未知字段和事件/绑定可证明保留 | 原始 document JSON、稳定 node id、imports/tokens/components/styles、unsupported diagnostics | A1 |
| A1 Penpot adapter | 在当前 Penpot 中导入 `.zui` 为可编辑 board，导出为可被 Zircon loader 接受的 `.zui` | plugin build/lint/unit tests + 一次真实 import/export round-trip | 语义层级、文本、几何、布局子集、raw metadata；辅助视觉 shape 不得成为语义节点 | A2 |
| A2 engine bootstrap | Zircon Editor/Runtime 消费同一 bridge contract，显示与 Penpot 导出的布局一致 | existing `.zui` loader/compiler + editor fixture evidence；无第二套 schema | `.zui` source of truth、版本拒绝、diagnostics、fallback layout | 后续 M0-M9 产品 UI 实现 |

#### A0/A1 capability matrix

| `.zui` surface | Penpot projection | Export policy | 状态 |
|---|---|---|---|
| node id, component, control id, children/slots | board/text shape metadata and hierarchy | preserve exact raw value; shape rename never changes id | lossless |
| x/y, width/height, padding, gap, direction, wrap, alignment | board geometry and flex layout | apply only changed numeric/layout fields | supported |
| text/placeholder and text style | text shape characters/font/alignment | export edited text/style; preserve other props | supported |
| fills, strokes, radius, opacity | Penpot fills/strokes/radius/opacity | export supported paint values; warn on unsupported paint syntax | supported-with-warning |
| events, bindings, repeat, params, slots, imports, tokens, style scopes | plugin/shared data JSON on asset board and semantic shapes | copy raw JSON unchanged unless an explicit editor maps a field | metadata-preserved |
| arbitrary plugin/runtime props and unknown tables | no visual projection | retain in raw document; emit diagnostic | preserved-but-not-editable |
| raster/vector assets, expressions, runtime-only controls | optional placeholder shape | never fabricate executable semantics; export original value | unsupported |

The adapter must report diagnostics with severity `info`, `warning`, or `error`. A warning is allowed for a successful export; an error blocks download and leaves the last valid document untouched. No field may be silently dropped.

### 1.3 证据目录

执行时在本目录创建以下目录和文件；规划阶段不预填结果：

    docs/plans/designment/
      evidence/
        a0-zui-penpot-contract.md
        a1-penpot-roundtrip.md
        a2-engine-bootstrap-parity.md
        m0-baseline-and-mapping.md
        m1-token-component-contract.md
        m2-shell-contract.md
        m3-viewport-inspector-contract.md
        m4-assets-library-tokens.md
        m5-feedback-task-recovery.md
        m6-hub-entry-contract.md
        m6-hub-product-surface.md
        m7-domain-extension.md
        m8-quality-performance-a11y.md
        m9-release-checklist.md
      manifests/
        a0-zui-penpot.yaml
        a1-penpot-roundtrip.yaml
        a2-engine-bootstrap.yaml
        m0-static.yaml
        m1-token-components.yaml
        m2-shell.yaml
        m3-mvp-viewport.yaml
        m4-assets-library.yaml
        m5-feedback-tasks.yaml
        m6-hub.yaml
        m7-domain-extension.yaml
        m8-quality.yaml
        m9-release.yaml

Evidence 文件必须记录 changed scope、manifest 路径、实际命令、profile/target-dir、结果摘要、修复的 failure、延后的外部检查和下一个解锁项。没有实际命令和结果的条目只能保持 planned/design-ready。

### 1.4 Evidence 文件模板

每个 evidence 文件使用以下字段：

    # Mx 证据
    - Gate: design-ready | f0-accepted | f5-accepted | accepted
    - Owner session(s):
    - Changed scope:
    - Manifest:
    - Commands actually run:
    - Result summary:
    - Repaired failures:
    - Deferred external checks:
    - Evidence links:
    - Unlocks:

主计划只汇总已验证结果，不能复制逐切片 changelog。

## 2. Owner 路由

| 设计主题 | 唯一 owner | 本 companion 可写内容 | 必须回传的证据 |
|---|---|---|---|
| token/style/cascade | editor_layout/01、20 | 跨产品语义映射、拒绝项、fixture 需求 | resolver/selector 测试、raw-value 扫描 |
| region/dock/preset/responsive | editor_layout/02、03、04、15e、16 | 页面区域和状态矩阵 | layout round-trip、断点/DPI 断言 |
| retained UI/assets/components | editor_ui/04、05、06 | 组件语义、slot/variant 需求 | .zui governance、catalog、component contract |
| Editor gateway | editor/01 | snapshot/handle/overlay 消费方式 | gateway contract、无深路径旁路 |
| Selection/scene/gizmo | editor/05 | 选择/视口流程 | SelectionModel、HighlightSet、mode/drag tests |
| command/transaction/undo | editor/03 | commit boundary、can-execute、undo label | journal/replay tests |
| Inspector/FieldEditor | editor/06 M2 + editor_ui/06 | 受控 binding、property row、typed command 边界 | binding/command、无直接 world mutation、component contract |
| assets/project references | editor/09、10 | library/token v1 设计子集 | registry、serialization、rollback tests |
| async/recovery/diagnostics | editor/14、17 | task/error/status 语义 | cancellation、error injection、recovery trace |
| Hub action/DTO/persistence | Hub 01、02 | 页面状态和 handshake | Rust contract、generation/race tests |
| Hub React/MUI | Hub 05、06、07 | 页面组件和视觉 fixture | npm run typecheck、npm run build、截图 |
| plugin/extension | `editor/06`、`12`；`zircon_runtime/runtime/06`；`zircon_plugins/01`、`10` | slot、permission、failure isolation 需求 | load/unload/migration/failure tests |
| `.zui` <-> Penpot authoring adapter | `zircon_runtime_interface`（格式 owner）+ `dev/penpot/plugins/apps/zircon-zui-plugin`（adapter owner） | bridge contract、projection/loss policy、import/export diagnostics | parser/serializer semantic round-trip、plugin projection tests、真实 Penpot asset round-trip |

跨越两行以上 owner 的切片，先建立 cross-plan handoff，再进入实现；本文件不能代替 owner 接口裁决。

## 3. Validation manifest

### 3.1 必填字段

每个 manifests/*.yaml 必须包含：

    milestone: Mx
    status: planned
    mvp_gate: "design-ready | f0-accepted | f5-accepted"
    owner_sessions: []
    changed_scope: []
    packages: []
    feature_profiles: []
    interface_boundaries: []
    target_dir: "<coordinator-assigned-approved-path>"
    cargo_profile: "development | release | profiling"
    storage_mode: "reuse | compact | diagnostic"
    commands: []
    focused_tests: []
    product_evidence: []
    failures_repaired: []
    deferred_external_checks: []
    unlocks: []

target_dir 必须替换为 coordinator 分配的 D:\cargo-targets、E:\cargo-targets、F:\cargo-targets、D:\targets、E:\targets、F:\targets、D:\ZirconBuilds、E:\ZirconBuilds 或 F:\ZirconBuilds 下的具体路径。不得在仓库生成 target/。

### 3.2 Windows 验证命令模板

以下命令只作为 manifest 模板；执行时必须写入实际 target、profile、过滤器和退出码：

    git diff --check -- docs/plans/designment
    .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package <package> -TargetDir <target-dir> -CargoProfile development -SkipTest
    .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package <package> -TargetDir <target-dir> -CargoProfile development -SkipBuild -LibTests -TestFilter <focused-filter>
    .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package <package> -TargetDir <target-dir> -CargoProfile development -SkipBuild -TestTarget <integration-target>
    .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime_interface -TargetDir <target-dir> -SkipBuild -LibTests -TestFilter <interface-filter>

`-LibTests`/`-TestFilter` 用于 crate lib test，`-TestTarget` 用于单个 integration target；validator 明确拒绝 `-LibTests` 与 `-TestTarget` 同传，manifest 必须把它们拆成独立命令。普通切片只做格式/结构/静态检查；Cargo、产品运行和截图集中在里程碑测试阶段。不要用未受管的 cargo test 绕过 validator，也不要为同一 package 启动第二条兼容构建线。

### 3.3 Hub Web 验证

Hub 的脚本定义在 zircon_hub/package.json：

    Push-Location zircon_hub
    npm run typecheck
    npm run build
    Pop-Location

manifest 必须记录 Node/npm 版本、依赖安装方式、命令退出码和输出文件位置。依赖不可用时写入 deferred_external_checks，不能把未执行命令写成通过。

## 4. 里程碑登记

| 里程碑 | Pre-F5 可做范围 | 产品实现 gate | Owner | Evidence | Manifest | 解锁 |
|---|---|---|---|---|---|---|
| A0 | `.zui`/Penpot bridge contract、fixture、capability matrix | design-ready | runtime interface + 本 companion | evidence/a0-zui-penpot-contract.md | manifests/a0-zui-penpot.yaml | A1 |
| A1 | Penpot plugin import/edit/export adapter | design-ready；真实 adapter evidence | Penpot adapter + runtime interface review | evidence/a1-penpot-roundtrip.md | manifests/a1-penpot-roundtrip.yaml | A2 |
| A2 | Editor/Runtime 消费同一 `.zui` contract、布局 parity | f0..f4 owner gate；不得绕过 MVP 闸门 | editor/runtime UI owners | evidence/a2-engine-bootstrap-parity.md | manifests/a2-engine-bootstrap.yaml | M0-M3 产品切片 |
| M0 | 文档/审计 | design-ready | 本计划 + 各 owner | evidence/m0-baseline-and-mapping.md | manifests/m0-static.yaml | M1/M2 契约 |
| M1 | token/schema/fixture/lint 规则 | f5-accepted（组件产品化） | layout/01/20、UI/04/05/06、Hub theme | evidence/m1-token-component-contract.md | manifests/m1-token-components.yaml | M2/M3 |
| M2 | 直接支撑 F0/F1 的 shell/入口 | F0/F1 对应 gate | layout/02/03/04/07/19、UI/08、Hub/05 | evidence/m2-shell-contract.md | manifests/m2-shell.yaml | M3/M6-PRE |
| M3 | 直接支撑 F2-F4 的 viewport/Inspector contract | F2/F3/F4 分别 accepted，F5 独占 wave | editor/01/03/05/06、UI/06/08 | evidence/m3-viewport-inspector-contract.md | manifests/m3-mvp-viewport.yaml | F5 input |
| M4 | schema/fixture/拒绝项 | f5-accepted | editor/09/10、UI/04/05/06 | evidence/m4-assets-library-tokens.md | manifests/m4-assets-library.yaml | M5/M7 |
| M5 | disabled collaboration fixture、错误 schema | f5-accepted | editor/14/17、Hub/02/07 | evidence/m5-feedback-task-recovery.md | manifests/m5-feedback-tasks.yaml | M8 |
| M6 | F0/F1 入口与握手 | M6-FULL=f5-accepted | Hub/01/02/05/06/07 | evidence/m6-hub-entry-contract.md、m6-hub-product-surface.md | manifests/m6-hub.yaml | M7/M8 |
| M7 | slot/permission schema | f5-accepted | editor/06/12、layout/04、runtime/06、plugins/01/10 | evidence/m7-domain-extension.md | manifests/m7-domain-extension.yaml | M8 |
| M8 | 静态 lint | f5-accepted + 产品 evidence | 本计划 + Windows validation lane | evidence/m8-quality-performance-a11y.md | manifests/m8-quality.yaml | M9 |
| M9 | 无 | M8 accepted + F0-F5 accepted | 发布/维护 owner | evidence/m9-release-checklist.md | manifests/m9-release.yaml | 后续维护 |

## 5. 性能预算与判定

下表数值在 M0 之前是 candidate observation line，不是已获证据支持的硬门槛。M0 只有在 5.5 的 fixture 与 run-set gates 落地并对当前源、固定 fixture 和固定环境跑出 baseline 后，才可由 `docs/plans/performance/01-mvp-performance-audit-and-optimization.md` owner 在 M0 evidence 中写入 `budget_status: frozen`、机器/环境摘要、source/binary/fixture fingerprint 和冻结值；在此之前只能报告观测值，M8 不得判定 accepted。预算后续若需改变，必须记录原因、前后值、回归风险和 owner 批准，不能静默放宽。

### 5.1 固定 fixture

正式 fixture 使用 schema v2 和 deterministic seed `0x5A495243`，禁止远程 I/O、随机网络等待和未固定的用户目录内容。manifest 必须写入 `fixture_id`、`fixture_schema_version: 2`、`generator_id: zircon.ui-profile-scale-fixture`、`generator_version: 2`、seed、输入计数、语义摘要与内容 digest；仅名称相同而 digest 或语义摘要不同的 fixture 不可比较。当前生成器的 `schema_version: 1` 只可用于 5.4 preflight，不能进入 frozen baseline。Editor 正式性能基线还必须绑定 `client=1440x900 physical px`、`window_dpi=96`、`effective_scale_factor=1.0`、`theme_id=dark`、`density_id=compact`；这些环境字段由 5.5 的 `capture_environment.json` 证明，不能从截图文件名推断。

| Fixture ID | 固定内容 | 主要场景 | Owner / 产出 |
|---|---|---|---|
| ZR-UI-SHELL-v2 | `RenderableEmpty` 启动；单场景含 camera、cube、light；Authoring preset；环境绑定见本节首段 | startup、viewport_image、idle_hover、click、drawer_resize、window_resize | editor_layout/02/03/04/07/19；M2 fixture |
| ZR-UI-HIER-10K-v2 | ZR-UI-SHELL-v2 + 10,000 个稳定层级 ID；4 叉 breadth-first tree，depth 0..7；固定查询 `Node 009` 有 1,000 个 direct match | hierarchy_scroll、hierarchy_filter、layout recompute | editor/05 + editor_ui/08；M3 fixture |
| ZR-UI-VIEWPORT-1K-v2 | ZR-UI-SHELL-v2 + 1,000 个可选静态节点；固定 camera、grid 与 spatial-probe pointer path | viewport_pointer、viewport_toolbar_click、dirty extraction | editor/01/03/05 + editor_ui/08；M3 fixture |
| ZR-UI-ASSET-10K-v2 | 10,000 条本地 registry/catalog DTO；固定类型、relative-path 长度和 thumbnail 状态分布，详见 5.5 | asset_refresh、asset_browser_scroll、thumbnail first result | editor/09/10 + editor_ui/05；M4 fixture |
| ZR-HUB-PROJECT-1K-v1 | 1,000 个本地 project DTO、10,000 条 catalog row、100 条 task row；Projects/Catalog/Builds 路由 | action update、filter、route switch、first usable shell | Hub/05/06；M6 fixture 与 Web harness |
| ZR-TASK-1K-v1 | 100 个排队任务、1 个运行任务、1,000 个固定间隔 progress snapshot；包含 blocked/error terminal state | progress heartbeat、generation monotonicity、recovery | Hub/02/07；M5/M6 fixture |

### 5.2 采样契约

Editor capture 通过 `ui-profile-capture.ps1` 自动调用 `performance-machine-manifest.ps1` 并导出 `machine_manifest.json`；Hub harness 必须复用同一 manifest 生成函数。当前机器清单只证明 CPU/GPU/内存、OS/build、显示分辨率/刷新率、电源模式和后台负载，不包含 window DPI、effective scale、theme 或 density；后四项必须来自 5.5 的窗口/产品 artifact。机器或环境绑定变化后禁止与旧 baseline 直接比较。

| Sample set | 进程与 workload | 每 run 最低有效样本 | Run-set 有效条件 |
|---|---|---:|---|
| Editor frame / dirty extraction | 5 个 fresh process；60 个 presented-frame warmup 后进入 measurement；`viewport_image`/viewport fixture | 300 个 measured presented frame | 合并 >= 1,500；5/5 run 成功；full-tree visit=0 |
| Editor input-to-visible | 5 个 fresh process；每 run 发送 300 个 source-bound pointer/click/wheel event | requested=completed=300；>=100 个 correlation-to-present sample | 合并 >=500 latency sample；最终 input sequence 可见；critical edge dropped=0 |
| Editor layout recompute | 5 个 fresh process；每 run 300 个 hierarchy wheel event | >=60 个 dirty-to-arrange span | 合并 >=300 span；visited/dirty node counter 齐全 |
| Asset thumbnail first result | 30 个 fresh process；每 run 使用独立且启动前为空的 cache partition，并执行一次 asset refresh/request | 1 个 cold request-to-result sample | 30/30 成功；30 个 partition ID/path hash 唯一且 fresh/empty 证据齐全；ready 与 error placeholder 分栏 |
| Editor cold startup | 10 个 fresh process；不做进程内 warmup | 1 个 navigation/process-start-to-first-usable sample | 10/10 成功，compile/fixture load 分栏 |
| Hub action update | 5 个 fresh process；production Web + release Tauri；每 run 300 个 typed action | 300 个 action-to-React-commit correlation | 合并 1,500；generation 单调且 missing=0 |
| Hub first usable shell | 10 个 release Tauri fresh process；不做 warmup | 1 个 navigation-start-to-usable sample | 10/10 成功 |
| Long task heartbeat | 5 个 run；每 run 1,000 个 running progress snapshot | 999 个相邻 interval | blocked/error 另表；running missing=0 |

`p50`、`p95` 使用 nearest-rank；同时报告每个 run 和合并样本的 p50/p95/max、valid/missing/dropped sample 数。计时统一使用 monotonic clock。warmup、fixture load、shader/pipeline compile 和 measurement window 必须分栏；不得把 warmup 样本混入 steady-state，也不得删除慢样本。崩溃、timeout、环境不匹配或缺失 generation 计为 failed run。对比只能发生在相同 fixture digest、client/DPI/scale/theme/density、build profile、机器清单、scenario binding 和 `cache_partition_policy` 上；asset cold-run 的实际 partition ID/path 必须逐 run 不同，不能因不同而判为环境不一致。替代环境只能作为独立观察组，不能冒充正式 baseline。

### 5.3 指标边界与采集 owner

| 指标 | 起点 -> 终点 | Fixture / harness | 归属与阻塞规则 |
|---|---|---|---|
| Editor shell/layout frame | 相邻两个成功 presented generation 的 monotonic timestamp 差；warmup 后开始 | ZR-UI-SHELL-v2；`ui-profile-capture.ps1` viewport_image/drawer_resize/window_resize | performance plan + editor_layout；缺 presented marker 则 M8 blocked |
| Editor input-to-visible feedback | 已进入 Editor input dispatcher 的 pointer/key event -> 首个包含对应 command correlation 的 presented generation | ZR-UI-SHELL-v2 / ZR-UI-VIEWPORT-1K-v2；click/viewport_pointer | editor/03 + editor_ui/08；不能以 handler return 代替 visible feedback |
| layout recompute | dirty layout request 被接受 -> 同一 generation arrange 结果发布 | ZR-UI-HIER-10K-v2；hierarchy_scroll/hierarchy_filter | editor_layout/03/04；必须同时记录 dirty/visited node 数 |
| viewport dirty extraction | 冻结 dirty set -> 对应 render batch generation 发布 | ZR-UI-VIEWPORT-1K-v2；viewport_pointer + render-extract scenario | editor/01/05 + runtime_interface；每帧 full-tree visit 非零即失败 |
| asset thumbnail first result | thumbnail request accepted -> 同一 asset generation 的 ready/error placeholder 可见 | ZR-UI-ASSET-10K-v2；asset_refresh | editor/09/10 + editor_ui/05；缓存命中与冷缓存分栏 |
| Hub action state update | `dispatchHubAction` 接受 action/correlation -> React commit 包含对应 `stateGeneration` | ZR-HUB-PROJECT-1K-v1；Hub/06 Web performance harness | Hub/01/05/06；handler promise 完成不等于可见完成 |
| Hub first usable shell | navigation start -> project list 和主 action 可交互且已绑定最新 generation | ZR-HUB-PROJECT-1K-v1；release Tauri + production Web cold-run harness | Hub/05/06；缺真实 shell 证据则保持 deferred |
| Long task heartbeat | 相邻两个 running progress snapshot 的 publication timestamp 差 | ZR-TASK-1K-v1；Hub/02 task harness | Hub/02/07；blocked 状态单独统计，不混入 running heartbeat |

缺少 marker、correlation/generation、fixture generator、环境绑定或 run-set harness 本身就是相应 owner 的 M8 阻塞项，不能改用人工秒表、浏览器肉眼或空页面数据替代。

### 5.4 当前可执行的 source-bound preflight

Editor 统一从协调器登记的 profiling product directory 采集。下面命令在当前仓库可执行，并通过 `AutoCloseSeconds` 保证每个 fresh process 无人值守退出；`ZIRCON_PROFILE_INITIAL_CLIENT_*` 请求 1440x900 client，最终尺寸仍必须由 `ui_profile_geometry.json` 验证。它们能产生 source/binary、适用场景的 input fixture、machine 绑定和场景证据，但当前 scale fixture 仍是 schema v1（hierarchy 扁平、asset 只有 JSON index），工具也尚不导出 window DPI/theme/density 或聚合 run-set 最低样本数，因此只能作为 M0 preflight，不能单独把预算冻结或让 M8 通过。

    $productDirectory = $env:ZIRCON_DESIGNMENT_PROFILE_PRODUCT_DIR
    if ([string]::IsNullOrWhiteSpace($productDirectory)) {
        throw 'Set ZIRCON_DESIGNMENT_PROFILE_PRODUCT_DIR to the coordinator-registered profiling product directory.'
    }
    $env:ZIRCON_PROFILE_INITIAL_CLIENT_WIDTH = '1440'
    $env:ZIRCON_PROFILE_INITIAL_CLIENT_HEIGHT = '900'

    .\tools\ui-profile-capture.ps1 -Scenario startup -OutputRoot E:\zircon-profiles\designment-m8-startup -ProductDirectory $productDirectory -SkipBuild -RequireScenarioEvidence -AutoCloseSeconds 10 -WithinProcessWarmupPresentCount 0 -MeasuredRunCount 10 -MaxFrames 2048

    .\tools\ui-profile-capture.ps1 -Scenario viewport_image -OutputRoot E:\zircon-profiles\designment-m8-frame -ProductDirectory $productDirectory -SkipBuild -AutoInteract -RequireScenarioEvidence -AutoCloseSeconds 30 -WithinProcessWarmupPresentCount 60 -MeasuredRunCount 5 -MaxFrames 4096

    .\tools\ui-profile-capture.ps1 -Scenario idle_hover -OutputRoot E:\zircon-profiles\designment-m8-input -ProductDirectory $productDirectory -SkipBuild -AutoInteract -RequireScenarioEvidence -AutoPointerMoveCount 300 -AutoCloseSeconds 30 -WithinProcessWarmupPresentCount 60 -MeasuredRunCount 5 -MaxFrames 4096

    .\tools\ui-profile-capture.ps1 -Scenario hierarchy_scroll -OutputRoot E:\zircon-profiles\designment-m8-layout -ProductDirectory $productDirectory -SkipBuild -AutoInteract -RequireScenarioEvidence -HierarchyLogicalNodeCount 10000 -AutoWheelCount 300 -AutoCloseSeconds 30 -WithinProcessWarmupPresentCount 60 -MeasuredRunCount 5 -MaxFrames 4096

    .\tools\ui-profile-capture.ps1 -Scenario viewport_pointer -OutputRoot E:\zircon-profiles\designment-m8-viewport -ProductDirectory $productDirectory -SkipBuild -AutoInteract -RequireScenarioEvidence -ViewportSelectableNodeCount 1000 -AutoPointerMoveCount 300 -AutoCloseSeconds 30 -WithinProcessWarmupPresentCount 60 -MeasuredRunCount 5 -MaxFrames 4096

    1..3 | ForEach-Object {
        $assetRoot = "E:\zircon-profiles\designment-m8-asset-$($_)"
        .\tools\ui-profile-capture.ps1 -Scenario asset_refresh -OutputRoot $assetRoot -ProductDirectory $productDirectory -SkipBuild -AutoInteract -RequireScenarioEvidence -AssetCatalogItemCount 10000 -AutoCloseSeconds 30 -WithinProcessWarmupPresentCount 60 -MeasuredRunCount 10 -MaxFrames 4096
    }

`asset_refresh` 当前每个进程只触发一次 source-bound 文件变更，因此用 3 x 10 fresh process 形成 30 个 cold sample；不能把 5 个进程里的 frame 数误当作 1,500 个 thumbnail latency sample。

### 5.5 M0 必须补齐的 fixture 与 run-set gates

#### 5.5.1 Fixture schema v2 硬切

当前 `ui-profile-scale-fixture.ps1` 会把 hierarchy 全部写成 `parent = 0`，asset source 只包含 `profile_asset_index`；`profile-capture-manifest.ps1` 只复核 count/file digest，没有 seed、generator version 或语义分布。因此这些 v1 digest 只能证明“同一批字节”，不能证明 5.1 声明的 workload。M0-S4 必须在原工具和原 Pester 文件中完成以下 hard cut，不新建第二个 fixture generator，也不让 schema v1 进入正式 baseline：

1. `ZR-UI-SHELL-v2` 为无 scale input 的启动/steady fixture 增加 `kind: shell_project` descriptor，绑定 canonical `renderable-empty` template、scene、Authoring preset 和相关文件 digest；formal capture 的 `input_fixture` 不得再为 null。
2. `ZR-UI-HIER-10K-v2` 使用稳定 4 叉 breadth-first tree：entity 1 是 `parent=0` 根，entity `n >= 2` 的 parent 为 `floor((n - 2) / 4) + 1`。10,000 个 entity 必须恰好覆盖 depth 0..7；名称仍为零填充 stable ID；查询 `Node 009` 必须得到 1,000 个 direct match，ancestor closure 另计。
3. `ZR-UI-VIEWPORT-1K-v2` 保留 1 camera + 1 light + 1,000 selectable node 的 grid 语义，但 descriptor 必须绑定 camera transform、grid dimensions、mobility、spatial-probe path、generator/seed 和 scene digest。
4. `ZR-UI-ASSET-10K-v2` 的导入后 catalog 必须恰有 10,000 个 stable row，并按 seed 的稳定 hash 交错排列，禁止按类别成段或使用 `Get-Random`。类型计数为 Data/JSON 4,000、Texture/PNG 2,000、Material/zmaterial 1,500、Model/OBJ 1,000、Scene/scene.toml 1,000、Unsupported/error 500；normalized relative-path 长度桶为 24-47 bytes 6,000、48-79 bytes 3,000、80-119 bytes 1,000；thumbnail 终态为 ready 7,000、source-generatable/pending 2,000、missing/error 1,000。生成器可复用 canonical tiny payload，但 acceptance 必须读取实际 registry/catalog DTO，不能只统计源文件扩展名。
5. v2 fixture descriptor 的 `semantic_contract` 至少包含上述层级公式或 asset 分布、expected query/match、稳定的 `cache_partition_policy: fresh-empty-per-process`、payload/template digest；共享语义摘要不得包含实际 partition ID/path。`profile-capture-manifest.ps1` 必须重新计算并比对语义摘要与所有文件集合 digest，拒绝缺字段、分布不符、schema v1、tamper 和不同 generator version。
6. 扩展 `tools/tests/ui-profile-scale-fixture.Tests.ps1`：两次不同外部根生成必须得到相同内容 digest/semantic summary；验证 4 叉 parent/depth/query、viewport grid、六类 asset/三种路径桶/三种 thumbnail 状态、实际 catalog DTO 计数、v1 fail-closed 和单文件/单字段篡改。扩展 `ui-profile-capture-output-contract.Tests.ps1`，证明 v2 descriptor 在进程启动前写入 source manifest。

#### 5.5.2 Run-set 与环境 gate

M0-S5 必须新增 `tools/ui-profile-run-set-contract.ps1` 及 `tools/tests/ui-profile-run-set-contract.Tests.ps1`，并扩展 capture 输出而不是另建第二套 profiler。M0-S4 完成后必须重新执行 5.4 的采集命令，让所有 capture root 绑定 schema v2；hard cut 前产生的 root 禁止复用。该工具落地前，下列命令是计划中的验收接口，不是当前已存在的测试：

    .\tools\ui-profile-run-set-contract.ps1 -CaptureRoots E:\zircon-profiles\designment-m8-startup,E:\zircon-profiles\designment-m8-frame,E:\zircon-profiles\designment-m8-input,E:\zircon-profiles\designment-m8-layout,E:\zircon-profiles\designment-m8-viewport,E:\zircon-profiles\designment-m8-asset-1,E:\zircon-profiles\designment-m8-asset-2,E:\zircon-profiles\designment-m8-asset-3 -ExpectedClientWidthPx 1440 -ExpectedClientHeightPx 900 -ExpectedWindowDpi 96 -ExpectedThemeId dark -ExpectedDensityId compact -RequireSampleMatrix designment-v2 -OutputPath E:\zircon-profiles\designment-m8-run-set.json

    Invoke-Pester -Path .\tools\tests\ui-profile-run-set-contract.Tests.ps1

run-set gate 必须完成以下工作：

1. 对每个 `source_manifest.json` 校验 scenario、capture-root/group、`run_ordinal`、`measured_run_count`、warmup/options、source/binary hash，以及 fixture schema v2 ID/generator/seed/semantic summary/content digest；同一 capture group 内 ordinal 缺失/重复、任何 v1/null fixture 或任意 group 来自不同 binary 的集合直接失败。asset 的 3 个 capture root 分别是一个 10-run group，合并后必须恰有 30 个唯一 session；fixture 中只比较共同的 `cache_partition_policy`，不把实际 partition ID 当成共享 fixture binding。
2. 从 `ui_hotspots.json`、`ui_interaction_evidence.json` 和 latency/counter artifacts 读取实际 sample，不使用 `MaxFrames` 或请求数冒充完成数；按 5.2 表逐指标验证 per-run 与 aggregate 下限。
3. capture 在主窗口可用后写出 `capture_environment.json`：`schema_version`、实际 `window_client_width_px/height_px`、Win32 `GetDpiForWindow` 得到的 `window_dpi`、`effective_scale_factor`、运行时解析后的 `theme_id/density_id`。asset run 还必须写入实际 `cache_partition_id`、规范化绝对路径的 `cache_partition_path_hash`、`created_for_run_ordinal`、`created_fresh` 和启动前扫描得到的 `pre_run_entry_count`。尺寸可复用 `ZIRCON_PROFILE_INITIAL_CLIENT_*` 和 `ui_profile_geometry.json`，DPI 查询可复用 `capture-editor-ui-visual.ps1` 的现有 Win32 路径；theme/density 必须由 retained host 发布，不能从期望参数回填。
4. 只在全部 run 的共同 fixture/environment/source binding 相同且 sample matrix 满足时生成 `status: passed`；asset 集合还必须证明 30 个 `cache_partition_id` 和 path hash 各自唯一、`created_fresh=true`、`pre_run_entry_count=0`，并在 run 结束前保持该 run 的专属 ownership。报告同时列出 failed/missing/dropped run、每 run 和 aggregate nearest-rank p50/p95/max。
5. Pester 至少覆盖少 run、少 sample、重复 ordinal、不同 binary/fixture/DPI/theme/density、warmup 混入、慢样本保留、重复/reused/nonempty cache partition 和完整通过 fixture。`ui-profile-capture-output-contract.Tests.ps1` 也必须增加 environment artifact、cache partition 证据与自动退出参数的源码/输出 contract。

`tools/tests/render-extract-performance-scenario.Tests.ps1` 继续作为 viewport dirty-extraction 的场景绑定与确定性 contract；fixture、machine、capture、render-extract 与 run-set 五组 Pester 必须全部通过：

    Invoke-Pester -Path .\tools\tests\ui-profile-scale-fixture.Tests.ps1
    Invoke-Pester -Path .\tools\tests\performance-machine-manifest.Tests.ps1
    Invoke-Pester -Path .\tools\tests\ui-profile-capture-output-contract.Tests.ps1
    Invoke-Pester -Path .\tools\tests\render-extract-performance-scenario.Tests.ps1
    Invoke-Pester -Path .\tools\tests\ui-profile-run-set-contract.Tests.ps1

Hub/06 必须在产品测量前提供 `zircon_hub/web/tests/ui_performance_contract.test.mjs`、production Tauri cold-run launcher、ZR-HUB-PROJECT-1K-v1 fixture 和同构的 run-set JSON；计划命令为：

    Push-Location zircon_hub
    node --test --test-name-pattern="ui performance contract" web/tests/ui_performance_contract.test.mjs
    Pop-Location

上述 run-set 文件和 Hub 文件目前都是计划产物而非现有测试；在它们、真实 shell marker、环境 artifact 和 fixture 落地前，`budget_status` 必须保持 `candidate`，对应性能项只能标记 `deferred_missing_harness`，不能判定通过。

### 5.6 冻结后的候选阈值

| 指标 | 目标 | P1 预警 | P0 失败 |
|---|---:|---:|---:|
| Editor shell/layout steady-state frame | p95 <= 16.7 ms（60 FPS） | > 16.7 ms 且 <= 33.3 ms | p95 > 33.3 ms 或连续掉帧影响输入 |
| Editor input-to-visible feedback | p95 <= 50 ms | > 50 ms 且 <= 100 ms | > 100 ms 或丢失 pointer/key event |
| 单次 layout recompute（标准 fixture） | p95 <= 4 ms | > 4 ms 且 <= 8 ms | > 8 ms 或全树重算造成卡顿 |
| viewport dirty extraction（无结构变化） | p95 <= 2 ms，零全树遍历 | > 2 ms | 每帧全树扫描或 p95 > 5 ms |
| asset thumbnail first result（本地 fixture） | p95 <= 250 ms | > 250 ms 且 <= 500 ms | > 500 ms 或阻塞输入 |
| Hub action state update | p95 <= 100 ms | > 100 ms 且 <= 250 ms | > 250 ms 或 generation 倒退 |
| Hub first usable shell | p95 <= 1500 ms（冷启动 fixture） | > 1500 ms 且 <= 2500 ms | > 2500 ms 或无错误反馈 |
| Long task progress heartbeat | <= 500 ms 间隔 | 500-1000 ms | > 1000 ms 且无明确 blocked 状态 |

预算不适用于尚未完成的功能，不能用空页面测出“通过”。M0 冻结值若与候选值不同，以带 owner 审核和 machine/source fingerprint 的 M0 evidence 为准；M8 报告必须同时保留 candidate、frozen baseline 和 measured 三列，避免基线漂移。

## 6. 视觉和交互证据矩阵

| 维度 | 值 |
|---|---|
| Editor viewport | 1280x720、1440x900、1920x1080、1024x768、900x700 |
| Hub viewport | 1280x720、1440x900、1024x768、768x1024 |
| DPI | 100%、125%、150%、200%（Editor）；browser device scale 记录（Hub） |
| Theme/density | Editor dark comfortable/compact；Hub 当前主题；后置 high-contrast/light fixture |
| Content | short、中文/英文长文本、数字/单位、缺失 asset、空 page/board |
| State | loading、empty、error、disabled、read-only、pending、saving、saved、conflict、offline |
| Input | mouse、keyboard-only、focus-visible、IME/numeric、reduced-motion |

截图命名格式：<milestone>_<product>_<surface>_<viewport>_<dpi>_<theme>_<density>_<state>_<fixture>_<generation>.<ext>。交互 trace 至少记录入口、焦点起点、键盘/指针序列、command/action id、预期状态、实际状态和 correlation/generation。

## 7. 失败与 handoff

1. 失败先按 runtime_interface -> runtime/editor owner -> UI host -> page 顺序缩小，不在页面层加旁路。
2. 若命中已有 failure-*.md，先阅读并把本次证据回传原 failure owner；不得复制同一 failure。
3. 若需要跨 owner 修复，创建带最小复现、当前源指纹、命令、输出和阻塞 gate 的 handoff；修复后只重跑受影响 focused batch，再回到里程碑 batch。
4. 视觉问题若不影响结构/交互可标为 P2 polish，但不能把 P0/P1 溢出、焦点丢失、错误不可恢复或命令绕过标为 polish。
5. 未能执行的 Web、GPU、DPI 或平台验证写入 deferred_external_checks，同时降低里程碑状态，不得填写绿色通过。

## 8. 发布前 checklist

- [ ] M0-M3 的 owner/区域/状态/SelectionModel/gateway 边界有 evidence。
- [ ] docs/plans/mvp/index.md F0-F5 均有当前源 accepted 记录，且 F5 使用 clean validation copy。
- [ ] M4-M8 的 manifest 都有 mvp_gate: f5-accepted、实际命令、target-dir、profile 和结果。
- [ ] zircon.ui.tokens/v1、zircon.ui.library/v1 的支持/拒绝字段、冲突和回滚 fixture 已执行。
- [ ] collaboration 只有真实 capability event；无 backend 时所有 presence/comment fixture 为 disabled/unavailable。
- [ ] Editor 五个基准尺寸、DPI、键盘、长文本和状态矩阵通过；Hub 四个尺寸和 npm run typecheck/build 有输出。
- [ ] 性能预算报告包含 p50/p95/max、样本数、环境和超预算处置。
- [ ] 未完成项、open failure、deferred checks、owner 和回滚方案已登记。
- [ ] M9 evidence、设计 changelog、迁移/deprecation 规则已由 owner review。

## 9. 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| A0 | `.zui`/Penpot bridge contract | validated | 2026-08-31 | [a0-zui-penpot-contract.md](./evidence/a0-zui-penpot-contract.md) |
| A1 | Penpot import/edit/export adapter | external-check-deferred |  | [a1-penpot-roundtrip.md](./evidence/a1-penpot-roundtrip.md) |
| A2 | Engine bootstrap/layout parity | in_progress |  | [a2-engine-bootstrap-parity.md](./evidence/a2-engine-bootstrap-parity.md) |
| M0 | 基线与模式映射 |  |  |  |
| M1 | token/component contract |  |  |  |
| M2 | shell/入口 contract |  |  |  |
| M3 | viewport/Inspector MVP contract |  |  |  |
| M4 | assets/library/tokens |  |  |  |
| M5 | feedback/tasks/recovery |  |  |  |
| M6 | Hub entry/product surface |  |  |  |
| M7 | domain/extension |  |  |  |
| M8 | quality/performance/a11y |  |  |  |
| M9 | release/maintenance |  |  |  |

> A0-A2 已进入执行阶段，因此记录当前证据和未闭合检查；M0-M9 仍保持规划态。此表不代替原 owner 计划的完成记录。
