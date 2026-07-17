---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: sound-kira-root-lockfile-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_plugins/02-sound.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/02
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/sound/runtime/Cargo.toml
  - Cargo.lock
  - zircon_plugins/Cargo.lock
tests:
  - cargo test -p zircon_runtime --lib scene:: --locked --jobs 1 -- --test-threads=1
---

# Plugins02：Sound Kira 依赖与根 workspace lockfile 漂移

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Editor02 M1 default-feature `scene::` broad gate | `OPEN / PLUGINS02 LOCKFILE OWNER` | 受管 reservation `a6fd78754a9d4e5ab1129cb34ee28038`、job `2018b8eb4e0947279734eb2f299dcb9e`、run `dd8042b574804c2b9b7093d7e9b2a30f` 在 target `F:\cargo-targets\zircon-engine\pool\cb1f5e8d9591a8cb3c6c5264bad2b46ff3aeb3896919bb8e3a4c2ee13c32c1cc` 执行原始门禁，6 秒内以 exit 101 终态；Cargo 在编译/测试前拒绝 `--locked`，诊断为根 `Cargo.lock` 需要更新。只读差异确认 `zircon_plugins/sound/runtime/Cargo.toml` 正由 Plugins02 将可选直接 `cpal` 依赖硬切为 `kira = "0.12.2"`，根/插件 lockfile 尚未同步。未运行任何 Editor02 scene 测试，不能据此判定 scene 实现失败。 | Plugins02 owner 同步根与插件 workspace lockfile 后，先验证两套 `cargo metadata --locked` 和 Sound M1 focused gate，再原样复跑 Editor02 broad gate；在此之前 Editor02 父 M1 测试阶段保持 pending。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：`M1` 测试阶段，修复返回后的 fresh default-feature `scene::` 汇总
- 修复责任计划：`docs/plans/zircon_plugins/02-sound.md`
- 交接原因：最低失败位是 Plugins02 当前 `kira` 依赖切换与 workspace lockfile 的原子同步，不在 Editor02 world-sync/inspection owner 内。

## 失败现象与复现证据

原始受管命令：

```powershell
cargo test -p zircon_runtime --lib scene:: --locked --jobs 1 -- --test-threads=1
```

终态为 exit 101，stderr 核心诊断：

```text
Updating crates.io index
error: cannot update the lock file E:\Git\ZirconEngine\Cargo.lock because --locked was passed to prevent this
```

运行未进入 `zircon_runtime` 编译，也未执行任何测试。当前只读 `git diff` 显示
`zircon_plugins/sound/runtime/Cargo.toml` 删除 `cpal-backend`/可选 `cpal`，新增精确
`kira = "0.12.2"`；`Cargo.lock` 和 `zircon_plugins/Cargo.lock` 均未随当前 manifest
发生工作树变更。Plugins02 active Session `plugins02-sound-m1-kira-core-20260717` 已声明三者为其写作用域。

## 最低共享层根因

Sound M1 的依赖硬切只更新了 crate manifest，尚未在同一受管切片中刷新两个消费该 manifest
的 workspace lockfile。根 workspace 的任意 `--locked` Cargo 命令因此在依赖解析阶段失败；
Editor02、Frameworks05 等上层门禁都无法到达自身编译/测试逻辑。

## 架构修复验收

- Plugins02 在同一受管 owner 下同步 `Cargo.lock` 与 `zircon_plugins/Cargo.lock`，两者都必须由当前 manifest 图生成并包含 `kira = 0.12.2` 的一致依赖闭包。
- 根 workspace 的 `cargo metadata --locked --no-deps --format-version 1` 通过，且工作树 lockfile 在命令后零漂移。
- 插件 workspace 的 `cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1` 通过，且工作树 lockfile 在命令后零漂移。
- Plugins02 计划定义的 `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked` 通过，证明 Kira 接入不是仅有 lockfile 文本更新。
- 原始 Editor02 受管命令原样复跑并到达 scene 测试终态；仅该复跑结果可恢复 Editor02 M1 broad-gate 判定。

## 禁止临时方案

- 不得移除 `--locked`、改为非受管 Cargo、手工拼接局部 lockfile，或让 Editor02 提交 Plugins02 的依赖文件。
- 不得恢复已裁决退役的直接 `cpal`/`cpal-backend` 兼容路径来规避 lockfile 同步。
- 不得把依赖解析失败记为 Editor02 scene 测试通过，也不得弱化 Editor02/Plugins02 的既定验收命令。

## 修复结果与回传

Open state: `待 Plugins02 同步双 lockfile、完成 Sound focused 验证并回传 Editor02 原始 broad gate`; no pass is claimed.
