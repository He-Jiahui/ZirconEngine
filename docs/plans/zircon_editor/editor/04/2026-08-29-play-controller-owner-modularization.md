# Editor04 Play Controller Owner Modularization

- 日期：2026-08-29
- 归属：Editor04 PIE 与 Simulation
- 范围：`PlaySessionController` 的 preview transport 与 runtime ownership 物理 owner 收敛
- 状态：源码完成、静态门通过、受管 Cargo 待当前源码验证

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-08-29 | preview transport 独立 owner | `source complete / static green / managed validation pending` | `capture_preview_frame`、Play input、SIE camera 与 gateway handle 读取整体迁入 `core/play/controller/preview_routing.rs`；根 owner 不保留 forwarding wrapper 或第二实现。 |
| 2026-08-29 | runtime ownership 独立 owner | `source complete / source guards 3/3 / managed validation pending` | gateway attach/detach、终态 detach reservation、attached-domain 查询与 backend retirement 整体迁入 `core/play/controller/runtime_ownership.rs`；根 `controller.rs` 从 854 行降至 623 行，减少 27.0%，低于 800 行软预算。独立 `rustc --test` 源码守卫 3/3、相关文件 `rustfmt --check`、scoped `git diff --check` 与六个方法唯一 owner 检查通过。 |
| 2026-08-29 | 验证边界 | `open` | 先前受管请求 `d565b22206454600abbc77f12cf73610` 在 `cargo.acquire` 后以 `command_post_timeout` 结束，且早于本次 runtime ownership 迁移，不能作为当前源码接受证据。后续须以新 current-source manifest 执行 `zircon_editor` 受管检查；通过前不关闭 Editor04 里程碑、不提交、不发送企微。 |
