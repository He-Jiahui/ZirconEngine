---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: hub-message-legacy-test-drift
origin_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
fixing_plan: docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md
origin_child_dir: docs/plans/zircon_editor/editor/10
fixing_child_dir: docs/plans/zircon_hub/07
related_code:
  - zircon_hub/src/state/hub_message/message.rs
  - zircon_hub/tests/project_management_contract.rs
plan_sources:
  - docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_hub --locked
---

# Hub 07：HubMessage legacy 测试 API 漂移交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 来源执行切片：Plan10 M1.1 Hub Summary 解析测试阶段
- 修复责任计划：`docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md`
- 交接原因：Hub 全包测试在执行 Plan10 `projects::validation` 夹具断言前编译所有 integration targets，当前最低错误位于 Hub 07 消息 schema 的历史测试调用。

## 失败现象与复现证据

受管命令 `cargo test -p zircon_hub --locked` 的 library build 已通过；Plan10 M1.2 首轮测试编译问题修复后，2026-07-11 复跑仍在 `zircon_hub/tests/project_management_contract.rs:204:33` 报 E0599：测试继续调用已删除的 `HubMessage::legacy("opened")`。当前硬切后的生产 API 只有 `new`、`with_params`、`raw_text` 与 `empty`；没有兼容 alias。仓库计划文本仍列出 `legacy(...)` 迁移用语，但生产实现已经收敛为 `RawText/raw_text`，导致测试 API 漂移。

这不是 Plan10 manifest Summary 或 Hub project validation 行为失败；Plan10 的 Hub 单元断言尚未执行。

## 最低共享层根因

Hub 07 的生产消息构造 API 已硬切到 `raw_text`，但 `project_management_contract` 仍锁定退役的 `legacy` 名称，且计划正文仍有旧目标形状。最低失配位于 Hub 07 消息 schema 与其 integration contract。

## 架构修复验收

- Hub 07 owner 按当前消息 schema 更新 integration contract，或若 `legacy` 确为最终命名则完成生产 API 设计并统一所有调用；禁止只添加无语义的兼容 alias。
- 同步修正 Hub 07 计划中仍引用 `HubMessage::legacy(...)` 的目标代码形状，避免计划继续要求已退役名称。
- 修复后先复验 `project_management_contract`，再通知 Plan10 owner重跑 Hub 全包与 `--lib projects::validation`。

## 禁止临时方案

- 禁止添加仅转发到 `raw_text` 的 `legacy` 兼容 alias、双写或测试专用入口。
- 禁止删除、跳过或弱化 integration contract；应统一生产 API、计划和测试的单一命名。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Hub 07 | HubMessage raw-text API 与 integration contract 对齐 | `未通过-待-hub-owner-修复` | 2026-07-11 | 2026-07-11 Plan10 M1.2 受管复跑确认 `cargo build -p zircon_hub --locked` 通过；`cargo test -p zircon_hub --locked` 在 `project_management_contract.rs:204` 因 `HubMessage::legacy` 不存在报 E0599，当前 API 候选为 `raw_text`，未执行 Plan10 Hub Summary 行为断言。 |

## 修复结果与回传

- 状态：`open / 待修复`。
- 修复后更新本文件并回传 Plan10 M1.1；不得把当前“未执行”记为 Plan10 Hub 行为失败。
