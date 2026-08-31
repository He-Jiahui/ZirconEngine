# Editor06 Extension Preview Action Owner Modularization

- 日期：2026-08-29
- 归属：Editor06 UI 扩展框架
- 范围：Workbench extension preview action 注册表物理 owner 收敛
- 状态：源码完成、静态合同通过、受管 Cargo 待办

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-29 | extension preview action 领域拆分 | `source complete / static green / managed validation pending` | 将 icon library、accessibility audit、menu flow、font atlas、console/runtime diagnostics、performance 与 telemetry dashboard 共 160 个质量观测 action ID 迁入 `workbench_preview_actions/extensions/quality_and_observability.rs`；`extensions.rs` 保留 728 个内容/工程扩展 ID，并从 890 行降至 787 行，新叶子 162 行，均低于 800 行软预算。 |
| 2026-08-29 | 无分配有序组合合同 | `source guard 1/1 / sequence 888 -> 888` | 根模块以 `const fn` 在编译期组合两个静态 slice，不创建 `Vec`、`LazyLock` 或第二运行时 registry；现有 `WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS: &[&str]` 合同不变，因此未修改已有并发输入的 `workbench_preview_actions.rs`。旧 HEAD 单体表与新组合逐项比较：总数 888、顺序差异 0；独立 Rust 守卫 1/1、相关 `rustfmt --check` 与 scoped `git diff --check` 通过。 |
| 2026-08-29 | 验证边界 | `open` | 本切片未运行 Cargo、Retained 产品 UI 或输入交互；后续须在 current-source 受管副本中验证完整 module resolution 与 shared preview registry tests。通过前不提升 Editor06 里程碑、不提交、不发送企微。 |
