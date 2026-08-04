---
related_code:
  - templates/projects/renderable-empty/assets/scenes/main.scene.toml
  - templates/projects/renderable-empty/assets/models/cube.obj
  - templates/projects/renderable-empty/assets/materials/default.zmaterial
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/render
  - zircon_app/src/entry/entry_runner/runtime.rs
related_tests:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample/end_to_end.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/02-f1-project-and-assets.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
status: blocked_by_f1
gate: F2
last_refined: 2026-07-24
---

# F2 场景运行 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans`。遇到渲染/输入失败先使用 `systematic-debugging` 和 `support-first-regression-testing`；产品图像必须保存真实 PNG 并检查非空像素。

**Goal:** F1 创建的持久项目包含 camera、可见 primitive 和 directional light；runtime 从磁盘加载它、接收 keyboard/mouse、输出可验证 WGPU 帧，并在两次 session 生命周期中确定性启动和退出。

**Architecture:** 以 product template 为唯一 F2 fixture，复用现有 `foundation_render` 的真实 WGPU capture 和诊断断言，删除测试 helper 与产品模板之间的场景分裂。Runtime session 负责加载和渲染，App runtime entry 只负责正常产品循环。

**Tech Stack:** SceneAsset TOML、AssetRef、WGPU frame capture、RuntimeDynamicSession、Winit input events、PNG evidence。

---

## 1. 入口条件

- [ ] F1 canonical 项目已由 staged editor 创建，并冻结 project root、manifest hash 和 registry summary。
- [ ] F2 Session 已领取 template scene、F2 test、runtime session/input/render 相关文件 lease。
- [ ] 当前 validation machine 有可用 WGPU adapter；adapter/backend/device limits 会进入证据。
- [ ] Runtime 08/09/12 和 render foundation 中直接影响单 camera/mesh/light/input 的 failure 已分类。

## 2. 固定 F2 场景合同

| Entity | 必需状态 |
|---|---|
| Camera | active；有效 projection；能看到 primitive；near/far 有效 |
| Primitive | active/static；非零 transform scale；场景持久化 `kind = "project"` + guid + `path_hint`，registry 解析后得到 `res://models/cube.obj` 和 `res://materials/default.zmaterial` |
| Sun | active directional light；非零 intensity；方向归一化或由 loader 合法归一 |

附加合同：

- 场景只通过 persisted AssetRef 连接 mesh/material，不使用运行时临时 handle 或测试注入 primitive。
- 至少 100 个像素与背景不同，且 render diagnostics 报告 mesh draw > 0、directional light > 0、material validation error = 0。
- 输入至少覆盖 viewport resize、pointer move、left-button press/release、`W` press/release。
- unchanged 第二帧不得重新编译 RenderGraph，也不得重新上传静态 GPU scene 数据。

## 3. 非目标

- 不要求 PBR 完整材质模型、阴影质量、volumetrics、GI 或多 camera。
- 不要求 viewport gizmo、selection overlay 或 Editor authoring。
- 不以 offscreen synthetic triangle 替代 template 中的 cube asset。
- 不因性能扩展 failure 阻塞单场景，除非 stable second-frame 合同被破坏。

## 4. M3.1 Product template 场景收敛

### 目标

`RenderableEmpty` 的 default scene 成为 F2 canonical scene，不再只有 Camera/Sun，也不再由 `foundation_render` 单独生成另一份场景语义。

### 实现切片

- [x] `templates/projects/renderable-empty/assets/scenes/main.scene.toml` 已包含唯一 Cube entity，以及 canonical project-kind mesh/material persisted refs；后续 gate 继续验证解析与可见性。
- [ ] 根据实际 camera convention 调整 camera/cube transform，保证 primitive 位于 view frustum，避免靠测试 shader 绕过 camera。
- [ ] 让 template contract 测试解析场景并断言实体数量、kind、active state、transform 和 refs；禁止用文本 substring 代替 schema parser。
- [ ] 把 `write_static_lit_default_scene` 与 product template 的共享语义收敛到一个 fixture builder/parsed contract；测试 helper 不复制另一套 entity constants。
- [ ] 保持 template 内已有 cube/material/shader 文件和 scene refs 一致；缺失、错误 kind 或 unresolved ref 必须让创建/open test 失败。

### 测试阶段：F2 Template Scene Gate

- [ ] 运行 Editor template creation 与 Runtime SceneAsset parse/roundtrip focused suites。
- [ ] 从全新 template 创建项目，解析 default scene 并验证三类实体和两个 asset refs。
- [ ] 让 ProjectManager scan/import 后解析相同 scene，确认 refs 解析到 F1 registry identity。
- [ ] 运行 missing mesh/material 和 invalid transform 负例，确认 typed diagnostic。

### 退出证据

- [ ] 产品模板本身满足 camera/primitive/light 合同。
- [ ] 测试 helper 与产品 template 不再维护两套不同基础场景。
- [ ] F1 canonical 项目重新生成后 registry summary 包含可解析 mesh/material refs。

## 5. M3.2 Runtime WGPU 帧与输入

### 目标

现有 `foundation_render` 从 canonical template 项目加载真实资产，验证输入、渲染、steady state 和 session restart。

### 实现切片

- [x] `foundation_render.rs` 已通过 `render_project_template(ProjectTemplateId::RenderableEmpty, ...)` 创建 canonical template 项目，不再手写独立 triangle/project manifest/scene。
- [ ] 保留真实 `capture_frame`、RGBA ownership/free callback 和 render diagnostics 断言。
- [ ] 使像素断言基于可见 primitive 与背景差异，不绑定只存在于测试 shader 的绿色常量；同时保留 draw/light/material error 等结构诊断。
- [ ] 输入测试在 session event ingress 注入 resize、pointer、mouse、keyboard press/release，并验证 InputManager 实际消费。
- [ ] 第二帧断言 compiled graph cache hit 增长、miss 不增长、static GPU scene dirty/upload 为 0。
- [ ] Drop 第一 session 后重新打开同一 persisted project，比较 mesh draw、light count、frame dimensions 和非背景像素。
- [ ] 保留可选 `ZR_F2_BASIC_SCENE_CAPTURE_PNG` 输出，用于 F5 保存真实 PNG evidence。

### 测试阶段：F2 Runtime Session Gate

- [ ] 在 Windows WGPU adapter 上运行 `render_product_f2_persisted_basic_scene_renders_accepts_input_and_shuts_down`。
- [ ] 保存 PNG，检查尺寸、alpha、非背景像素和 primitive 覆盖；空白/全背景图直接失败。
- [ ] 在相同项目连续建立两次 RuntimeDynamicSession，确认第一次 Drop 后项目目录无被占用文件。
- [ ] 记录 adapter/backend/device limit 和 render stats；software adapter 只有在明确记录且满足产品 policy 时可接受。

### 退出证据

- [ ] canonical template 产生真实非空 WGPU 帧。
- [ ] keyboard/mouse press/release 被 runtime input owner 接收和消费。
- [ ] stable second frame 不重复编译 graph 或上传静态 scene。
- [ ] session restart 重现相同持久场景。

## 6. M3.3 Runtime 产品循环

### 目标

staged `zircon_runtime` 通过正常 App entry 打开 F1 项目，进入窗口循环、呈现 F2 场景并在首帧后干净退出。

### 实现切片

- [ ] 确认 runtime startup args 把 canonical project root 投影到 `RuntimeProjectConfig`，不在 App 层重新创建 fixture。
- [ ] `ZIRCON_RUNTIME_CAPTURE_FRAME_PNG` 必须是绝对路径，并在 runtime startup 通过
  `ProjectPaths` 只解析一次：文件写入保留物理操作路径，产品诊断只发布显示路径。
- [ ] 首帧退出只在 scene load、input manager、render graph 和 presented surface 成功后生效。
- [ ] 产品诊断输出 project identity、scene URI、adapter/backend、frame dimensions、draw/light/pass counts 和 teardown result。
- [ ] 对 missing scene、unresolved mesh/material、device/surface failure 返回非零退出和 typed diagnostic。
- [ ] process harness 支持把运行时截图/帧 capture 与 stdout/stderr 归档到同一 validation run。

### 测试阶段：F2 Product Runtime Gate

- [ ] 从 F0 staging 启动 runtime product 并传入 F1 canonical project root。
- [ ] 发送或通过 host 产生至少一次 keyboard/mouse 输入，再等待首个成功 presented frame。
- [ ] 捕获产品帧/窗口和 runtime diagnostics，断言与 session-level F2 数据一致。
- [ ] 退出后立即第二次启动同一项目，重复帧/诊断断言并检查无文件锁。

### 退出证据

- [ ] `zircon_runtime` executable 而非仅测试 harness 完成 persisted scene loop。
- [ ] 两次产品运行都呈现可见 primitive、接受输入并干净退出。
- [ ] F3 接收相同项目，未替换 fixture。

## 7. F2 阶段退出清单

- [ ] M3.1、M3.2、M3.3 全部通过。
- [ ] product template、session integration test 和 runtime executable 使用同一场景/资产语义。
- [ ] PNG/窗口证据非空且诊断 mesh draw/light/pass 均大于 0。
- [ ] 输入包含 pointer、mouse button 和 keyboard 的 press/release。
- [ ] 两次 session/product 生命周期都确定性退出并释放文件。
- [ ] 高级 render/performance failure 未被错误纳入 MVP。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Code Review 处理结果 (2026-08-01)

### 已处理

- M3.1 的 canonical Cube/project refs 与 M3.2 的 template-driven `foundation_render` 创建路径已在实现 checklist 中标记为当前源码已落地。
- 固定场景合同已区分持久 `kind = "project"` + guid/path_hint 与 registry 解析后的 `res://` 逻辑 URI。

### 实现风险 / 技术债

- `failure-2026-07-30-runtime-frame-capture-sibling-module-projection.md` 的源码路径已改为 `super::super::frame_capture`，但 canonical failure 在精确受管 Rust 测试通过前继续保持 open；不得用静态路径核对替代 failure return。
