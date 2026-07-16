---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: dynamic-reflection-json-projection-regression
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/08
related_code:
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/reflect/dynamic_json.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
tests:
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-14
---


# Plugins08：dynamic reflection JSON projection 回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 声明的默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：Plugins08 当前 reflection hard-cut 改变了 direct-success 控制流与 scalar JSON 精度；Editor02 不拥有 dynamic reflection projection 或 VM schema conversion。
- 受管 Windows 日志：`E:\ZirconBuilds\editor02-m1-runtime-scene-default-after-text01-fix-20260714.log`。

## 失败现象与复现证据

- `scene::tests::ecs_reflect::dynamic_components::dynamic_component_reflection_read_helpers_use_direct_success_branches` 失败：当前 `read_declared_field` 改为 `as_object().and_then(...).ok_or_else(...)`，删除了结构守卫要求的显式 field-presence success branch。
- `scene::tests::ecs_reflect::dynamic_components::dynamic_component_reflection_writes_json_property_through_facade` 失败：写入 `ReflectedValue::Scalar(0.9)` 后 JSON 得到 `0.8999999761581421`，不再保持 facade 约定的数值投影。
- Editor11 受管 core-min scene 门禁 `0828b8e5681045ccb47296cbcc1880f3` 运行 596 项，595 通过、1 失败；新增失败 `scene::tests::ecs_dynamic_components_structure::dynamic_component_property_writes_split_and_insert_only_at_map_boundaries` 指向同一 Plugins08 owner：`set_dynamic_component_json_property` 的 VM 分支重新引入 `.and_then(...).map(...).ok_or_else(...)`，违反动态组件写入只在 map 插入边界分配拥有字符串且使用显式成功/错误分支的结构约定。日志：`E:\ZirconBuilds\editor11-m2-dynamic-scene-coremin-retry3-20260714.log`。
- 同一轮其余 Editor02 generation、split inspection、5k 深链和 cycle-edge 测试全部通过；两项失败均位于 Plugins08 当前持有租约的 reflection hard-cut 路径。

## 最低共享层根因

Plugins08 把 dynamic component reflection 硬切到新的 JSON projection helper 时，同时改变了既有 direct-success 分支结构和 `f64 -> f32 -> JSON` 的精度语义；最低修复点是共享 `dynamic_component`/`dynamic_json` projection，而不是 Editor02 上层查询或测试过滤。

## 架构修复验收

- 恢复或以同等清晰的显式分支实现 field-presence 成功/错误路径，满足 Runtime15 结构约定，不使用 closure 链隐藏控制流。
- `ReflectedValue::Scalar` 写入 JSON 时保持输入的可观察 `f64` 语义；如 schema 明确声明 f32，必须由 typed schema conversion 给出可审计规则并同步测试，而不是无提示窄化。
- fresh 重跑 `cargo test -p zircon_runtime --lib scene:: --locked`，两项测试通过且 VM-backed reflection、unknown-field 与 non-editable 路径不回归。

## 禁止临时方案

- 禁止在 Editor02 中对 `0.9` 做容差掩盖或跳过 reflection 测试。
- 禁止恢复旧 reflection facade、双轨 JSON owner 或 test-only bypass。
- 禁止仅改期望值为 `0.8999999761581421` 而不定义 typed conversion 契约。

## 修复结果与回传

- 根因：VM dynamic JSON hard-cut replaced explicit field-presence branches and widened f32 values through f64 JSON formatting, while legacy and VM missing-key semantics diverged.
- 架构修复：Centralized strict VM declared-type conversion, restored direct-success field checks, emitted shortest finite f32 JSON decimals, separated legacy and VM routes, and made complete candidate writes transactional.
- 验证：回传时证据为 default-feature managed dynamic-components 13/13、lib-test dynamic-components 14/14、VM backing 3/3、Runtime reflection 28/28、plugin 20/20；回传后 Plugins08 又补齐 catalog provenance capability 回归，最终 Runtime reflection 已 fresh 29/29，插件新增 direct-resolve 回归仍由 Plugins08 主产出记录跟踪，不改变本 Editor02 JSON projection failure 已关闭的结论。
- 回传：Plugins08 reflection regression is fixed; Editor02 may resume its scene gate. Any remaining broad compile failure belongs to the independent UI text ProjectAssetManagerAccess consumer.
