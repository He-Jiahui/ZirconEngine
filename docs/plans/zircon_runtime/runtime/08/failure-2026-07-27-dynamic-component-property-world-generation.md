---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: dynamic-component-property-world-generation
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
related_code:
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/reflect/dynamic_component.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
  - zircon_runtime/src/scene/world/generation/tests.rs
  - zircon_plugins/navigation/runtime/src/manager/state.rs
tests:
  - cargo test -p zircon_runtime --lib dynamic_component_reflection --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib generation --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked --jobs 1 -- --nocapture --test-threads=1
---

# Runtime08: dynamic component property write world-generation contract

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：fallback navigation typed projection and immutable generation support layer
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：`World` 是 generation 的唯一发布者。Runtime08 已承接 generation-owned compiled accessor 和 scene property write boundary；Navigation 只能消费 generation，不能为缓存另建变更真相。

## 失败现象与复现证据

`World::set_property` 在成功变更后标记 inspection dirty 并确保 world generation 前进；但反射写入绕过此入口：`scene/reflect/dynamic_component.rs::write_declared_field` 直接调用 `set_dynamic_component_property` 或 `set_dynamic_component_json_property`。后者两条成功写入分支仅更新 JSON，未发布 generation 或 inspection dirty。

因此动态 NavMeshAgent/Obstacle 经反射或动态组件属性路径修改后，generation-keyed navigation projection 可将旧 typed descriptor 当成当前值复用。该结论来自当前源代码静态追踪；尚未声称 Cargo gate 通过。

## 最低共享层根因

动态组件属性的实际 mutation boundary 没有统一发布 `{inspection dirty, world generation}`。把发布逻辑只留在 `set_property` 使反射、脚本和其他直接动态属性 consumer 获得不同的缓存失效语义。

## 架构修复验收

- 在唯一动态组件属性 mutation boundary 上，对 VM 和非 VM 成功且值变化的写入各发布一次 inspection dirty 与 world generation；未变化、校验失败和缺失实体不发布。
- `set_property` 保持一次写入只推进一次 generation，不以调用方双重发布作为兼容方案。
- 为 VM 和非 VM 的 reflection/property write 补回归：修改后 generation 增一且 inspection artifact generation 刷新；相同值写入 generation 不变。
- 回跑上述 Runtime08 focused gates，再回跑 Navigation typed projection gate，证明属性编辑使 projection 重建、稳定帧继续复用。

## 禁止临时方案

- 不在 Navigation、Editor、脚本或反射调用点添加本地 generation bump、额外 cache key 或测试专用失效分支。
- 不保留“反射写入不失效、property write 才失效”的双语义。
- 不削弱 stable-generation projection 计数断言或以每帧 JSON 重扫隐藏失效问题。

## 修复结果与回传

Open state: `implementation_complete / focused_and_navigation_validation_pending`.

- `scene/world/dynamic_components.rs` 的非 VM 与 VM/JSON changed 写入分支已在 mutation boundary 同时发布 inspection dirty 和 world generation。
- `scene/world/property_access/write.rs` 只在下层未推进 generation 时补发布，避免一次写入双增量。
- `scene/tests/ecs_reflect/dynamic_components.rs` 已覆盖 VM/非 VM changed 写入只增一、unchanged 不增的回归合同。
- 本轮未把记录改为 fixed：当前 Windows coordinator 有其他 owner 的 managed Cargo 作业占用，且最新 runtime lib-test 编译仍有外部 owner 错误；focused Runtime08 与 Navigation gate 实际执行前不声称通过。
