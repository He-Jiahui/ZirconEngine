---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/ui/text
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/tests/extensions/animation_physics_absorption.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack.rs
  - zircon_runtime/src/platform/tests/feature_manifest.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
status: completed
last_refined: 2026-07-12
---

# 01 技术选型与依赖治理

## 现状与证据（2026-06-12 重核）

- 声称栈 5 处失实（cosmic-text、kira、zip/tar、rfd、arboard）：五库在全仓任何 `Cargo.toml` 均 0 命中（2026-06-12 grep 重核属实）。核对表全文见 `docs/plans/zircon_runtime/runtime/index.md` §1.1；本次细化只复核了五库缺席这一项，§1.1 其余行执行时逐项核验（命令见"执行前检查清单"）。
- 文本栈三库并存且口径未定：glyphon `0.11.0`（`zircon_runtime/Cargo.toml:77`）、fontsdf `0.5.3`（L78）、unicode-segmentation `1.13.2`（L95）；fontdue `0.9.3` 仅在 editor（`zircon_editor/Cargo.toml:11`）。自研 text shaper / hit-testing 在 `zircon_runtime/src/ui/text/`（mod.rs、shaper.rs、hit_test.rs、layout_engine.rs 约 25.8KB、edit_state.rs、grapheme.rs、rich_text.rs）。
- glyphon 口径矫正（2026-07-01 重核）：原文"glyphon 承担 runtime GPU 文本渲染"只对了一半——渲染侧仍由 glyphon 承担 native text/render intent，但 shaping/layout 当前已统一到 `SharedTextService`：`shaper.rs:99-104` 的 `active_layout_backend_for_intent` 对 `SharedTextService` / `NativeGlyphon` / `SdfAtlas` 均返回 `SharedTextService`，`fallback_reason_for_backend` 返回 `None`，且测试 `text_shaper_stack_uses_shared_text_service_for_font_backends` 锁定 Native/SDF render mode 通过共享文本服务获得 layout metrics。
- 公共面注意（2026-06-12 重核）：`ui/text` 对外仅 `pub use shaper::layout_text`（`layout_text(text, style, frame, clip_frame) -> UiResolvedTextLayout`，shaper.rs:196-203）；`UiTextShaper` trait（shaper.rs:34-37）、`hit_test_text_layout`、`UiTextHitTest` 等均 `pub(crate)`。文档示例不得引用不出 crate 的类型。
- 版本风险：winit `0.31.0-beta.2`（根 `Cargo.toml:37`，default-features = false）、notify `9.0.0-rc.3`（L27）。同文件 wgpu `29.0.1`（L36）、naga `29.0.1`（L26）、glam `0.32.1`（L23）。
- `zr_vm_rust_binding` / `zr_vm_rust_binding_sys` 是指向仓库外 `../../zr_vm/...` 的路径依赖（`zircon_runtime/Cargo.toml`，optional），由 feature `backend-zr-vm` 门控。2026-06-12 的 plugin lifecycle 修复已在 `../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs` 落地空参数导出调用 marshalling 防御，当前 `backend-zr-vm` 验证必须与这份本地 binding 修复配对。
- 物理现状矫正（2026-06-12 重核）：原文"`zircon_plugins/physics/runtime` 为空壳"已过时。该插件现有 37 个文件 / 4353 行 Rust：自研 builtin 物理（manager/ 7 文件 859 行：builtin_step、clock、query、service、settings、validation、world_sync）、query_contact（raycast aabb/capsule/sphere、overlap、contact/filter/geometry）、trigger、scene_hook，外加 1707 行 `physics_manager_runtime_contract` 集成测试。"无任何物理依赖"仍属实：`zircon_plugins/physics/runtime/Cargo.toml` 仅依赖 `zircon_runtime`。jolt 空 feature 有两处：`zircon_runtime/Cargo.toml:18` 与 `zircon_plugins/physics/runtime/Cargo.toml:10`（原文漏列后者）；`backend.rs:5-10` 中 `JOLT_ENABLED = cfg!(feature = "jolt")` 而 `JOLT_BACKEND_AVAILABLE = false` 硬编码——jolt 是"可声明但永不可用"的后端槽位。
- 物理当前落地（2026-07-10）：上述 2026-06-12 基线已被 Plugins 03 M1-T3 硬切取代。Runtime manifest 只保留 `backend-jolt` profile vocabulary；Physics plugin 通过 optional `joltc-sys` 独占真实 Jolt backend，feature-on 为 Ready/native step，feature-off 为 Unavailable，且两条路径均不静默降级 builtin。当前守卫为 `physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned`；具体状态归 Runtime 01 编号产出记录。
- 导出打包：`ExportPackagingStrategy` 三变体 Copy 枚举（SourceTemplate / LibraryEmbed / NativeDynamic，serde snake_case，`plugin/export_profile.rs:115-121`）；`ExportProfile.strategies: Vec<ExportPackagingStrategy>`（L131），默认 `[SourceTemplate, LibraryEmbed]`（L188-193）。仓内唯一压缩/归档依赖是 zstd `0.13.3`（`zircon_runtime/Cargo.toml:100`），无 zip/tar/容器实现。全仓 `ExportPackagingStrategy` 引用 76 个代码文件 / 386 处。
- 守卫口径矫正（2026-06-12 重核）：原 M1 切片 4"锁定 `zircon_runtime_interface`、`zircon_editor` 不出现 `wgpu`/`winit` 直依（现状已满足）"对 zircon_editor 不成立——`zircon_editor/Cargo.toml:23` 存在 `winit.workspace = true` 直依（softbuffer `0.4.6`（L19）自绘 retained host 需要 winit 类型）。`zircon_runtime_interface` 确认无 wgpu/winit（依赖仅 glam/serde/serde_json/thiserror/toml/unicode-segmentation/uuid）。守卫口径已在 M1 切片 1.4 修正。
- 参考引擎对照矫正（2026-06-12 重核）：原 M3 称 `dev/Fyrox` 为"自研物理"失实——`dev/Fyrox/fyrox-impl/Cargo.toml:30-31` 依赖 rapier2d/rapier3d `0.32`，Fyrox 是 rapier 外挂形态。

参考引擎对照（每点一行）：

- Bevy：winit + wgpu + glam + notify 同代基础栈 — `dev/bevy/crates`
- Bevy 文本：cosmic-text 整合方案对照 — `dev/bevy/crates/bevy_text/src/lib.rs`
- Fyrox：rapier 外挂物理 — `dev/Fyrox/fyrox-impl/Cargo.toml:30-31`
- Godot：自研 + Jolt 双后端并存 — `dev/godot/modules/godot_physics_3d`、`dev/godot/modules/jolt_physics`
- Unreal：巨型 Runtime 模块树（物理内置 Chaos）— `dev/UnrealEngine/Engine/Source/Runtime`

## 目标

1. 让"声称技术栈"与实仓一致，固化为一份权威技术选型文档。
2. 给每个风险依赖定稿治理策略（锁定、升级 gate、替换或 vendor）。
3. 对物理、导出归档、编辑器辅助三个缺口给出选型决策（决策本身在本计划，落地实现各归 owner 计划）。

## 非目标

- 不在本计划内实现物理引擎、归档器或编辑器对话框。
- 不调整渲染相关依赖（wgpu/naga 升级节奏归 render 计划与 RHI 会话）。
- 渲染骨架内容（RDG/MeshDrawCommand/GPUScene/可见性/光照/时域/后处理/permutation）一律归 render 计划 01-08，本计划不展开；glyphon 渲染侧提交路径的改造同样不在本计划。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留 re-export/alias/shim（含 feature 别名）。
- generated 产物只许 leaf DTO/table。
- 动态边界只传 ABI-safe 值与序列化负载。
- 非网络语义 server 命名是 blocker：本计划新增的文档、测试、feature 命名一律不得引入。

## 执行前检查清单

开工前逐项完成，未过项禁止动工：

1. 活动会话对齐：列出 `.codex/sessions/` 最新条目（细化时点最近为 `20260612-0222-runtime-plan-engineering-refinement.md`），确认无并发会话占用文本栈 / 物理 / 选型文档同一路径。
2. worktree 脏文件检查（应全部无输出，或差异与本计划无关）：
   - `git status --porcelain -- Cargo.toml zircon_runtime/Cargo.toml zircon_editor/Cargo.toml zircon_runtime_interface/Cargo.toml`
   - `git status --porcelain -- docs/engine-architecture/ docs/zircon_runtime/ui/text.md docs/zircon_plugins/`
   - `git status --porcelain -- zircon_runtime/src/tests/ zircon_runtime/src/ui/text/`
3. 行号/事实重核（行号漂移则以重核结果为准并回写本计划）：
   - `grep -n "winit\|notify\|wgpu\|naga\|glam" Cargo.toml`（核 L37/L27/L36/L26/L23）
   - `grep -n "glyphon\|fontsdf\|unicode-segmentation\|zstd\|zr_vm\|jolt" zircon_runtime/Cargo.toml`（核 L77/L78/L95/L100/L103-104/L18）
   - `grep -n "fontdue\|softbuffer\|winit\|resvg" zircon_editor/Cargo.toml`（核 L11/L19/L23/L16）
   - `grep -n "wgpu\|winit" zircon_runtime_interface/Cargo.toml`（应 0 命中）
   - `grep -n "jolt" zircon_plugins/physics/runtime/Cargo.toml zircon_plugins/physics/runtime/src/manager.rs`
   - `grep -n "is not connected to layout yet" zircon_runtime/src/ui/text/shaper.rs`
4. §1.1 核对表逐项复核：通读 `docs/plans/zircon_runtime/runtime/index.md` §1.1，对每行"证据"列跑一次对应 grep（本次细化仅验证了五项失实库缺席）。
5. 基线记录：在干净工作区跑一次 `cargo check -p zircon_runtime --lib --locked`，把耗时记入"状态与产出记录"。

## 里程碑

### M1 选型文档与版本策略定稿（纯文档 + 守卫）

#### 切片 1.1 权威技术选型文档

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`（新建；2026-06-12 已核验该目录现有 21 个文档，无 tech-stack/选型类重名）；`docs/engine-architecture/index.md`（挂接一行链接）。
- 改动形态：新增文档，含权威依赖矩阵（列：库 / 版本 / owner crate / feature 门 / 替换条件 / 升级 gate），逐项矫正 §1.1 五处失实（cosmic-text、kira、zip/tar、rfd、arboard 均不在仓内），并收录 §1.2"声称未列但实际承重"的依赖（rayon、libloading、zstd、accesskit、taffy 等）。无代码改动。
- 调用方迁移：无代码调用方；文档入口在 `docs/engine-architecture/index.md` 增链接。
- 验收：`runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index`（归属切片 1.4 新建的 `zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`）——断言新文档文件存在、`index.md` 文本含 `runtime-tech-stack`。
- DoD：`test -f docs/engine-architecture/runtime-tech-stack.md && grep -q runtime-tech-stack docs/engine-architecture/index.md` 均真。

#### 切片 1.2 winit / notify 预发布版本锁定策略

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`（"预发布版本治理"章节）；根 `Cargo.toml` 只读核验（L37/L27），本切片不 bump 版本。
- 改动形态：决策记录——winit 锁定 `0.31.0-beta.2`，升级 gate = 0.31 final 发布 + `ApplicationHandler` API 无破坏性变更确认；notify 锁定 `9.0.0-rc.3`，gate = 9.0 final。升级窗口到来时必须独立成里程碑（见"风险与协调"）。
- 调用方迁移：无。
- 验收：`runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate`（tech_stack_dependency_guard.rs）——断言根 `Cargo.toml` 含 `0.31.0-beta.2` 与 `9.0.0-rc.3` 字面，使任何 silent bump 必须连同守卫与决策文档一起改。
- DoD：守卫断言版本字面与文档 gate 条目一一对应且测试通过。

#### 切片 1.3 zr_vm 仓外路径依赖治理决策

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`（"仓外路径依赖"章节）。
- 改动形态：决策记录三选一并给成本表——A 保持 `../../zr_vm` 外部 checkout（文档化目录布局与 clone-即建说明；optional + `backend-zr-vm` 门控已缓解）；B 迁入 `zircon_plugins` workspace；C git submodule。本里程碑只记录决策，不动 `zircon_runtime/Cargo.toml`。
- 调用方迁移：无。
- 验收：`zr_vm_path_dependency_gate_is_documented_with_version_pairing`（tech_stack_dependency_guard.rs）——断言 `zircon_runtime/Cargo.toml` 中 `zr_vm_rust_binding` 仍是 optional 外部路径依赖，且 `[features]` 含 `backend-zr-vm`；文档同步记录空参数导出调用的 binding 版本配对 gate。
- DoD：决策记录含目录布局 + 三方案成本表，守卫通过。

#### 切片 1.4 依赖守卫源断言测试（口径修正版）

- 目标文件：`zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`（新建）；`zircon_runtime/src/tests/extensions/mod.rs`（追加一行 `mod tech_stack_dependency_guard;`，现有四行 mod 声明保持）。
- 口径修正（2026-06-12 重核）：原计划"锁 zircon_editor 不出现 winit"会立即失败（`zircon_editor/Cargo.toml:23` 直依 winit）。修正为：wgpu 锁 `zircon_runtime_interface` + `zircon_editor` 双 crate；winit 仅锁 `zircon_runtime_interface`。zircon_editor 的 winit 直依处置作为决策条目记入选型文档并转 editor 计划 backlog（与切片 2.3 同一决策面）。
- 改动形态：新增测试函数（签名草案，执行时定稿）；实现模式照搬 `zircon_runtime/src/tests/extensions/animation_physics_absorption.rs:1-13` 的跨 crate `std::fs::read_to_string`（经 `CARGO_MANIFEST_DIR` 上行至 repo root）守卫惯例：

  ```rust
  #[test]
  fn runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate() { /* 断言根 Cargo.toml 含两预发布版本字面 */ }
  #[test]
fn zr_vm_path_dependency_gate_is_documented_with_version_pairing() { /* 断言 optional = true、backend-zr-vm 与 binding 版本配对 gate */ }
  #[test]
  fn interface_and_editor_dependency_boundaries_stay_documented_and_guarded() { /* 断言 interface 不含 wgpu/winit，editor 不含 wgpu 且记录 winit 直依现状 */ }
  #[test]
  fn removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack() { /* 断言 cosmic-text/kira/rfd/arboard/zip/tar 不静默进 manifest */ }
  #[test]
  fn runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index() { /* 断言文档存在且 index.md 挂接 */ }
  ```

- 调用方迁移：无（纯新增测试，不动任何生产代码与公共面）。
- 验收：上列 5 个测试，每条断言带失败信息说明被守卫的 owner 边界（参照 `feature_manifest.rs`、`graphics/tests/boundary.rs` 的字符串断言风格）。
- DoD：`cargo test -p zircon_runtime --lib tech_stack --locked` 全绿。

#### M1 测试阶段（milestone-first）

- 切片期仅轻量确认：`cargo check -p zircon_runtime --lib --locked`。
- 里程碑末：
  - `cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture`
  - `cargo test -p zircon_runtime --lib extensions --locked`（extensions 守卫树无回归）
- 验收证据：守卫通过；选型文档与 §1.1 核对表逐项一致；`docs/engine-architecture/index.md` 挂接完成。

### M2 文本栈职责定稿

#### 切片 2.1 三层职责矩阵

- 目标文件：`docs/zircon_runtime/ui/text.md`（更新；2026-06-12 核验该文档已存在且含 `UiTextShaper` boundary 章节——原计划"写入"按既有文件增量章节执行，非新建）。
- 改动形态：新增"后端职责矩阵"章节，三层各列输入/输出/owner 模块：
  - 层 1 shaping/分段：自研 shaper + unicode-segmentation `1.13.2`；owner `zircon_runtime/src/ui/text/{shaper.rs,grapheme.rs,layout_engine.rs}`；公共入口仅 `layout_text`（其余 `pub(crate)`）。
  - 层 2 光栅/SDF：fontsdf `0.5.3`（runtime）对 fontdue `0.9.3`（editor-only，softbuffer 自绘栈）。
  - 层 3 GPU 提交：glyphon `0.11.0`——必须标注双状态：native render/backend intent 仍归 glyphon；Native/SDF layout metrics 现在统一走 `SharedTextService`，直到后续文本里程碑替换 `UiTextShaper` 实现。
- 调用方迁移：无代码改动。glyphon 在 `zircon_runtime/src` 的 6 个引用文件全列（≤10）：`rhi_wgpu/ui_surface/text.rs`、`rhi_wgpu/ui_surface/geometry.rs`、`graphics/scene/scene_renderer/ui/text.rs`、`ui/surface/render/resolve.rs`、`ui/text/shaper.rs`、`ui/tests/text_shaper.rs`。
- 验收：复用既有测试为现状锚——`text_shaper_stack_uses_shared_text_service_for_font_backends` 与 `shared_text_shaper_matches_public_layout_entrypoint`（均在 `zircon_runtime/src/ui/tests/text_shaper.rs`）必须与矩阵 "`SharedTextService` active backend" 口径一致；矩阵文档显式引用这两个测试名。
- DoD：`text.md` 含三层矩阵，且 GPU/native text submission 行同时说明 glyphon native render/backend intent 与 `SharedTextService` layout metrics。

#### 切片 2.2 cosmic-text（及 parley/swash/harfbuzz）评估决策

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`（文本栈章节）；`docs/zircon_runtime/ui/text.md`（交叉引用一行）。
- 改动形态：决策记录——默认"不引入"；引入触发条件 = BiDi/连字/复杂文种成为真实需求；替换面固定 = 以 `UiTextShaper` trait（shaper.rs:34-37，`shape_text`/`measure_text` 两操作）实现替换 shaper 层，不替 glyphon GPU 提交。注意：`text.md` 既有口径写的是 Parley/Swash/HarfBuzz 候选，本切片把候选清单（含 cosmic-text）统一收口到选型文档，消除两处口径分叉。对照：`dev/bevy/crates/bevy_text/src/lib.rs`（cosmic-text 方案）。
- 调用方迁移：无。
- 验收：决策记录含"不引入"判词、触发条件、替换面三要素；`text.md` 不再保留与选型文档冲突的候选口径。
- DoD：`runtime-tech-stack.md` 文本栈章节含显式"不引入 cosmic-text（触发条件除外）"条目且 `text.md` 交叉引用指向它。

#### 切片 2.3 fontdue 留任裁决

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`；若裁决"待移除"则 backlog 条目落 `docs/editor-and-tooling/index.md`（执行时核验挂接位置：`grep -n -i backlog docs/editor-and-tooling/index.md`，无既有 backlog 段则新增段落）。
- 改动形态：决策记录——若 editor 文本最终走 runtime UI 链路（glyphon/SDF）渲染，fontdue `0.9.3`（`zircon_editor/Cargo.toml:11`）标记为待移除项并入 editor 计划；本计划不动任何 manifest。连带记录同一决策面：editor 的 winit 直依（L23）与 softbuffer（L19）自绘栈整体归属。
- 调用方迁移：无（fontdue 调用方枚举归 editor 计划执行期：Grep 模式 `fontdue::`，path `zircon_editor/src`）。
- 验收：M1 守卫 `fontdue_and_text_raster_stack_stay_out_of_runtime_manifest` 持续通过——防止裁决落地前 fontdue 被误迁入 runtime。
- DoD：裁决条目落地（留任或待移除二选一，无"待定"），守卫通过。

#### M2 测试阶段（milestone-first）

- 切片期仅轻量确认：`cargo check -p zircon_runtime --lib --locked`。
- 里程碑末：
  - `cargo test -p zircon_runtime --lib text_shaper --locked -- --nocapture`（现有文本测试无回归；过滤词由原计划的 `ui::text` 修正为 `text_shaper`，与既有测试模块路径 `ui::tests::text_shaper` 匹配更精确）
  - `cargo test -p zircon_runtime --lib tech_stack --locked`
- 验收证据：职责矩阵文档落地；无未决归属的文本相关依赖（glyphon/fontsdf/fontdue/unicode-segmentation 四库各有 owner 行）。

### M3 完备性缺口决策（物理 / 归档 / 编辑器辅助）

#### 切片 3.1 物理选型 spike（基线修正版）

- 当前落地补记（2026-07-10）：选型已从 spike 进入 Plugins 03 M1-T3 实现；本节后续条目保留为历史设计输入，当前事实以 `docs/zircon_plugins/physics-plugin-options.md` 和 Runtime 01 编号产出记录为准。

- 目标文件：`docs/zircon_plugins/physics-plugin-options.md`（新建，格式对齐 `docs/zircon_plugins/rendering-plugin-options.md`）；`docs/zircon_plugins/physics/runtime.md`（交叉引用更新）。
- 基线矫正（2026-06-12 重核）：spike 出发点是"已有自研最小刚体/查询雏形"，不是"从零"——`zircon_plugins/physics` 现有 37 文件 / 4353 行（manager 7 文件 859 行、query_contact raycast/overlap/contact/filter/geometry、trigger、scene_hook、1707 行契约测试）。
- 改动形态：决策记录三方案对比表（方案/理由/参考引擎对照/接入边界/回退条件）：A jolt-rust 绑定填入既有 jolt 槽位（`backend.rs:5-10`：`JOLT_ENABLED = cfg!(feature = "jolt")`、`JOLT_BACKEND_AVAILABLE = false`）；B rapier；C 扩展自研 builtin_step。接入边界固定不变：`core::framework::physics` 契约 + `zircon_plugins/physics` 实现，runtime 不直依物理库（既有守卫 `physics_domain_keeps_framework_contract_and_plugin_owns_runtime_behavior` 已断言 runtime manifest 不含插件 crate）。决策输出必须裁决两处 `jolt = []` 空 feature 去留（`zircon_runtime/Cargo.toml:18`、`zircon_plugins/physics/runtime/Cargo.toml:10`）：若弃 jolt 方案则两处删除——硬切换，不留 alias feature；删除落地归实现计划。
- 参考对照（一行一点）：Fyrox = rapier 外挂（`dev/Fyrox/fyrox-impl/Cargo.toml:30-31`）；Godot = 自研 + Jolt 双后端（`dev/godot/modules/{godot_physics_3d,jolt_physics}`）；Bevy = 核心无物理、生态 rapier/avian 外挂（`dev/bevy/crates` 无 physics crate）。
- 调用方迁移：无代码。物理契约调用面枚举归执行期：Grep 模式 `framework::physics`，path `zircon_runtime/src`。
- 验收：既有锚点测试继续通过并被决策记录引用为现状锚——`empty_jolt_feature_slot_reports_unavailable_not_ready`、`unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick`（`zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/`）。
- DoD：`physics-plugin-options.md` 含三方案五列表 + jolt 空 feature 裁决条目，且 `runtime.md` 交叉引用存在。

#### 切片 3.2 导出归档决策

- 目标文件：`docs/engine-architecture/runtime-editor-pluginized-export.md`（更新，2026-06-12 核验存在）或 `runtime-tech-stack.md` 归档章节——执行时定稿单一落点，禁止双写。
- 改动形态：决策记录——`ExportPackagingStrategy`（`export_profile.rs:115-121` 三变体）的归档实现三选一：zip crate / tar + 既有 zstd `0.13.3` / 自定容器；评估列含格式兼容、压缩率、流式读取、跨平台路径语义。决策不改变枚举形状与 `default_export_strategies()` 默认值（L188-193，`[SourceTemplate, LibraryEmbed]`）；落地实现归 export_build_plan owner（`zircon_runtime/src/plugin/export_build_plan/`）。
- 调用方迁移：`ExportPackagingStrategy` 全仓引用 76 个代码文件 / 386 处（>10）：代表路径 `zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs`、`zircon_runtime/src/plugin/runtime_plugin/descriptor.rs`、`zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs`；枚举命令：Grep 模式 `ExportPackagingStrategy`，glob `**/*.rs`。本切片决策零迁移；若未来新增变体归 owner 实现计划。
- 验收：决策记录含三方案对比表 + 与默认策略集的兼容声明 + owner 计划落地条目链接；ZIP materialization 落地后由 `export_archive_policy_allows_zip_only_for_archive_materializer` 断言 `zip` 只允许 runtime archive materializer 使用，且 `tar` 仍未进入 manifests。
- DoD：决策记录落地于单一文档且含 owner 计划链接；`tech_stack_dependency_guard.rs` 的归档策略守卫独立通过。该切片允许同步更新守卫源文件，不再是纯 docs-only 变更。

#### 切片 3.3 rfd / arboard 归属裁决

- 目标文件：`docs/engine-architecture/runtime-tech-stack.md`（"明确不在 runtime 栈"清单）；`docs/editor-and-tooling/index.md` 或独立 backlog 文档（执行时定稿，与切片 2.3 的 backlog 落点保持同处）。
- 改动形态：决策记录——rfd（文件对话框）/arboard（剪贴板）确认为 `zircon_editor` 需求，移出 runtime 声称栈；editor backlog 条目含需求场景、候选版本、预期接入位置（`zircon_editor/src/ui/host` 一带，执行时核验：`ls zircon_editor/src/ui/host`）。
- 调用方迁移：无（两库全仓 0 命中，grep 已证）。
- 验收：`runtime-tech-stack.md` 含"editor-only 候选"清单且明确 runtime 不引入两库。
- DoD：两库在 `runtime-tech-stack.md` 标注 editor-only 且 backlog 条目链接存在。

#### M3 测试阶段（milestone-first）

- 本里程碑以决策记录为产物，无新代码。切片期与里程碑末：`git status --porcelain` 确认仅 `docs/` 变更；`cargo check -p zircon_runtime --lib --locked` 确认无意外代码漂移。
- 既有物理契约锚点回归确认（可选）：`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked`
- 验收证据：三份决策记录（方案、理由、参考引擎对照、接入边界、回退条件）齐备；`docs/zircon_plugins/physics-plugin-options.md` 与 `rendering-plugin-options.md` 同格式。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`01/2026-07-09-tech-stack-and-dependency-governance-output-records.md`](01/2026-07-09-tech-stack-and-dependency-governance-output-records.md)
- fixed 已修复：[wgpu-hal-windows-version-split](../../zircon_editor/editor/11/fixed-2026-07-12-wgpu-hal-windows-version-split.md)
- fixed 已修复：[wsl-vhdx-sharing-violation](../../zircon_editor/editor/11/fixed-2026-07-11-wsl-vhdx-sharing-violation.md)
- 失败交接（`open / 待恢复受管验证空间`）：[`01/failure-2026-07-11-editor-libtest-link-disk-space.md`](01/failure-2026-07-11-editor-libtest-link-disk-space.md)
- 当前状态（2026-07-11）：原计划声明的五项 locked Cargo 门禁曾在当时源码上闭合；当前依赖图重新出现 `wgpu-hal`/Windows 类型分裂，Editor lib-test 的受管链接空间也待恢复，因此 Runtime 01 重新进入 `in_progress`，以编号失败交接为当前事实。WSL 虚拟磁盘冲突已由环境释放并回传 fixed。
