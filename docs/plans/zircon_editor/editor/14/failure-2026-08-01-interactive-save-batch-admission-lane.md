---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: interactive-save-batch-admission-lane
origin_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/06
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/category.rs
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/extension/toolkit/mod.rs
  - zircon_editor/src/ui/retained_host/app/close_prompt
tests:
  - cargo test -p zircon_editor --lib --locked core::jobs::tests -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked save_dirty_views -- --test-threads=1
---

# Editor14：缺少交互式文档保存批次的有界 admission lane

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 来源执行切片：Editor06 DocumentToolkit failure 的 save-all / close-prompt 上行验收
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor06 只拥有 toolkit 写入 hook，Editor09 拥有 dirty generation 与批次语义；实际 I/O
  admission、资源互斥、取消、进度、完成回流和 shutdown 收口属于 Editor14 唯一 `EditorJobSystem`。

## 失败现象与复现证据

`docs/plans/zircon_editor/editor/09-editor-asset-management.md` 的 2026-07-31 PERF-MVP-602 约束要求
`SaveDirtyViewsRequest/Result` 先完成全批 typed preflight，再提交一次 canonical save batch，且 UI 线程的
serialize/fs/import 必须为零。Editor14 同日计划记录要求显式有界 interactive lane，包含 entry、estimated
bytes、age reservation、per-resource mutex group、cooperative cancel 与有界 completion apply。

当前 `zircon_editor/src/core/jobs/category.rs` 只有 `Import/Compile/Thumbnail/Export/Index/Play/Misc`；
`EditorJobLimits` 仅为 `Thumbnail` 与 `Export` 提供默认上限，其余类别默认 `usize::MAX`。因此 close prompt
若直接提交 `Misc + Interactive` 仍是无界准入，若在 retained callback 逐 view 调 toolkit 则继续同步执行
serialize、文件写入、import 与 workspace refresh。两者都不能满足既定架构。

## 最低共享层根因

Editor14 的统一 job 门面尚未提供交互式保存专用类别和有界 reservation，也没有将轻量
`document + dirty generation + estimated bytes` intent 转为执行期 toolkit payload 的 adapter。Editor06/09
不能在 toolkit、retained host 或领域 editor 中自行建立队列、worker 或 parallel save-all owner。

## 架构修复验收

- Editor14 在唯一 `EditorJobSystem` 中建立 typed interactive save 类别/adapter，并提供 entries、estimated
  bytes、oldest age 的硬上限；不得落入默认无限 `Misc`。
- admission 队列只持有轻量 document/generation intent；serialize payload 只在 ticket 获准执行后，从唯一
  DocumentToolkit/transaction owner 取得，取消或 supersede 必须发生在 payload 构建前。
- 每个文档使用稳定 resource mutex group，与 autosave/source save 共用互斥 owner；同文档写入不得重叠，
  不同文档可在显式预算内并行。
- worker 不捕获 `UiHostWindow`、retained host borrow、session mutex 或可变 UI 状态；完成结果通过有界回流，
  Editor09 仅在 dirty generation 匹配时 compare-and-mark。
- 覆盖 submit 拒绝、partial failure、cancel、stale generation、1/100/10k 文档、1KiB/1GiB payload、
  1/16 writers、stall 0/10ms/2s 与 shutdown deadline；记录 queue entries/bytes/age、payload owners/RSS、
  mutex wait 和 terminal latency。
- lower-layer gate 通过后，Editor06/09 重跑 DocumentToolkit save-all 与 close-prompt 矩阵：retry 只重提失败
  或新 generation 项，全部成功且 generation 匹配才允许 close commit。

## 禁止临时方案

- 禁止用 `Misc + Interactive` 的默认无限配额冒充有界保存 lane。
- 禁止在 UI callback、toolkit、asset/animation editor 中逐 view 同步写盘或创建第二个 save-all 循环。
- 禁止 admission 前序列化全部 payload、缓存第二份 dirty 状态、无条件 mark clean 或跨 generation 提交 close。
- 禁止用 test-only executor、全局禁止编辑或缩小规模矩阵掩盖 backpressure 和 partial failure。

## 修复结果与回传

Open state: `Editor06 DocumentToolkit 单文档 hook 已物化，但 canonical save batch 的有界 interactive I/O lane
尚不存在；Editor14 完成最低层 admission/terminal 合同、focused gate、独立复审和受管提交后按 lifecycle key
回传 fixed，Editor06/09 再完成 save-all/close-prompt 上行验收。`

## 产出记录与时间

- 2026-08-01：状态 `open_handoff_recorded`。完成当前源码与 Editor09/14 PERF-MVP-602 计划对账，确认
  `JobCategory` 无 interactive save 类别、非 Thumbnail/Export 类别默认无界；已将最低层 bounded admission、
  resource mutex、cancel/completion/shutdown 责任交接 Editor14，未增加同步 fallback、第二 job owner 或兼容 API。
