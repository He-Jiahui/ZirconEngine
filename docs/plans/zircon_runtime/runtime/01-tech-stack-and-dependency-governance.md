---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src/backend.rs
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
status: in_progress
last_refined: 2026-06-21
---

# 01 技术选型与依赖治理

## 现状与证据（2026-06-12 重核）

- 声称栈 5 处失实（cosmic-text、kira、zip/tar、rfd、arboard）：五库在全仓任何 `Cargo.toml` 均 0 命中（2026-06-12 grep 重核属实）。核对表全文见 `docs/plans/zircon_runtime/runtime/index.md` §1.1；本次细化只复核了五库缺席这一项，§1.1 其余行执行时逐项核验（命令见"执行前检查清单"）。
- 文本栈三库并存且口径未定：glyphon `0.11.0`（`zircon_runtime/Cargo.toml:77`）、fontsdf `0.5.3`（L78）、unicode-segmentation `1.13.2`（L95）；fontdue `0.9.3` 仅在 editor（`zircon_editor/Cargo.toml:11`）。自研 text shaper / hit-testing 在 `zircon_runtime/src/ui/text/`（mod.rs、shaper.rs、hit_test.rs、layout_engine.rs 约 25.8KB、edit_state.rs、grapheme.rs、rich_text.rs）。
- glyphon 口径矫正（2026-06-12 重核）：原文"glyphon 承担 runtime GPU 文本渲染"只对了一半——渲染侧确有 6 个文件引用 glyphon，但 shaping/layout 侧未接通：`shaper.rs:99-106` 的 `active_layout_backend_for_intent` 对 `NativeGlyphon`/`SdfAtlas` 一律回退 `Heuristic`，fallback_reason 源码注明 `"glyphon native text backend is not connected to layout yet"`（shaper.rs:108-124），且测试 `text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land` 锁定该现状。
- 公共面注意（2026-06-12 重核）：`ui/text` 对外仅 `pub use shaper::layout_text`（`layout_text(text, style, frame, clip_frame) -> UiResolvedTextLayout`，shaper.rs:196-203）；`UiTextShaper` trait（shaper.rs:34-37）、`hit_test_text_layout`、`UiTextHitTest` 等均 `pub(crate)`。文档示例不得引用不出 crate 的类型。
- 版本风险：winit `0.31.0-beta.2`（根 `Cargo.toml:37`，default-features = false）、notify `9.0.0-rc.3`（L27）。同文件 wgpu `29.0.1`（L36）、naga `29.0.1`（L26）、glam `0.32.1`（L23）。
- `zr_vm_rust_binding` / `zr_vm_rust_binding_sys` 是指向仓库外 `../../zr_vm/...` 的路径依赖（`zircon_runtime/Cargo.toml:103-104`，optional），由 feature `zr-vm-real-backend`（L26）门控。2026-06-12 的 plugin lifecycle 修复已在 `../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs` 落地空参数导出调用 marshalling 防御，当前 `zr-vm-real-backend` 验证必须与这份本地 binding 修复配对。
- 物理现状矫正（2026-06-12 重核）：原文"`zircon_plugins/physics/runtime` 为空壳"已过时。该插件现有 37 个文件 / 4353 行 Rust：自研 builtin 物理（manager/ 7 文件 859 行：builtin_step、clock、query、service、settings、validation、world_sync）、query_contact（raycast aabb/capsule/sphere、overlap、contact/filter/geometry）、trigger、scene_hook，外加 1707 行 `physics_manager_runtime_contract` 集成测试。"无任何物理依赖"仍属实：`zircon_plugins/physics/runtime/Cargo.toml` 仅依赖 `zircon_runtime`。jolt 空 feature 有两处：`zircon_runtime/Cargo.toml:18` 与 `zircon_plugins/physics/runtime/Cargo.toml:10`（原文漏列后者）；`backend.rs:5-10` 中 `JOLT_ENABLED = cfg!(feature = "jolt")` 而 `JOLT_BACKEND_AVAILABLE = false` 硬编码——jolt 是"可声明但永不可用"的后端槽位。
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
   - `grep -n "jolt" zircon_plugins/physics/runtime/Cargo.toml zircon_plugins/physics/runtime/src/backend.rs`
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
- 改动形态：决策记录三选一并给成本表——A 保持 `../../zr_vm` 外部 checkout（文档化目录布局与 clone-即建说明；optional + `zr-vm-real-backend` 门控已缓解）；B 迁入 `zircon_plugins` workspace；C git submodule。本里程碑只记录决策，不动 `zircon_runtime/Cargo.toml:103-104`。
- 调用方迁移：无。
- 验收：`zr_vm_path_dependency_gate_is_documented_with_version_pairing`（tech_stack_dependency_guard.rs）——断言 `zircon_runtime/Cargo.toml` 中 `zr_vm_rust_binding` 仍是 optional 外部路径依赖，且 `[features]` 含 `zr-vm-real-backend`；文档同步记录空参数导出调用的 binding 版本配对 gate。
- DoD：决策记录含目录布局 + 三方案成本表，守卫通过。

#### 切片 1.4 依赖守卫源断言测试（口径修正版）

- 目标文件：`zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`（新建）；`zircon_runtime/src/tests/extensions/mod.rs`（追加一行 `mod tech_stack_dependency_guard;`，现有四行 mod 声明保持）。
- 口径修正（2026-06-12 重核）：原计划"锁 zircon_editor 不出现 winit"会立即失败（`zircon_editor/Cargo.toml:23` 直依 winit）。修正为：wgpu 锁 `zircon_runtime_interface` + `zircon_editor` 双 crate；winit 仅锁 `zircon_runtime_interface`。zircon_editor 的 winit 直依处置作为决策条目记入选型文档并转 editor 计划 backlog（与切片 2.3 同一决策面）。
- 改动形态：新增测试函数（签名草案，执行时定稿）；实现模式照搬 `zircon_runtime/src/tests/extensions/animation_physics_absorption.rs:1-13` 的跨 crate `std::fs::read_to_string`（经 `CARGO_MANIFEST_DIR` 上行至 repo root）守卫惯例：

  ```rust
  #[test]
  fn runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate() { /* 断言根 Cargo.toml 含两预发布版本字面 */ }
  #[test]
  fn zr_vm_path_dependency_gate_is_documented_with_version_pairing() { /* 断言 optional = true、zr-vm-real-backend 与 binding 版本配对 gate */ }
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
  - 层 3 GPU 提交：glyphon `0.11.0`——必须标注双状态：渲染侧已用 / layout 后端未接（`active_layout_backend_for_intent` 回退 Heuristic，引用 shaper.rs 的 fallback_reason 源串）。
- 调用方迁移：无代码改动。glyphon 在 `zircon_runtime/src` 的 6 个引用文件全列（≤10）：`rhi_wgpu/ui_surface/text.rs`、`rhi_wgpu/ui_surface/geometry.rs`、`graphics/scene/scene_renderer/ui/text.rs`、`ui/surface/render/resolve.rs`、`ui/text/shaper.rs`、`ui/tests/text_shaper.rs`。
- 验收：复用既有测试为现状锚——`text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land` 与 `heuristic_text_shaper_matches_public_layout_entrypoint`（均在 `zircon_runtime/src/ui/tests/text_shaper.rs`）必须与矩阵"未接通"口径一致；矩阵文档显式引用这两个测试名。
- DoD：`text.md` 含三层矩阵且 glyphon 行同时出现"渲染侧已用"与"layout 后端未接"两个状态词。

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

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 选型文档 | 独立守卫通过；Cargo 待共享测试窗口 | 2026-06-12 | `docs/engine-architecture/runtime-tech-stack.md`；`docs/engine-architecture/index.md`；`runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index` |
| M1 | 1.2 winit/notify 策略 | 独立守卫通过；Cargo 待共享测试窗口 | 2026-06-12 | `runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate` |
| M1 | 1.3 zr_vm 治理决策 | 独立守卫通过；Cargo 待共享测试窗口 | 2026-06-12 | `zr_vm_path_dependency_gate_is_documented_with_version_pairing` |
| M1 | 1.4 依赖守卫测试 | code_complete_static_passed；Cargo 待 plugin bridge 切片稳定 | 2026-06-12 | `zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`；`rustfmt --edition 2021`；standalone `rustc --edition 2021 --test ...` 10/10 passed；earlier Cargo attempt failed before tech_stack tests on `ecs_schedule.rs` / `ecs_scheduled_native_systems.rs` missing `Runtime` match arms, and those non-exhaustive match test sites are now patched locally；2026-06-13 rerun `cargo test -p zircon_runtime --lib tech_stack --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-01-tech-stack-0612 --message-format short --color never -- --nocapture` failed before tech_stack tests because active plugin bridge work left `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` importing unresolved `crate::plugin::{BridgeInterfaceSnapshot, BridgeInterfaceStatus}` |
| M2 | 2.1 三层职责矩阵 | 文档与独立守卫完成，Cargo 待共享测试窗口 | 2026-06-12 | `docs/zircon_runtime/ui/text.md` 新增 Backend Responsibility Matrix，分出 shaping/layout、font/raster/SDF、GPU/native submission 三层；glyphon 行同时标注“渲染侧已用”与“layout 后端未接”；新增 `runtime_text_doc_records_three_layer_stack_and_cross_reference`；standalone `rustc --edition 2021 --test zircon_runtime\src\tests\extensions\tech_stack_dependency_guard.rs` 通过 10/10 |
| M2 | 2.2 cosmic-text 决策 | 决策与交叉引用守卫完成，Cargo 待共享测试窗口 | 2026-06-12 | `runtime-tech-stack.md` 已裁决 cosmic-text/Parley/Swash/HarfBuzz 只能通过 `UiTextShaper` 替换实现进入；`docs/zircon_runtime/ui/text.md` 已交叉引用 runtime tech-stack text boundary；新增 `complex_text_backends_can_only_enter_through_ui_text_shaper` 与 `runtime_text_doc_records_three_layer_stack_and_cross_reference`；standalone `rustc --edition 2021 --test zircon_runtime\src\tests\extensions\tech_stack_dependency_guard.rs` 通过 10/10 |
| M2 | 2.3 fontdue 裁决 | 决策与守卫完成；Cargo 待共享测试窗口 | 2026-06-12 | `fontdue` 裁决为 `zircon_editor` retained-host 临时文本 fallback，不属于 runtime 技术栈；待 editor UI text 计划把 retained-host 文本迁到 runtime UI text/glyphon/SDF 后移除；新增 `fontdue_editor_retained_host_dependency_has_migration_owner` |
| M3 | 3.1 物理选型 spike | 决策与守卫完成；Cargo 待共享测试窗口 | 2026-06-12 | 新增 `docs/zircon_plugins/physics-plugin-options.md`，并从 `docs/zircon_plugins/physics/runtime.md` 交叉引用；裁决 builtin 为 V1 唯一可执行后端，Jolt 为未来 native 方向但保持 unavailable，Rapier 不进入主路径；新增 `physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned`，守卫两处 `jolt = []` 空 feature、`JOLT_BACKEND_AVAILABLE = false`、Jolt 不降级 builtin、Rapier/Avian 不进 manifest、物理 backend 仍由插件持有 |
| M3 | 3.2 导出归档决策 | 决策与守卫完成；Cargo 待共享测试窗口 | 2026-06-12 | `docs/engine-architecture/runtime-tech-stack.md` 写入 Export Archive Decision：当前目录优先，未来桌面/editor 归档容器选 ZIP，拒绝 V1 自定义容器；守卫后续由 `export_archive_policy_allows_zip_only_for_archive_materializer` 接管，只允许 runtime archive materializer 使用 zip |
| M3 | 3.2a export build-plan directory materialization boundary | export_materialize_owner_split_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `zircon_runtime/src/plugin/export_build_plan/materialize.rs` 硬切到 `materialize/{mod,generated,paths,native,package_lookup,copy,report}.rs`；公开 `ExportBuildPlan::write_generated_files` / `materialize` / `materialize_with_native_packages` 与 `ExportMaterializeReport` shape 不变。`materialize/paths.rs` 现在在目录式写入前拒绝 empty、absolute/root/prefix、`.`、`..`、trailing separator 与 backslash 生成文件路径，先补齐未来归档实现要求的 path traversal guard；本切片不引入 `zip` / `tar` 依赖、不声明 archive materialization 完成。验证：rustfmt check、旧 `materialize.rs` absent、conflict-marker scan、stale old-path scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2b NativeDynamic materialization symlink boundary | export_materialize_symlink_boundary_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `materialize/package_lookup.rs` 现在只遍历真实目录、只读取真实 `plugin.toml` 文件，避免 symlinked package root 或 manifest 作为 NativeDynamic source；`materialize/copy.rs` 跳过 symlinked package top-level payload、resource children 与 native artifact entries，顶层跳过项进入 materialization diagnostics。公开 API 与报告 shape 不变，目录式 materialization 继续不引入 `zip` / `tar`。验证：rustfmt check、non-following helper/source scan、conflict-marker scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2c export materialization dry-run preview | export_materialize_preview_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | 新增 `ExportBuildPlan::preview_materialize(...)` 与 `preview_materialize_with_native_packages(...)`，复用 `ExportMaterializeReport` 返回 planned generated file paths、planned copied package directories、plan diagnostics 与 fatal diagnostics；preview 复用同一 generated path resolver、NativeDynamic package lookup、duplicate output-directory、native artifact 和 symlink diagnostics，但不创建目录、不写文件、不复制 payload、不写 package report。验证：rustfmt check、preview helper scan、write/copy call-site scan（确认 filesystem writes/copies 仍只在 mutating leaves）、conflict-marker scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2d export materialization fatal preflight gate | export_materialize_fatal_gate_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `materialize/mod.rs` 现在在 mutating materialization 前计算 `effective_fatal_diagnostics()`；fatal plan 返回空 `generated_files` / `copied_packages`、保留 fatal diagnostics、追加 materialization-blocked diagnostic，并跳过 NativeDynamic package copy。直接 `write_generated_files(...)` 在 fatal plan 下 no-op，preview 仍保留 no-write planned path 语义。验证：rustfmt check、fatal gate source scan、write/copy call-site scan、conflict-marker scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2e editor native-aware fatal export early exit | editor_native_aware_export_fatal_gate_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs` 现在在 native-aware export plan 生成后、`prepare-native-packages` 前检查 `plan.has_fatal_diagnostics()`；fatal plan 复用 runtime no-op `materialize(...)` report，写出 diagnostics，并返回空 generated/copied/native-cargo/source-cargo 结果，避免 fatal export 仍触发 NativeDynamic staging/build。验证：rustfmt check、fatal early-exit source scan、conflict-marker scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2f editor native-aware discovery reuse | editor_native_aware_export_discovery_reuse_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `manifest_completion/native.rs` 新增 host-internal `complete_project_plugin_manifest_with_native_report(...)`，`export_build/manager.rs` 在执行 native-aware export 时复用初始 `NativePluginLoadReport` 生成 plan 并传给 package preparation，避免同一执行路径重复扫描 plugin directory。公开 `generate_native_aware_export_plan(...)` 与 project/plugin projection 行为不变。验证：rustfmt check、helper/discovery call-site scan、conflict-marker scan、trailing-whitespace scan、path-scoped `git diff --check`；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| M3 | 3.2g export ZIP archive materialization | export_archive_zip_materialization_static_passed_cargo_deferred_tests_deferred | 2026-06-20 | `zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs` 新增 `ExportBuildPlan::materialize_zip_archive(...)` / `preview_zip_archive(...)`，把 generated files、NativeDynamic runtime payload 与 `native_dynamic_package.toml` 写入单个 ZIP；`ExportMaterializeReport` 新增 `archive_file`，directory materialization 保持 `None`。`zircon_runtime/Cargo.toml` 仅为该 materializer 引入 `zip = { version = "9.0.0-pre2", default-features = false, features = ["deflate-flate2"] }`，`tar` 仍缺席；entry 名复用 `validated_materialized_relative_path(...)`，generated entries 排序，ZIP 时间戳/权限稳定，fatal plan 不创建 archive，preview 不落盘。测试锚新增 `native_dynamic_zip_archive_materialization_writes_generated_files_and_runtime_payloads`、`native_dynamic_zip_archive_preview_reports_archive_without_writes`，并扩展 missing-required fatal plan 对 archive blocked report 的覆盖；Cargo 与 focused behavior tests 按“先实现功能”方向暂缓。 |
| 横切 | Runtime 01 export fatal status-output coverage sync | export_fatal_status_output_coverage_static_passed | 2026-06-20 | `status_output_tables/expected_slices.rs` 现在覆盖总索引中的 `Runtime 01 export materialization fatal preflight gate`、`Runtime 01 editor native-aware fatal export early exit` 与 `Runtime 01 editor native-aware discovery reuse` 三条 2026-06-20 export 状态行，锁定各自 status/date；`expected_status_row_data/runtime_01_04.rs` 同步 `effective_fatal_diagnostics()`、`materialization-blocked diagnostic`、`plan.has_fatal_diagnostics()`、native staging/build early-exit 与 `complete_project_plugin_manifest_with_native_report(...)` 证据锚。同步将本计划 `last_refined` 提升到 `2026-06-20`，并在 runtime index 的 Runtime 01 子计划行恢复 `tech_stack/text_shaper/plugin physics Cargo gates` 锚点、单列 `export_build_plan Cargo gate` 待验证，满足 `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation`。验证：`rustfmt --edition 2021 --check` 覆盖 touched status-output expectation 文件；standalone `plan_status` status-output filter 2/2 通过；完整 standalone `plan_status` 首次复验暴露上述 Runtime 01 元数据漂移，修正后待复验；本切片只同步状态输出/元数据守卫，不启动 Cargo、不提升 Runtime 01 export Cargo gates。 |
| M3 | 3.3 rfd/arboard 裁决 | 决策与守卫完成；Cargo 待共享测试窗口 | 2026-06-12 | 新增 `docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md` 并挂接 `docs/editor-and-tooling/index.md`；`rfd` = native file/folder dialog 候选，`arboard` = clipboard 候选，owner 均为 editor host/retained host，runtime/interface 不引入；新增 `editor_only_dependency_candidates_have_editor_backlog_owner` |
| 横切 | Tech-stack 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::tech_stack::runtime_01_tech_stack_mirror_docs_match_structure_audit_counts` 并在 `runtime_absorption/mod.rs` 挂接，锁定 Runtime 01 计划、runtime index、M0 review、runtime-interface convergence 与 `runtime-tech-stack.md` 必须同步 `tech_stack_boundary` 的 `expected_manifest_count = 5`、`expected_non_dependency_count = 5`、`zip_dependency_count = 1`、`expected_zip_dependency_count = 1`、`zip_dependency_violations = []`、`tech_stack_guard_count = 12`、`editor_only_candidate_count = 3`、`jolt_feature_slot_count = 2`、`declared_removed_dependencies = []`、`rapier_or_avian_dependencies = []`、`mirror_docs_guard_present = true` 与 `risks = []`。验证：rustfmt check、Python py_compile、direct `tech_stack_boundary_audit`、standalone rustc 1/1、stale old-count scan、scoped conflict/diff checks 通过（diff 仅 LF-to-CRLF warnings）；Cargo tech_stack/extensions/text_shaper/plugin physics gates 仍 pending。 |
| 横切 | Tech-stack 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `tech_stack_boundary` 与 `runtime_01_tech_stack_mirror_docs_match_structure_audit_counts` 现在锁定 Runtime 01 M2/M3 的 4 个现状验收锚：`heuristic_text_shaper_matches_public_layout_entrypoint`、`text_shaper_stack_uses_current_heuristic_backend_until_font_backends_land`、`empty_jolt_feature_slot_reports_unavailable_not_ready`、`unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick`；当前 `behavior_test_anchor_count = 4`、`missing_behavior_test_anchors = []`。Runtime 01、`runtime-tech-stack.md`、runtime index、M0 review、runtime-interface convergence 与状态输出表守卫已同步；验证：rustfmt check、Python py_compile、direct `tech_stack_boundary_audit`、aggregate Runtime 01 + plan-status assertions、standalone tech_stack 1/1、standalone status-output 2/2；tech_stack/extensions/text_shaper/plugin physics Cargo gates pending。 |
| 横切 | Tech-stack current audit recheck | tech_stack_current_audit_static_passed_cargo_pending | 2026-06-20 | 状态锚 `tech_stack_current_audit_static_passed_cargo_pending`；本轮只复核 Runtime 01 当前技术栈/依赖治理边界事实，生产代码未改：`tech_stack_boundary_audit` 报告 manifest files 5/5、corrected non-dependencies 5、zip dependency 1/1、tech-stack guard anchors 12/12、behavior-test anchors 4/4、editor-only candidate count 3、Jolt feature slots 2、`declared_removed_dependencies = []`、`rapier_or_avian_dependencies = []`、`mirror_docs_guard_present = true`、`risks = []`。验证：Python py_compile、direct `tech_stack_boundary_audit` risks=[]、standalone `tech_stack.rs` 1/1、standalone `tech_stack_dependency_guard.rs` 11/11、standalone `plan_status.rs` 32/32；tech_stack/extensions/text_shaper/plugin physics/export_build_plan Cargo gates 仍 pending。 |
| 横切 | Tech-stack inventory split | tech_stack_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `tech_stack_source_inventory.py` now owns Runtime 01 manifest inventory, dependency/version anchors, corrected non-dependency list, ZIP dependency policy line, and expected manifest/non-dependency/ZIP/editor-only candidate counts; `tech_stack_anchor_inventory.py` now owns text-stack doc anchors, tech-stack doc anchors, Rust/static guard anchors, behavior-test anchors, physics decision anchors, editor backlog anchors, mirror-doc guard, and pending Cargo gate anchors; `tech_stack_boundary.py` now remains the audit reader / dependency scanner / risk layer at 341 lines, while `tech_stack_markdown.py` owns the Markdown layer at 103 lines. Direct audit reports manifest files 5/5, corrected non-dependencies 5, ZIP dependency declarations 1/1, tech-stack guard anchors 12/12, behavior-test anchors 4/4, editor-only candidate count 3, Jolt feature slots 2, `declared_removed_dependencies = []`, `rapier_or_avian_dependencies = []`, `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile, direct `tech_stack_boundary_audit` risks=[], standalone `tech_stack.rs` 1/1, standalone `tech_stack_dependency_guard.rs` 11/11, standalone `plan_status.rs` 33/33; broader `tech_stack` / `extensions` / `text_shaper` / plugin physics / export_build_plan Cargo gates remain deferred while external compile lanes are active. |
| 横切 | Tech-stack Markdown renderer split | tech_stack_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | 状态锚 `tech_stack_markdown_split_static_passed_cargo_deferred_tests_deferred`；`tech_stack_markdown.py` now owns `render_tech_stack_boundary_markdown`, and `audit_runtime_structure.py` imports the renderer from that Markdown owner instead of `tech_stack_boundary.py`; `tech_stack_boundary.py` now owns audit read, dependency scanning, missing-anchor calculation, and risk aggregation at 341 lines, while the Markdown owner is 103 lines. Direct audit reports manifest files 5/5, corrected non-dependencies 5, ZIP dependency declarations 1/1, tech-stack guard anchors 12/12, behavior-test anchors 4/4, editor-only candidate count 3, Jolt feature slots 2, `declared_removed_dependencies = []`, `rapier_or_avian_dependencies = []`, `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile and direct `tech_stack_boundary_audit`; standalone `tech_stack.rs` 1/1, standalone `tech_stack_dependency_guard.rs` 11/11, and standalone `plan_status.rs` 33/33; broader `tech_stack` / `extensions` / `text_shaper` / plugin physics / export_build_plan Cargo gates remain deferred while external compile lanes are active. |

2026-06-13 状态复核：Runtime 01 M3 三个完备性缺口已由上述文档和 `tech_stack_dependency_guard.rs` 守卫静态闭合，runtime 总览 P10 已同步为已裁决状态；本次复核的锚点扫描覆盖 `physics-plugin-options.md`、`runtime-tech-stack.md`、`runtime-editor-only-dependency-backlog.md`、`tech_stack_dependency_guard.rs` 与三处索引交叉引用。07:41 追加物理后端选项守卫 `physics_backend_option_decision_keeps_jolt_unavailable_and_plugin_owned`，以源码/manifest/doc 三点锁定 Jolt 仍不可用且插件持有、builtin 为唯一 V1 可执行后端、Rapier/Avian 不进 manifest。Cargo 仍未重跑：当前有其他 active cargo/rustc lanes。

2026-06-13 16:20 状态复核：新增 `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation`，把 M1/M2/M3 的 `tech_stack` / `extensions` / `text_shaper` / plugin physics Cargo gates 统一锁定为 pending；该守卫要求 Runtime 01 保持 `in_progress`，并把 `runtime-tech-stack.md`、`text.md`、`physics-plugin-options.md`、editor-only backlog、runtime 总览 P10/子计划行与 M0 评审同步到同一待验证状态。16:28 之后尝试 `cargo test -p zircon_runtime --lib runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-01-plan-status-0613 --message-format short --color never -- --nocapture`，20 分钟超时且无通过/失败测试结果；残留的该目标目录 cargo/rustc 进程已停止。

2026-06-13 23:18 状态复核：新增 `tech_stack_boundary` 结构审计 owner 并接入 `audit_runtime_structure.py`；2026-06-21 inventory split 后 `tech_stack_source_inventory.py` 承接 manifest/dependency/version/count 清单，`tech_stack_anchor_inventory.py` 承接 doc/guard/behavior/decision/Cargo gate 锚点；2026-06-21 Markdown renderer split 后 `tech_stack_boundary.py` 保留 341 行审计读取、依赖扫描、缺失锚点计算与风险聚合，`tech_stack_markdown.py` 承接 103 行 Markdown 渲染。当前报告 `expected_manifest_count = 5`、`expected_non_dependency_count = 5`、`zip_dependency_count = 1`、`expected_zip_dependency_count = 1`、`zip_dependency_violations = []`、`tech_stack_guard_count = 12`、`behavior_test_anchor_count = 4`、`missing_behavior_test_anchors = []`、`editor_only_candidate_count = 3`、`jolt_feature_slot_count = 2`、`declared_removed_dependencies = []`、`rapier_or_avian_dependencies = []`、`mirror_docs_guard_present = true` 与 `risks = []`。这仍是静态结构证据；`tech_stack` / `extensions` / `text_shaper` / plugin physics / export_build_plan Cargo gates 继续待 active lanes 清空后补跑。

基线数值（开工首日记录，完工时复核漂移）：

- `cargo check -p zircon_runtime --lib --locked` 耗时基线：__（执行时填写）
- `cargo test -p zircon_runtime --lib tech_stack --locked` 通过数：未执行到 tech_stack 测试；2026-06-12 用 `--jobs 1 --target-dir D:\cargo-targets\zircon-runtime-01-tech-stack-0612` 尝试时，lib-test 编译先因 `zircon_runtime/src/scene/tests/ecs_schedule.rs` 与 `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs` 未匹配新增 `Runtime` 调度分支失败；这些非穷尽 match 测试位点已在本地补齐；2026-06-13 用同一目标目录重跑，lib-test 编译先因活跃 plugin bridge 切片中 `extension_registry_bridge.rs` unresolved imports `BridgeInterfaceSnapshot` / `BridgeInterfaceStatus` 失败；standalone guard 10 / 10；2026-06-13 16:28 +08 focused plan-status Cargo guard `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation` 用 `D:\cargo-targets\zircon-runtime-01-plan-status-0613` 目标目录运行 20 分钟超时，无通过结论
- §1.1 失实项矫正数：5 / 5（cosmic-text、kira、zip/tar、rfd、arboard）
- glyphon 引用文件数基线：6（漂移重核：Grep `glyphon`，path `zircon_runtime/src`）
- `ExportPackagingStrategy` 引用基线：76 代码文件 / 386 处（漂移重核：Grep `ExportPackagingStrategy`，glob `**/*.rs`）
- jolt 空 feature 位置基线：2 处（`zircon_runtime/Cargo.toml:18`、`zircon_plugins/physics/runtime/Cargo.toml:10`）

## 风险与协调

- 物理决策影响 `zircon_plugins` workspace 与 CI 矩阵，决策记录必须先过 `zr-architecture-first-engineering` 的深度测试再排实现计划。
- winit/notify 升级窗口到来时，升级切片必须独立成单独里程碑，禁止夹带进其他子计划；升级时必须同步修改 `runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate` 守卫的版本字面与选型文档 gate 条目，三者一次提交内闭合。
- editor 自绘栈（winit/softbuffer/fontdue）裁决与 editor 计划耦合：本计划只出决策记录，`zircon_editor/Cargo.toml` 改动一律归 editor 计划，避免双计划同改一份 manifest 冲突。
- M2 文本矩阵与未来"glyphon layout 接通"会话的冲突面集中在 `zircon_runtime/src/ui/text/shaper.rs` 与 `ui/tests/text_shaper.rs`：开工前按"执行前检查清单"核对脏文件与活动会话；若接通会话已启动，矩阵中"未接通"状态词以该会话产出为准重核。
- 渲染骨架相关内容（含 glyphon GPU 提交路径改造、wgpu/naga 版本节奏）一律归 render 计划 01-08，本计划文档只做现状引用不做规划。
- 本计划新增文档/测试不得引入非网络语义的 server 命名（blocker 级约束）；守卫与文档命名已按 tech_stack/dependency 口径规避。
- `zr_vm` 路径依赖治理不再只是 clone 布局问题：`zircon_runtime` 的真实 VM 生命周期依赖 binding 层把空参数导出调用表示为"合法非空指针 + len=0"。后续若切换到 submodule、vendor 或发布版本，必须把该修复作为版本配对 gate。
