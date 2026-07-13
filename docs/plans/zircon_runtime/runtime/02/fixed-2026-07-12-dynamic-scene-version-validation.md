---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: dynamic-scene-version-validation
origin_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
fixing_plan: docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
origin_child_dir: docs/plans/zircon_runtime/runtime/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/05
related_code:
  - zircon_runtime/src/scene/dynamic_scene/scene/validation.rs
  - zircon_runtime/src/scene/dynamic_scene/document/write.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --lib runtime_session_archive_normalizes_noncanonical_inner --locked
resolved_at: 2026-07-12
---


# Runtime 05：DynamicScene archive 版本 fixture 漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 来源执行切片：Runtime 02 M3 上行 `core::` package-filter gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md`
- 交接原因：失败位于 DynamicScene 文档版本与 RuntimeSession archive 语义，属于 Runtime 05 场景序列化边界，不属于 Runtime 02 core/root/generated owner。

## 失败现象与复现证据

2026-07-12 当前 `zircon_runtime` lib-test 二进制运行 `core::` 字符串过滤器时，以下两个 RuntimeSession archive 测试失败：

- `runtime_session_archive_rejects_unsupported_embedded_scene_versions`
- `runtime_session_archive_rejects_unsupported_slots_on_push_and_upsert`

两个测试均把过渡期的 `DynamicScene::format_version` 改为 `999`，仍期望旧模型的 `DynamicSceneError::UnsupportedFormatVersion`。typed `PayloadHeader` 迁移后，schema id/version 已由 envelope 负责，canonical writer 会把过渡期内层字段规范化为 1；旧 fixture 因而与当前文档契约相反。

## 最低共享层根因

最低失败层是 `scene/tests/dynamic_scene/archive_core.rs` 的旧 fixture，而非生产校验。`docs/zircon_runtime/scene/dynamic_scene.md` 明确规定 external envelope header 是版本权威，内层 `format_version` 在后续移除前只作过渡双写；`document/write.rs` 克隆 payload 并把该字段规范化为当前值。生产 `validate_format_version` 对 typed header 的 schema id/version 校验符合该契约。

## 架构修复验收

- 保留 typed `PayloadHeader` 的 schema id/version 单一权威，不恢复内层字段的第二套版本裁决。
- 将两个 archive fixture 改为验证 `from_slots`、push 与 upsert 接受当前 typed envelope，并由 canonical writer 把内层 999 规范化为 1。
- 同步 structure-convention 对测试 owner 的函数名锚点，禁止留下“拒绝内层版本”的旧语义命名。
- 复跑 Runtime 02 的 `core::` 上行 gate；若还有失败，按其最低 owner 继续拆分，不得把本切片标为整个 gate 完成。

## 禁止临时方案

- 禁止为旧 fixture 恢复 envelope/payload 双版本权威。
- 禁止删除 typed header 校验或绕过 canonical writer。
- 禁止只改断言数值而保留误导性的 `rejects_unsupported_*` 测试命名。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime 05 | DynamicScene archive fixture 归因 | `completed` | 2026-07-12 | 当前 lib-test 的两个 archive 测试失败；对照 canonical writer 与模块文档后确认 typed envelope 是版本权威，旧测试仍要求内层字段拒绝语义。一次错误的生产校验尝试已在验证前完整撤回，未改变 production contract。 |
| Runtime 05 | archive fixture 与结构锚点收敛 | `实现完成-待-package-复验` | 2026-07-12 | 两个 fixture 现验证 construction/push/upsert 后 canonical JSON 不含内层 999，并可重新加载为 1；structure-convention owner 锚点同步为 `normalizes_noncanonical_inner_*`。生产 `validation.rs` 保持 typed header 单一权威。 |
| Runtime 05 | 当前源码结构守卫复验 | `completed` | 2026-07-12 | 新编译的 exact structure harness 首先暴露该 owner 仍读取已迁档父计划，随后把 Runtime 15 plan/index/review/structure 证据硬切到编号目录的四个 output records；最终 `runtime_15_dynamic_scene_root_tests_are_folder_backed` 通过 1/1。临时 wrapper 已删除。 |
| Runtime 05 | 当前 package 二进制聚焦复验 | `completed` | 2026-07-12 | 在 coordinator 兼容 target lane 上按最终源码重新生成 `zircon_runtime` lib-test 二进制；两个 `runtime_session_archive_normalizes_noncanonical_inner_*` 行为测试通过 2/2，folder-backed structure guard 通过 1/1。此前由并发编译生成的旧二进制仍包含 retired test names，已明确弃用，不计证据。 |

## 修复结果与回传

- 根因：RuntimeSession archive tests still asserted retired inner-format-version rejection after typed $zircon envelope became version authority.
- 架构修复：Kept typed envelope validation authoritative; rewrote archive fixtures to verify canonical normalization, synchronized structure guards to numbered output records, and updated module docs.
- 验证：Fresh current package binary: two normalization behavior tests 2/2, dynamic-scene folder-backed structure guard 1/1; Runtime02 core filter no longer reports either DynamicScene failure.
- 回传：Runtime05 fixture drift fixed and verified; Runtime02 upward core gate now has only Render/Shader/UI/Text failures.
