---
plan_source: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
related_code:
  - zircon_runtime/src/scene/dynamic_scene/document/
  - zircon_runtime/src/scene/dynamic_scene/scene/
  - zircon_runtime/src/scene/dynamic_scene/session/slot/summary.rs
  - zircon_runtime/src/scene/mod.rs
tests:
  - zircon_runtime/tests/plan11_scene_serialization_contract.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/archive_core.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
  - zircon_runtime/src/scene/tests/component_structure/dynamic_scene_owner_tree.rs
---

# Editor11 DynamicScene 版本壳硬切换

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-14 | M2 / 切片 2.2 | 切片完成；Plan11 专属合同 8/8；broad 外部失败已路由 | `DynamicScene.format_version` 与 `DYNAMIC_SCENE_FORMAT_VERSION` 已从 `dynamic_scene/scene` owner 删除；协调器延迟补丁 `6` 按 Plugins08 租约基线哈希删除 canonical root 重导出且保留对方改动。DynamicScene schema 从 v1 提升为 v2：v0→v1 生成历史 payload，v1→v2 必须先验证内嵌版本存在且等于 1 才删除字段，project-world 输入走完整顺序链；capture 与 canonical writer 不再双写，`#[serde(deny_unknown_fields)]` 使当前 v2 若重新携带旧字段返回 `PayloadDecode`，runtime-session summary 改从私有 payload header 投影版本。TDD 自审负例作业 `c3c58d75dc7549c6adede931f806742b` 0/1 RED，收紧迁移后 `feded57b4f9946cdb5e70e8cf1c18075` 1/1，最终 `e6b4ac85c8994c0eadfea26fb026061f` 8/8；受管 core-min broad `0828b8e5681045ccb47296cbcc1880f3` 为 595/596，唯一失败属于 Plugins08 的动态反射写入结构守卫，证据已由协调器补丁 `7` 写入其开放 failure。Text01 target-client 产品作业 `d80d6dabac754907b50aa3ae2c1c1056` 1/1，根导出 failure 已按 lifecycle 回传；rustfmt、旧符号扫描与 scoped `git diff --check` 通过。 | 等 Plugins08 修复并回传 dynamic reflection 后复跑默认 `scene::` broad 门；该外部门不回滚本切片 8/8 的版本壳验收。M2.1 与 M3 未完成，Plan11 保持 `in_progress`；最终协调器 snapshot 在本记录落盘后刷新。 |
| 2026-07-14 | M2.2 协调器提交准备 | `未通过-计划工作流拓扑缺失` | `milestone prepare --session-id editor11-dynamic-scene-m2-20260714 --milestone M2.2` 返回 `workflow_topology_missing`：Plan11 没有 `zircon-workflow` block 或可识别 milestone heading；未生成 run/manifest，未执行普通 Git 提交。 | 由计划维护边界补齐受保护 workflow topology 后重新 prepare/review/validate/commit；不得为提交方便伪造 M2.1 完成状态，也不得吸收 `scene/mod.rs` 的 Plugins08 同文件改动。 |
| 2026-07-16 | M2.2 新鲜关闭门复放 | `实现/独立复核/专属合同8项完成-受管提交待外部依赖提交与checker修复` | Plan11 已补齐机器可读 workflow 与原生 `M2.2` node；exact16 独立复核最终 Critical/Important/Minor=`0/0/0`，唯一文档 Minor 已把无壳 project-world 迁移说明从 v1 修为 v0；exact12 Rust `rustfmt --edition 2024 --check --config skip_children=true`、scoped `git diff --check` 与计划产出审计通过。首次 Windows canonical reservation `a8fbcc6c702d4c3f84c9058e3003d694`、job `40ddc27bbce24cdb81b03ea9e76d6166`、run `5d6fcfbb5c654845bbb1850e2939ec36` 以 clean target 执行既定命令，在测试枚举前由 Text raster pool 的退役 `crate::core::ZirconError` consumer 触发唯一 E0432，exit 101；最低 owner 已写入 Frameworks05 `failure-2026-07-16-text-raster-pool-zircon-error-consumer.md` 并硬切为 `CoreError/CoreResult`。复跑 reservation `6512e8ec838a4cd6a91403cd28e958c9`、job `7c7b4dfb478d4a82b3e54c22144b6039`、run `3219941cf2ee41328b99f2973853a4a4` 越过该 E0432，最终 `plan11_scene_serialization_contract` 8 passed / 0 failed、exit 0：v0 project-world、v0 夹具幂等、v1 历史内嵌版本正负合同、future embedded header 优先拒绝、当前 v2 拒绝旧字段与 archive 单一 header authority 全部实际执行。关闭证据复放进一步确认 Coordinator01 投影漂移：本 Session 固定 `baseline_epoch=141`，但 `read-closeout-evidence.py` 读取全局最新 epoch 158；16 条 attribution 的原始 SHA256 均与当前文件一致，其中 7 条仅因后来进入 epoch 158 baseline 而被错误排除，最终只投影 9/16，触发 `manifest_path_not_owned`。 | Frameworks05 仍须把 Text consumer 修复、focused gate、独立复核和 failure return 受管提交，确保 M2.2 不依赖未提交外部工作树；随后由 Coordinator01 按既有 `failure-2026-07-16-native-slice-closeout-checker-staged-index-contract-drift.md` 修复 native slice、空 index 与 Session-baseline attribution 投影，再提交 exact16。不得恢复 `ZirconError` alias/root re-export、吸收外部 Text 文件或把 pending 的 M2.1 伪装为完成。 |
