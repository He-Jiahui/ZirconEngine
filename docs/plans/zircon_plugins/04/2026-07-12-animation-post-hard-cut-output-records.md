# Plugins 04 Animation post-hard-cut 产出记录

> 来源文件：`docs/plans/zircon_plugins/04-animation.md`
> 记录边界：本文件承接正式测试阶段后续的 M6/WSL/IK 与 post-hard-cut Windows/WSL 验收；早期 8 条测试记录仍由 `2026-07-11-animation-testing-stage.md` 独占。

## 状态与产出记录

| 日期 | 阶段 | 状态锚点 | 验证事实 | 未纳入项 |
|---|---|---|---|---|
| 2026-07-11 | M6 Animation Editor 独立 workspace 复验 | `plugins_04_m6_editor_check_304s_dependency_timeout_no_diagnostic` | 根 workspace 正确拒绝未包含的 editor package；切换 `zircon_plugins/Cargo.toml` 后，nightly locked/offline Animation Editor `cargo check --tests` 在 Editor 依赖编译阶段达到 304.1 秒门禁，无 Rust diagnostic，且无遗留进程。 | 该次尝试没有 Cargo 退出 0；后续正式 20/20 由 M6 编号产出记录拥有。 |
| 2026-07-11 | post-hard-cut WSL `--tests` 静态门重试 | `plugins_04_wsl_tests_check_604s_runtime_metadata_timeout_no_diagnostic` | WSL nightly locked/offline `cargo check -p zircon_plugin_animation_runtime --tests --jobs 1` 进入 Linux `zircon_runtime` metadata 编译，并在 604 秒硬门超时；没有 Rust diagnostic。外层超时留下的本会话 WSL cargo/rustc 经 PID/命令行核对后精确清理。 | 没有退出 0，故 post-hard-cut WSL 仍待验；旧 75/75 WSL executable 基线不更新。 |
| 2026-07-11 | M5 IK contract 低内存行为门 | `plugins_04_m5_t1_lowmem_ik_contract_4_of_4_passed` | `CARGO_INCREMENTAL=0`、`RUSTFLAGS=-C debuginfo=0`、单 job、独立 F: target 的 Windows nightly locked/offline `cargo test -p zircon_plugin_animation_runtime --test animation_ik_contract` 在 457.8 秒后退出 0，TwoBone 可达/不可达、LookAt clamp 与 Manager per-World queue 合计 4/4 通过。 | 这是真实 executable 行为结果，但只更新 IK contract；post-hard-cut Windows/WSL 全套仍待运行。 |
| 2026-07-12 | post-hard-cut Windows 全套重试 | `plugins_04_windows_full_blocked_by_foreign_project_document_visibility` | Windows nightly locked/offline 全套使用受管理 D: target 重试；首次在进入 Animation 测试前，被并行资产文档迁移的 14 个 E0364/E0603 可见性错误阻断。Frameworks 所有者收紧 material/model 可见性后立即重试，错误缩减为 `project_document/codec.rs` 向上重导出 `decode_document`/`encode_document` 的 2 个 E0364。两次错误均在新建资产文档 owner，不在 Plugins 04 所有权边界。 | 两次均未执行 Animation test，不计为插件失败或通过；待活跃 Frameworks 所有者完成 codec 可见性收敛后重试。 |
| 2026-07-12 | post-hard-cut Windows 全套验收 | `plugins_04_post_hard_cut_windows_full_101_of_101_passed` | Frameworks 资产文档所有者收敛 codec 访问边界后，在专属 `D:\cargo-targets\plugins04-post-hardcut-windows-20260712` 上运行 nightly locked/offline `cargo test -p zircon_plugin_animation_runtime --tests --jobs 1 -- --nocapture --test-threads=1`；18m37s 构建后执行 16 个 test executable，合计 101/101 通过、0 failed，包含 production Tick 34/34。 | Windows post-hard-cut 最终门已通过；WSL 同源全套仍待运行。 |
| 2026-07-12 | post-hard-cut WSL 全套验收 | `plugins_04_post_hard_cut_cross_platform_101_of_101_passed` | 在 WSL2 Ubuntu-22.04 与专属 `/mnt/d/cargo-targets/plugins04-post-hardcut-wsl-20260712` 上运行 nightly locked/offline 同源命令；执行 16 个 test executable，结果分布与 Windows 一致（6/2/4/13/2/6/4/4/2/2/4/1/4/3/10/34），合计 101/101 通过、0 failed、退出码 0。日志保存在 `D:\cargo-targets\plugins04-post-hardcut-wsl-20260712\wsl-full-abcdc380a79c498885c5c9f113a2d0d0.{out,err}`。 | Plugins 04 post-hard-cut Windows/WSL 跨平台全套门均已通过；剩余工作仅为结构/记录复核与里程碑 closeout。 |
