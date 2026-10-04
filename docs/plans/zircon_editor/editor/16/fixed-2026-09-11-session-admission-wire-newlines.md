---
handoff_kind: fixed
status: fixed
created_at: 2026-09-08
summary_slug: session-admission-wire-newlines
origin_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
origin_child_dir: docs/plans/zircon_editor/editor/16
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/project/session_lock/codec.rs
  - zircon_runtime_interface/src/project/session_lock/tests.rs
  - zircon_editor/src/core/recovery/session_guard/record.rs
  - zircon_hub/src/process/editor_focus/probe.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib project::session_lock::tests::
  - cargo test -p zircon_editor --locked --lib core::recovery::tests -- --test-threads=1
  - cargo test -p zircon_hub --locked --lib process::editor_focus::
resolved_at: 2026-09-11
---

# Interface02: project session admission wire newlines

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 来源执行切片：Editor16 唯一 session admission 对公共持久记录的读写依赖。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 交接原因：codec 属于 Interface02 公共持久契约，Editor recovery 和 Hub focus 共同消费。
- 关联上层：[Editor16 session lock recovery failure](../../../zircon_editor/editor/16/failure-2026-07-23-project-session-lock-reuse-for-recovery.md)。

## 失败现象与复现证据

Windows locked/static 作业 `893e5568e6364fd9944fd431c04c6ffa` 的原有
`admission_record_round_trips_through_the_shared_strict_format` 在
`encoded.starts_with("version=2\n")` 失败。完整接口库 747 passed、16 failed、
101 ignored。冻结输入 `interface-cache-generation-3159-20260908` 的 manifest 为
`e275ded38d2cd913d9bac17ee5ec8fdcff1f9e17e75306de307a82d6b5c38038`；
日志为该输入下 `results/interface-library-3159.log`。

## 最低共享层根因

生产编码器 format 字面量把每个真实换行转义为字面字符反斜杠和 n；严格解码器使用
`source.lines()`，因此连自身输出也不能解析。Editor 的 `encode_record/read_lock`
以及 Hub 的 `probe_project_editor_session` 直接调用同一公共 codec，影响正常项目
会话的持久化与重新检查。协议版本 2、字段和 lifecycle 约束本身无需改变。

## 架构修复验收

- 原始严格 round-trip 与同模块全部生命周期和无效记录测试通过。
- 生产 writer 使用真实 LF；reader 继续拒绝旧版本、未知或重复字段与无效 lifecycle。
- Editor recovery 与 Hub focus 直接消费者的受管验证完成，独立审查 C0/I0/M0。
- 通过正式 fixing-Session 验收、failure return 和协调器 closeout 后才关闭。

## 禁止临时方案

- 不接受被双重转义的旧损坏记录，不在 Editor/Hub 增加第二套 codec 或宽松读取。
- 不绕过 OS lease/admission，不以本 codec 回归代替 Editor16 的 crash-restart 全链验收。

## 修复结果与回传

- 根因：The session admission writer encoded every record line as a literal backslash-n sequence, while the strict reader parses real source lines; the writer could not round-trip its own v2 records.
- 架构修复：Restored real LF delimiters in the shared Interface02 codec only; strict version, field, lifecycle, duplicate-field and invalid-record validation remain unchanged, and Editor recovery/Hub focus continue consuming the single codec.
- 验证：Managed Windows locked/static Interface02 jobs 085a233ed2c945b28885f2be3f0edf8e and deff74bafac140c8e7edde2f617f1 on frozen Interface02 source passed the five session_lock round-trip/lifecycle tests and 18 reflect contracts; source codec hash 14ac9df67ddf65f3918a42cdb1abe9fd98c521a56ee20a41a35c9ad2ddc49a42, tests hash 88495e59f93aef6550de66e9451ffc11416cf7171764a0567ac2ae958b7bd8c1; independent review C0/I0/M0 report .codex/tmp/interface02-runtime09-3173-review-20260908-result.txt.
- 回传：Return session-admission-wire-newlines as fixed: shared writer now emits real LF wire records and the existing strict codec tests pass; Editor/Hub consumers remain on the same public codec. Formal closeout remains coordinator-gated.
