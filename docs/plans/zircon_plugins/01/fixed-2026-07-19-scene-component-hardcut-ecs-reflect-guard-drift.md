---
handoff_kind: fixed
status: fixed
created_at: 2026-07-19
summary_slug: scene-component-hardcut-ecs-reflect-guard-drift
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/structure.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib scene::tests::ecs_reflect::structure::builtin_component_metadata_is_owned_by_zr_reflect_derives --locked --jobs 1 -- --exact --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib profile --locked --jobs 1 -- --nocapture --test-threads=1
resolved_at: 2026-07-19
---


# Frameworks06：scene component hard cut 漏同步 ECS reflection guard

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行者：`plugins01-runtime-profile-availability-projection-r3-20260718`
- 来源执行切片：runtime profile broad source guard job `c1fe7621b2bc4aa1b68291f8fa117248` / run `835c9dcd9316494eba57e2f929f1f7df`
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 交接原因：失败由 Frameworks06 `scene.rs -> scene/` 物理 hard cut 与结构 guard 清单不同步造成，Plugins01 不应恢复旧 component owner 或修改 scene reflection 测试边界。

## 失败现象与复现证据

受管 Windows job 的 exact command 为 `cargo +1.94.1 test -p zircon_runtime --lib profile --locked --jobs 1 -- --nocapture --test-threads=1`。raw stderr 位于 `.codex/state/session-coordinator/cargo-runs/c1fe7621b2bc4aa1b68291f8fa117248/835c9dcd9316494eba57e2f929f1f7df/stderr.log`，rustc 报告：

`zircon_runtime/src/scene/tests/ecs_reflect/structure.rs:193` 的 `include_str!("../../components/scene.rs")` 读取已物理删除的 monolith，产生 missing-file compile error。

当前工作树已删除 `zircon_runtime/src/scene/components/scene.rs`，并新增 `scene/{mod,activation,animation,camera,hierarchy,identity,mesh_renderer,node,physics,transform}.rs` owner 集合。Frameworks06 owner Session `frameworks06-m2-scene-component-owner-hardcut-r4-20260719` 的 exact16 scope 覆盖 hard cut 与两个结构 guard，但没有包含 `scene/tests/ecs_reflect/structure.rs`，所以旧镜像没有原子迁移。

协调器在该 Plugins01 job 尾段重启；进程树已自然退出，owner 使用完整 raw log 补登记 `exit 101` 并受管释放。该运行只证明旧 owner 与 7 条独立 Text01 compile diagnostic，不能作为 Plugins01、Frameworks06 或 Runtime acceptance。

## 最低共享层根因

Frameworks06 只迁移了 component owner 与其直接 file-budget guard，没有枚举所有通过 `include_str!` 读取旧 monolith 的结构/反射镜像。`ecs_reflect::structure` 仍假定所有 builtin metadata type path 位于单一 `scene.rs`，与新九域 owner 架构冲突。

## 架构修复验收

- 将 `zircon_runtime/src/scene/tests/ecs_reflect/structure.rs` 纳入 Frameworks06 successor exact manifest，并移除对 `components/scene.rs` 的读取。
- guard 必须按新 domain owner 集合验证 canonical reflected metadata：每个 type path 只在其真实 owner 检查，并逐类型绑定相邻 `ZrReflect` derive；`scene/mod.rs` 只验证 module/re-export facade，不冒充声明 owner。
- 递归扫描确认生产、测试与文档没有把旧 `scene.rs` 当现存 owner；保留 hard-cut、9/9 domain owner、Active alias 0 的既有约束。
- 执行 fresh immutable `ecs_reflect` focused gate 与 `zircon_runtime --lib` broad gate，独立复审后进入 Frameworks06 managed commit/fixed return；Plugins01 随后重建源绑定门禁。

## 禁止临时方案

- 不得恢复 `scene.rs`、增加 forwarding include/shim、把声明复制回 facade，或删除 reflection metadata guard。
- 不得让 Plugins01、Text01、Editor 或 Render owner 吸收 Frameworks06 hard-cut 路径。
- 不得把 orphaned/no-exit job 记为 RED 验收，只能保留其 raw rustc 诊断。

## 修复结果与回传

- 根因：Frameworks06 r4 的 exact16 hard-cut manifest 删除 flat scene.rs 并新增 domain owners，但漏纳仍 include_str! 旧 owner 的 ecs_reflect/structure.rs，导致生产 owner 与 reflection structure guard 非原子迁移。
- 架构修复：r6 将 ECS reflection guard 纳入同一 hard-cut manifest，按 9 个 builtin type path 绑定 identity/hierarchy/transform/activation/camera/mesh_renderer/physics 真实 owner，并验证每个声明相邻的 ZrReflect derive；未恢复 scene.rs、shim 或兼容 facade。
- 验证：受管 job 683eb23631aa4364a6cdbc82de80dddd / run 23f5eb7e782443469c6a7862e305a06d 使用 Rust 1.94.1 与 source manifest fingerprint fc564368ad11bf37d5632cc0b04bf3fde3e2c5bfe470e02a49b71e60f423d7d9，完整 libtest 路径实际执行 1/1，0 failed，8528 filtered，exit 0；rustfmt/diff check 通过；最终独立复审 C0/I0/M0。
- 回传：Frameworks06 scene component owner hard-cut 已在最低共享层闭合并回传 Plugins01；Plugins01 可基于本 fixed record 重建 profile upward gate，但不得复用此前 RED。
