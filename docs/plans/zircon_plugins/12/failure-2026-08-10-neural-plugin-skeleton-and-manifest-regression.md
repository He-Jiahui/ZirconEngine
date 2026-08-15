---
handoff_kind: failure
status: open
created_at: 2026-08-10
summary_slug: neural-plugin-skeleton-and-manifest-regression
origin_plan: docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/04
fixing_child_dir: docs/plans/zircon_plugins/12
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/neural/plugin.toml
  - zircon_plugins/neural/runtime
  - zircon_plugins/neural/editor
  - zircon_plugins/neural/features/post_process/runtime
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_app/Cargo.toml
  - tools/plugin_structure_audits/capability.py
  - tools/tests/test_plugin_structure_audit_capability.py
tests:
  - python tools/audit_plugin_structure.py --json
  - python -m unittest tools.tests.test_plugin_structure_audit_capability
  - cargo +1.94.1 test -p zircon_plugin_neural_runtime --lib --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_neural_post_process_runtime --lib --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_plugin_neural_editor --lib --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_first_party_runtime_catalog --features advanced-render-runtime-plugins --lib --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_first_party_editor_catalog --features neural-editor-plugin --lib --locked --jobs 1 -- --nocapture --test-threads=1
---

# Plugins12: neural plugin skeleton and manifest regression

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md`
- 来源执行切片：Frameworks04 first-party plugin structure audit gate
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：最低共享原因位于 Plugins12 所有的 first-party plugin skeleton、manifest 和 catalog projection 边界。

## 失败现象与复现证据

Frameworks04 的 first-party plugin 结构审计在当前源码发现 neural package 已进入 workspace，
但 optional feature 写入了 schema 不支持的 `maturity` 字段；editor 与
`features/post_process/runtime` 两个 crate 都缺少 `capability.rs`、`plugin.rs` 和注册 owner。
同时 neural 尚未进入 first-party runtime/editor catalog，因此即使单 crate 编译也仍是产品孤儿。

初始静态证据：39 个 manifest 中 `manifest_schema_violations = 1`，
`skeleton_migration_debt_count = 1`，共 8 条 neural skeleton 明细。初始 44 文件 ordinal
fingerprint 为 `3f4eca3db4b2282bb4522e9ac824024f6306208d90c71c6be2cf0575c31da746`。

## 最低共享层根因

neural 是在 Plugins12 已宣称 migration debt 清零后新增的 first-party package，却绕过了
runtime feature manifest 单一真相、editor declaration mirror 和 catalog projection 三个既有
边界。继续在 `lib.rs` 暴露设置类型不能形成可选择、可验证、可装配的 plugin package。

## 架构修复验收

- root runtime 独占 `neural.post_process` manifest，并显式声明 neural primary dependency、
  rendering post-process required dependency、runtime module、capability 和默认禁用策略。
- feature crate 只从 root manifest owner 构造 `RuntimePluginFeatureRegistrationReport`；不复制
  manifest 字面量，不伪造尚未实现的 render extension。
- editor crate 通过 `EditorPluginDeclaration` 镜像 runtime package，并声明真实 ONNX/model
  authoring capability；不创建空 UI contribution 或兼容 shim。
- runtime/editor catalog 和 `zircon_app` feature wiring 能把选择的 neural package 投影到已链接
  provider；动态 plugin key 保持外部 key 模型，不向核心枚举硬编码新 variant。
- Plugins12 schema/skeleton 审计归零，三个 neural crate 的 focused current-source tests 通过，
  独立二次审查为 `Critical 0 / Important 0` 后才能返回 fixed。

## 禁止临时方案

- 不恢复可选 feature 的非法 `maturity` 字段，不加解析兼容分支。
- 不在 feature/editor crate 复制 package manifest，不添加 facade、alias 或旧注册入口。
- 不以“workspace 已列 crate”替代 runtime/editor catalog 和 App feature 接线。
- 不注册无真实执行行为的 render pass、editor window、menu 或 importer。

## 修复结果与回传

待修复；尚未向来源计划回传 fixed。当前 Session 为 `resolving_failure`。TDD 已新增 runtime
manifest、feature registration 和 editor mirror/capability 回归；
root manifest owner、feature/editor plugin skeleton、ONNX authoring contribution、runtime/editor
catalog 与 App feature wiring 已实现。当前静态审计为
`manifest_schema_violations = 0`、`skeleton_migration_debt_count = 0`、
M1 `classified-and-clear`、M2 `sample-clean-migration-debt-clear`；capability roots `16`、
editor/runtime mirror violations `0`、distribution entries `41`、core workspace dependencies `154`。

尚未宣称 fixed：最终不可变快照的受管 Rust 1.94.1 focused tests、独立二次审查、managed
atomic commit 和向 Frameworks04 的 fixed return 仍待完成。Navigation editor/runtime mirror
审计已前向升级为 `NavigationOverlayFrame` 合同，没有恢复已退役的 agent-tick 路径。
snapshot1552 与其验证票据已因后续二审修复失效，不得作为验收证据。Neural 现同时进入
runtime capability roots 与 editor/runtime mirror roots；审计会验证 editor declaration、SDK
editor feature dependency、runtime manifest mirror 和测试合同，而不是只检查 runtime root。
完整 Neural package 的 40 个文件均纳入当前原子范围；root runtime 通过 dev graph 链接
post-process runtime feature，并由回归直接比较 linked registration 与 root 唯一 manifest owner。
`cargo metadata --locked --no-deps` 现可选择 neural runtime、post-process runtime 与 editor 三包。
snapshot1554 与其七张 queued 验证票据已因独立二审发现可执行语义缺口及四个 Runtime 输入漂移而失效，
不得作为验收或提交证据。当前 importer 已 hard-cut 拒绝没有 CPU/GPU 执行后端的 `Concat`/`Slice`，
`Resize` 只接受与现有后端一致的 nearest 模式，tensor id 分配在超过 `u16` 容量时返回显式诊断而不再回绕；
dynamic Session 静态守卫也已改为验证 process-log lease 仅在成功 shutdown 后清除的新 owner 合同。
修复后的 snapshot1558 exact72 source manifest 曾提交七张 Windows-native Rust 1.94.1 focused validation ticket，
但后续独立二审以 `C0/I5/M0` 拒绝该候选：production dependency graph 没有显式启用
`zircon_runtime/graphics`，catalog 仍断言旧 mirror root 数，候选遗漏 workspace/lock/session construction
编译输入，且 importer 未在导入边界验证全映射 arity 与 Resize 的固定采样语义。因此 snapshot1558
及其 queued tickets 只保留为诊断证据，不得用于验收或提交。当前源码已显式闭合 production graphics
feature、更新 mirror roots `4 -> 5`，并以统一 V1 arity 表拒绝 backend-incompatible 输入/输出数量；
Resize 只接受显式 scales、`nearest_mode=floor` 和
`coordinate_transformation_mode=asymmetric`，不再继承与现有 CPU/GPU backend 不一致的 ONNX 默认值。
workspace manifest、nested lock 与 Session construction 已纳入扩展原子范围；新不可变快照、受管 Cargo、
最终独立二审、managed commit、fixed return 和 accepted 结论仍待完成。
snapshot1560 exact76 已稳定通过 pre/post hash，但独立二审以 `C0/I3/M0` 功能性拒绝：normalization
epsilon 与 pooling 属性会被静默接受，broadcast/Conv/Pool 等 shape 合同只在执行期拒绝，超大 initializer
还会在生成诊断前发生 `u32` 乘法溢出。当前源码已继续前向修复：ONNX 转换只接受与 CPU/GPU V1
交集一致的属性和值，所有映射 op 在编码前经过集中 shape contract，initializer 计数使用 checked `u64`
并受 V1 `u32` element 上限约束。shape owner 已拆为 `editor/src/onnx/executable_contract.rs`，没有继续堆入
转换编排文件；broadcast、各后端 family shape、非法 epsilon/`count_include_pad` 与 overflow 回归均已新增。
snapshot1560 因这些源码变更失效；fresh immutable snapshot、最终独立二审、受管 Cargo、managed commit、
fixed return 和 accepted 仍待完成。
exact77 以稳定 pre/post 指纹 `78ba89a7161acf610503d250ea72510d8c969db7fe27d05ba6983fd7c807a0c0`
完成独立二审，但被 `C0/I4/M1` 功能性拒绝：终端 Reshape/Flatten 只生成 GPU 内部 alias 而不写 output，
Pool 可生成完全落在 padding 内的空窗口，Resize 的 `u32::MAX as f32` 边界会饱和转换，且容量检查只覆盖
output、没有覆盖 GPU WGSL 以 `u32` 扁平寻址的所有 input；exact77 另有 26 个旧范围 Rust 文件未通过
Rust 1.94.1 格式门。当前功能源码已前向修复：终端 view graph output 明确拒绝，Pool 每个窗口必须与输入相交，
全部张量在模型构造前受 V1 `u32` 元素地址上限约束，Resize scale 必须是精确可表示的正整数；新增回归同时
锁定 Max/Avg 空窗口、终端 Reshape/Flatten、大 Gemm input 与上界/小数 scale。exact 61 个 Rust 输入现已统一通过
Rust 1.94.1 格式门。exact77 已失效；fresh immutable snapshot、最终独立二审、受管 Cargo、managed commit、
fixed return 和 accepted 仍待完成。

## 完成项目与证据

| 日期 | 状态 | 完成项目与验证证据 |
|---|---|---|
| 2026-08-10 | `RED / reproduced` | schema `1`、skeleton debt root `1`、8 条缺失 owner/module 明细；两个 plugin owner 文件均不存在。 |
| 2026-08-10 | `source implemented / validation pending` | schema `1 -> 0`、skeleton debt `1 -> 0`；M1/M2 静态门清零，Rust 1.94.1 精确格式通过。catalog/App、Cargo、二审和 fixed return 未完成。 |
| 2026-08-10 | `wiring and review repair complete / validation pending` | dynamic neural key 已接入 runtime/editor catalog 与 App editor-host feature；editor importer/menu 现绑定真实、不可远程调用且可撤销的 ONNX -> ZNN operation factory，feature 回归锁定全部 16 个公开 extension family 为空；上一版 immutable 二审 `C0/I1/M1` 的两项发现均已前向修复。Python capability 单测 `4/4`、plugin structure audit exit `0`；旧 `f0f3adbe...` 候选因二审修复失效，fresh immutable 二审与受管 Cargo 仍待回执。 |
| 2026-08-10 | `second-review repair implemented / validation pending` | 后续 exact34 二审发现 ONNX 原始路径缺项目 authority，且 16-family 回归未给测试构建启用 graphics。现要求绝对 project root、规范化项目相对 source/output，并依据已加载项目清单的实际 asset roots 做物理 containment；新增覆盖恢复、首次创建撤销、解析失败不改输出及逃逸拒绝回归。graphics 仅作为 dev dependency 启用，不扩大生产依赖。旧 `f1412406...` 快照已失效，fresh immutable 二审与受管 Cargo 仍待回执。 |
| 2026-08-10 | `mirror audit repair implemented / validation pending` | snapshot1552 二审发现 Neural 只进入 runtime capability roots，未进入 editor/runtime mirror roots。现将 Neural 纳入既有 SDK mirror 强制审计并新增直接回归；capability 单测 `5/5`，完整 plugin structure audit 为 runtime roots `16`、editor mirror roots `5`、mirror violations `0`、compatibility shim sites `0`。snapshot1552 保留为 stale 诊断证据，fresh immutable snapshot、受管 Cargo 与独立二审仍待完成。 |
| 2026-08-10 | `atomic package scope repaired / validation pending` | 冻结前发现 Neural 目录 40 个文件仍全部 untracked，而旧候选仅覆盖 14 个，且 post-process runtime 未进入 workspace package graph。现将剩余 26 个 runtime/editor/dist 编译单元纳入同一 Session，并由 root runtime dev dependency + parity 回归接入 feature crate；metadata 已识别三个 neural test package。最终候选扩为 exact72，仍待 immutable 二审与受管 Cargo。 |
| 2026-08-10 | `executable import contract repaired / tickets queued` | snapshot1554 独立二审确认 linear Resize 被静默降级为 nearest、Concat/Slice 可导入却没有执行后端、tensor id 在第 65,537 项回绕，并检测到四个 Runtime 输入漂移。现以回归锁定 nearest-only Resize 及 scale 属性、无后端 op 导入拒绝、tensor 容量显式诊断，同时将 teardown source guard 对齐 process-log retry owner；snapshot1554 及其旧票据作废，fresh exact72 七张受管票据已取得 durable queued receipt，terminal Cargo、最终二次审查、commit/fixed return 均未完成。 |
| 2026-08-10 | `second-review contract repair implemented / candidate rebuilding` | snapshot1558 独立二审 `C0/I5/M0` 发现 production graphics feature 未闭合、catalog mirror count 仍为 4、原子输入遗漏 workspace/locks/construction，以及 importer 可接受 backend 必然拒绝的 arity/Resize 默认采样。现已显式启用 runtime production graphics，mirror count 更新为 5，完整 arity 表与显式 `scales + floor + asymmetric` 回归/拒绝路径落地，并扩展原子输入范围；snapshot1558 及七张 queued tickets 作废，新 immutable snapshot、受管 Cargo、最终二审和 fixed return 仍待完成。 |
| 2026-08-10 | `executable-only contract completed / candidate rebuilding` | snapshot1560 独立二审 `C0/I3/M0` 发现非法 normalization/pooling 属性、broadcast/shape 漂移与 initializer 计数溢出。现已用集中 V1 属性/shape 合同和 checked element count 前向修复，新增属性、九类 shape family 与 overflow 回归，并按模块预算拆出 `onnx/executable_contract.rs`；snapshot1560 作废，fresh immutable snapshot、最终二审、受管 Cargo 与 fixed return 仍待完成。 |
| 2026-08-10 | `exact77 review repair implemented / fresh candidate rebuilding` | exact77 稳定二审 `C0/I4/M1` 发现终端 view 不写 GPU output、Pool 空窗口、Resize 上界饱和、input 索引容量与旧范围格式缺口。四项功能合同及对应回归已前向修复，exact 61 个 Rust 输入现统一通过 Rust 1.94.1 格式门，`convert.rs`/`executable_contract.rs`/tests 保持 773/326/898 行；exact77 作废，fresh immutable snapshot、最终二审、受管 Cargo 与 fixed return 仍待完成。 |
