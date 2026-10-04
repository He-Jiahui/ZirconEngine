---
title: Editor01 project launch authority measurement plan
category: zircon_editor
report_id: Editor01-project-launch-authority
date: 2026-09-02
status: measurement_pending
implementation_files:
  - zircon_editor/src/core/project/authority/preflight.rs
  - zircon_editor/src/core/project/project_launch_preflight.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
tests:
  - zircon_editor/src/core/project/tests/preflight.rs
  - zircon_editor/src/ui/host/editor_manager_project_session/tests.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
---

# Editor01 项目启动 authority 测量计划

## 目的

验证项目创建、打开和 Editor session 激活是否存在结构性瓶颈，再决定是否进入算法优化。当前源码已经将创建/打开请求收敛为 `ProjectLaunchIntent` → `ProjectAuthority::preflight_project_launch` → `ProjectLaunchPreflight` → session admission；本文件不把静态接线或确定性模型当作 CPU、内存或首帧性能证据。

## 测量边界

| 阶段 | 必须记录 |
|---|---|
| Preflight | manifest read、template render、composition compile 的 wall time 与输入字节 |
| Create | staging entry 数/字节、manifest save、atomic rename、rollback 次数与耗时 |
| Open | canonical path、manifest load、project/asset registry load、scene document load |
| Context/session | `EditorContext` construction、session admission、runtime handoff 到首个可提交帧 |
| Failure | typed failure stage、staging/backup 清理耗时、残留临时目录数量 |

## 规模矩阵

- 空 `RenderableEmpty` 工程；
- 1,000 个资产、100 个实体的工程；
- 10,000 个资产、1,000 个实体的工程；
- 每个规模分别执行 cold open、连续 reopen、创建失败回滚和正常创建。

## 采集协议

1. 使用 Windows-native coordinator managed validation；Cargo 输出和 profiling artifacts 只写 `E:/`、`D:/` 或 `F:/`，不得使用 `C:/`。
2. 在 authority/session owner 增加阶段计时和字节计数时，必须让同一 receipt 携带这些数据，避免跨文件系统快照比较。
3. 必要时使用 ETW/WPR 观察文件 I/O、线程等待和进程生命周期；Rust counters 只解释 owner 阶段，不替代产品进程证据。
4. 只在 current-source managed run 取得 CPU、RSS、I/O、首帧和 p95 数据后形成性能结论；静态 pressure model 只能用于筛选待测假设。

## 可证伪假设

- 若 `preflight` 与 `activation` 的 p95 随资产规模近似线性增长，先检查重复 manifest/registry materialization，再考虑按 authority generation 设计增量读取；不得直接增加 cache。
- 若 staging write 占主导，先检查模板 entry 合并与原子事务边界；不得牺牲 rollback/durability 语义换取吞吐。
- 若 session/runtime handoff 占主导，先用 trace 区分 admission lock、runtime frame wait 与 UI construction；不得将同步等待隐藏到新线程。
- 若性能没有随规模恶化，保持当前结构，不进行没有数据支持的局部优化。

## 当前状态

`measurement_pending`。本轮只完成 authority 接线复核与静态合同补强；尚未执行 managed Cargo、真实产品运行、ETW/WPR 或 CPU/RSS/GPU 采样，因此不宣称瓶颈消失或性能改善。
