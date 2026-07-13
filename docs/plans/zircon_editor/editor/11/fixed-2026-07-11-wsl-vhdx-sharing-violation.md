---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
summary_slug: wsl-vhdx-sharing-violation
origin_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
fixing_plan: docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
origin_child_dir: docs/plans/zircon_editor/editor/11
fixing_child_dir: docs/plans/zircon_runtime/runtime/01
related_code:
  - zircon_runtime/src/scene
  - zircon_runtime_interface/src/serialization
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-11
---


# Runtime 01：WSL 验证磁盘共享冲突

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/11-serialization-and-versioning.md`
- 来源执行切片：Plan11 M1.2 场景反射版本壳、v0→v1 AssetRef 迁移与 canonical writer
- 修复责任计划：`docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md`
- 交接原因：Linux/CI-parity 场景测试无法启动 WSL，失败发生在发行版虚拟磁盘挂载前，不能记为序列化行为失败或通过。

## 失败现象与复现证据

2026-07-11 执行 WSL 环境探针 `wsl.exe -e bash -lc 'uname -a; cargo --version; df -h / /mnt/e /mnt/d 2>/dev/null | sed -n "1,6p"'` 返回 exit 1。WSL 服务报告 `D:\Tools\virtual\wsl\ext4.vhdx` 无法挂载，错误链为 `Wsl/Service/CreateInstance/MountDisk/HCS/ERROR_SHARING_VIOLATION`。因此 Linux 进程未创建，未执行任何 `cargo` 或场景测试。

## 最低共享层根因

最低失败层是本机 WSL2 发行版虚拟磁盘被其他进程独占或存在未释放挂载句柄，属于 Runtime 01 技术栈验证环境，不是 `zircon_runtime_interface::serialization`、场景反射 schema、AssetRef 迁移或 canonical writer。

## 架构修复验收

- 在不终止无关用户进程、不删除或复制 `ext4.vhdx` 的前提下释放 WSL 虚拟磁盘占用，使发行版可正常启动。
- 重新执行 WSL 环境探针，确认 `uname`、`cargo --version` 与磁盘检查实际运行。
- 使用仓库规定的 WSL/CI-parity 验证流程复跑 Plan11 M1 场景 focused tests；只有测试进程实际执行并通过，才可关闭本失败单。
- 若复跑出现 Rust 编译或行为失败，按最低功能 owner 新建或更新对应失败交接，不得混入本基础设施失败。

## 禁止临时方案

- 禁止删除、移动、复制或强制修复用户的 `ext4.vhdx`。
- 禁止用 Windows 接口门通过代替计划要求的 Linux/CI-parity 证据。
- 禁止把“WSL 未启动”记为场景测试通过或失败。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Runtime 01 / Editor 11 M1 | Linux/CI-parity 场景验证环境 | `未执行-WSL虚拟磁盘共享冲突` | 2026-07-11 | WSL 探针 exit 1；`ext4.vhdx` 挂载返回 `ERROR_SHARING_VIOLATION`，Linux 与 Cargo 均未启动。 |

## 修复结果与回传

- 根因：WSL2 ext4.vhdx had a transient sharing-violation mount conflict.
- 架构修复：The conflicting mount handle was released externally; no repository or virtual-disk file was modified.
- 验证：wsl.exe probe now returns exit 0 and prints Linux kernel, cargo 1.94.1, and / /mnt/e /mnt/d capacity.
- 回传：WSL environment is available again; Plan11 owner may run Linux scene tests.
