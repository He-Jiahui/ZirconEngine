# Frameworks01 M0 Windows Build Timings Baseline

Status: `measurement-pending`。本文件只锁定 M0 测量方法和当前机器；四个 HTML 报告及数值必须来自受管 Cargo 自然终态，未运行前不填写推测值。

## 环境

- CPU: AMD Ryzen 7 5800H，8 cores / 16 logical processors，MaxClockSpeed 3201 MHz
- Memory: 39.86 GiB visible
- OS: Windows 11 Pro 10.0.26200 build 26200
- Rust: `rustc 1.94.1 (e408947bf 2026-03-25)`，host `x86_64-pc-windows-msvc`，LLVM 21.1.8
- Cargo: `cargo 1.94.1 (29ea6fb6a 2026-03-24)`
- 测量前可用空间：D 92.43 GiB、E 90.47 GiB、F 83.18 GiB

## 方法

1. Workspace 使用一个全新 coordinator compatibility pool，先运行 `cargo +1.94.1 build --workspace --locked --timings --jobs 1 --color never` 取得 cold 报告，再在同一 pool、源输入未漂移时运行相同命令取得 incremental 报告。
2. Runtime 使用另一个全新 pool，先运行 `cargo +1.94.1 build -p zircon_runtime --locked --timings --jobs 1 --color never`，再在同一 pool、源输入未漂移时重复一次。
3. 每个作业记录完整 compile-input pre/post fingerprint、job/run id、exit code、Cargo Finished duration 与 timing HTML SHA-256。任何源漂移、非零终态、进程树残留或复用已有 target 都使该轮失效。
4. 不执行 `cargo clean`，不删除共享 target，不使用 repo-local target；通过不同 compatibility key 获得两个 M0 专属空 pool。

## 待产物

| 范围 | cold | incremental | 状态 |
|---|---|---|---|
| root workspace | `workspace-cold.html` | `workspace-incremental.html` | pending |
| `zircon_runtime` package | `zircon-runtime-cold.html` | `zircon-runtime-incremental.html` | pending |

M0 只有在四个报告均为 current-source GREEN、依赖图与 crate/CI 锁定同步后才可完成；本文件存在不等于 timings 已通过。
