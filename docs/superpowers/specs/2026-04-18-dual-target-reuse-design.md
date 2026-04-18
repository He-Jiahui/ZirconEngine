# Dual Target Reuse Design

## Context

`zirconEngine` 当前的 Cargo 验证经常依赖手写 `CARGO_TARGET_DIR=target/...` 或 `--target-dir target/...`。这能避免不同验证面互相污染，但也带来了两个持续问题：

- 会话级 target 命名不断增长，`target/` 下产物无限累积，磁盘占用越来越大。
- 同一条 Codex 线程在反复执行 `validate-matrix.ps1` 时没有稳定复用规则，热缓存收益不可预测。

仓库里现有正式验证入口是 [`.codex/skills/zircon-dev/scripts/validate-matrix.ps1`](E:/Git/ZirconEngine/.codex/skills/zircon-dev/scripts/validate-matrix.ps1)，因此 target 复用规则必须首先落在这个脚本里，并由 `zircon-dev` skill 文档统一说明。

## Goals

- 把默认自动 target 目录限制为两个固定 slot，而不是无限生成新名字。
- 让同一条 Codex 线程优先复用自己已经占用的 slot，保留热缓存收益。
- 当第一个 slot 被其他近期活跃线程占用时，自动切到第二个 slot。
- 当两个 slot 都被其他近期活跃线程占用时，明确失败，不隐式创建第三个 slot。
- 让 `zircon-dev-validation` 文档和手工命令约束与脚本行为一致。

## Non-Goals

- 不修改历史文档中已经记录的旧 `target/...` 命令。
- 不尝试为仓库里所有独立 cargo 命令做全局拦截；本轮只收敛 `validate-matrix.ps1` 和相关 skill 说明。
- 不引入后台守护进程、系统级文件锁服务或周期清理任务。

## Constraints

- 仓库策略要求继续在现有 `main` checkout 上工作，不创建 worktree 或 feature branch。
- 默认行为必须对 Codex Desktop 友好；当前环境已提供稳定的 `CODEX_THREAD_ID`。
- 脚本仍需要支持显式覆盖 target 目录，避免复杂场景被自动策略锁死。

## Design Summary

默认验证路径改为双 slot 自动分配：

- `target/codex-shared-a`
- `target/codex-shared-b`

脚本通过 `.codex/tmp/cargo-target-slots/` 下的 lease 文件维护 slot 归属。lease 记录当前 owner、最近活跃时间和对应 target 目录。`validate-matrix.ps1` 每次运行时都会刷新 lease，并在调用 `cargo build/test` 时自动附带 `--target-dir <selected-target>`.

如果用户显式传入 `-TargetDir`，脚本跳过双 slot 自动分配，直接使用指定目录。

## Slot Ownership Model

### Owner identity

默认 owner identity 采用以下优先级：

1. `CODEX_THREAD_ID`
2. 回退到 `"manual:<user>@<machine>:<repo-root>"` 指纹

设计意图：

- 在 Codex 会话中，同一线程多次运行脚本时始终命中同一 owner id。
- 在没有 `CODEX_THREAD_ID` 的普通 PowerShell 中，脚本仍可运行，但会退化为较粗粒度的人工共享 owner；需要更细粒度隔离时，用户应显式传 `-TargetDir`.

### Lease files

每个 slot 对应一个 JSON lease：

- `.codex/tmp/cargo-target-slots/slot-a.json`
- `.codex/tmp/cargo-target-slots/slot-b.json`

lease 字段：

- `slot_name`
- `target_dir`
- `owner_id`
- `owner_pid`
- `claimed_at_utc`
- `last_seen_utc`
- `host_name`
- `repo_root`

### Staleness

lease 在以下任一条件下视为可回收：

- lease 文件不存在
- lease 内容损坏且无法解析
- `last_seen_utc` 超过 12 小时
- lease 的 `repo_root` 与当前仓库不匹配

12 小时 TTL 的取舍：

- 足够覆盖同一天内的反复验证，保留 cache 价值
- 不会让已废弃线程永久占用 slot

## Allocation Algorithm

默认自动分配流程：

1. 如果传入 `-TargetDir`，直接解析并使用该目录，结束分配流程。
2. 读取两个 slot 的 lease。
3. 如果某个 slot 的 `owner_id` 等于当前 owner，直接复用该 slot。
4. 否则，优先尝试第一个可回收 slot；如果 `a` 可回收则拿 `a`，否则尝试 `b`。
5. 如果两个 slot 都被其他活跃 owner 占用，脚本报错并退出：
   `Both shared cargo target slots are occupied by other active sessions. Pass -TargetDir to override.`

这样可以满足目标规则：

- 当前线程已有 slot 时复用
- 当前线程没有 slot 且第一个 slot 被别的会话占用时，自动切第二个
- 不再创建第三个默认产物目录

## Locking Strategy

为避免两个会话同时抢占同一 slot，脚本在读取和写入 lease 前获取一个短生命周期全局锁目录：

- `.codex/tmp/cargo-target-slots/.lock`

锁规则：

- `New-Item -ItemType Directory` 成功即视为拿到锁
- 短轮询等待
- 超时后报错，避免无限卡住
- `finally` 中释放锁目录

这里不追求跨机器锁；仓库 checkout 本身就是单机路径，目录锁足以覆盖本地并发会话。

## Script Surface Changes

[`.codex/skills/zircon-dev/scripts/validate-matrix.ps1`](E:/Git/ZirconEngine/.codex/skills/zircon-dev/scripts/validate-matrix.ps1) 将新增：

- `-TargetDir` 参数，允许显式覆盖自动 slot 分配
- target 解析与 lease 管理辅助函数
- 在输出中打印本次 target 来源，例如：
  - `Auto target dir: ... (reused current thread slot a)`
  - `Auto target dir: ... (claimed stale slot b)`
  - `Manual target dir: ...`

`Get-CargoArgs` 也会统一附带 `--target-dir`.

## Skill Documentation Changes

### zircon-dev-validation

[`.codex/skills/zircon-dev/validation/SKILL.md`](E:/Git/ZirconEngine/.codex/skills/zircon-dev/validation/SKILL.md) 需要增加：

- 默认通过 `validate-matrix.ps1` 执行 Cargo 验证
- 默认脚本只会在两个共享 target slot 中复用，不会无限新建
- 需要独立 target 时使用 `-TargetDir`

### manual-commands

[`.codex/skills/zircon-dev/validation/manual-commands.md`](E:/Git/ZirconEngine/.codex/skills/zircon-dev/validation/manual-commands.md) 需要增加：

- 手工命令优先复用 `target/codex-shared-a` 或 `target/codex-shared-b`
- 不要再随手发明新的 `target/<ad-hoc-name>`
- 如果必须做隔离试验，显式注明临时 target，并在任务结束后自行处理

### zircon-dev root skill

[`.codex/skills/zircon-dev/SKILL.md`](E:/Git/ZirconEngine/.codex/skills/zircon-dev/SKILL.md) 可补一条高层说明，把默认验证入口与双 slot 复用策略挂钩，减少子技能之间的漂移。

## Testing Strategy

脚本修改采用 PowerShell TDD：

1. 先为 slot 选择逻辑抽出可测试函数。
2. 用 Pester 写 red tests，覆盖：
   - 当前 owner 复用自己的 slot
   - 第一 slot 被其他活跃 owner 占用时切到第二 slot
   - stale lease 会被回收复用
   - 两个 slot 都活跃时抛错
   - `-TargetDir` 手工覆盖时跳过自动分配
3. 再补最小实现让测试转绿。
4. 最后跑脚本 dry-run，确认实际命令行包含 `--target-dir`.

## Risks And Mitigations

- `CODEX_THREAD_ID` 缺失时 owner 粒度较粗
  - 缓解：保留 `-TargetDir` 手工覆盖入口，并在文档里说明。
- 旧版 Pester 能力有限
  - 缓解：把核心分配逻辑写成纯函数，避免依赖复杂 mocking。
- lease TTL 过短会抢占仍在使用的 slot，过长会拖慢回收
  - 缓解：先固定 12 小时，后续再按实际使用反馈调整。

## Acceptance

本设计完成后的可验收结果：

- 默认运行 `validate-matrix.ps1` 时总是带 `--target-dir`
- 默认自动 target 只可能落在 `target/codex-shared-a` 或 `target/codex-shared-b`
- 同一 Codex 线程重复运行脚本时能稳定复用同一 slot
- 两个 slot 被别的活跃线程占满时脚本明确报错，不再生成第三个默认目录
- `zircon-dev` 相关 skill 文档明确说明这套规则
