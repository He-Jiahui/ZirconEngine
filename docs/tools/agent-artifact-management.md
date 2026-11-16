---
doc_type: workflow-detail
created: 2026-10-11
updated: 2026-11-16
---

# Agent 直接开发、版本流程与产物管理

## 当前默认与权限

Agent 在任务授权内直接管理源码、原生 Git/worktree、生成、编译验证、产物复用与收尾。使用任务工作树和项目的独立本地工具。

Full Access 仅改变命令执行权限，不扩大任务范围。明确的只读、计划、审批或禁止提交检查点仍有效。

提交、回滚、集成及清理遵守各自的实际授权。已有明确的精确任务提交授权可直接复用；完成里程碑本身不授予提交权限。推送、创建或合并远端 PR、发布及外部通知仍需对应授权，本地提交许可不包含这些动作。

## 源码、Git 与工作树

- 遵循[checkout policy](../../.codex/skills/zircon-dev/references/main-branch-development-policy.md)。共享 `main` 用于同步和检查；普通实现放在任务专用分支/worktree。Agent 可在授权范围内用原生 Git 建立和管理它们，明确指定的 checkout 优先。不修改或清理脏共享原根来创建一个“干净”任务。
- 记录 repository、worktree、branch、不可变 base SHA、精确任务路径及既有修改。检查可能重叠的 chats、diff 和交接；保留其他作者的工作区、暂存及未跟踪内容，也保留同文件内的外来修改。
- 每次 Git 写操作前用 `git rev-parse --absolute-git-dir` 定位真实 Git 目录，检查该树的 `index.lock` 及当前索引归属。linked worktree 的 `.git` 是文件。未知锁保持原状；年龄、空字节或暂时无句柄均不能证明可删锁。
- 获准提交时只暂存、核对并提交任务的精确文件/增量，读取实际提交 SHA 后才报告成功。不使用整树 `add`、reset、stash、clean 或批量提交吸收/清除外来工作。不要给普通提交附加整树哈希对账或完整编译门；仍须如实报告验证失败和未完成验收。
- 集成前固定输入 SHA、目标和授权范围，保留冲突，不强制覆盖；验证实际结果树，不将联合树成功归给未验证子集。共享 `main` 只在同步前后要求干净。远端操作不因分支已提交而自动获准。
- 工作树是原生 Git 管理的源码位置，不是备份或临时验证副本。不要另造源码快照、选中文件备份、源码压缩包或 `.txt`/证据 JSON 中的完整前后源码。需要额外源码复制时先取得明确范围授权。移除工作树前确认 writer 已终结、独有修改已交付且该移除动作获准；未知树保留。

## 生成、编译与复用

Agent 直接执行任务所需的生成工具、资源导出、截图、日志和验证命令，记录输入、工具版本、实际路径及结果。生成源码仅写获准的规范路径；非源码输出使用明确的任务所有者和允许的输出命名空间，不随意创建 `tmp`、`backup` 或重复源码树。

编译范围和节奏遵循[里程碑验证政策](../testing-and-validation/milestone-validation-policy.md)与[验证指南](../../.codex/skills/zircon-dev/validation/guide.md)。每个可编译命令前声明模块、包、features、targets/filters、profile、命令及覆盖理由。普通工作关闭默认 features、选择最小受影响集合；文档要求的显式 workspace/package-set gate 按该政策声明。广域构建限制不变；实现切片不默认运行 Cargo。

使用 `tools/dev/dev-module-build.ps1 -Package <package>` 的受支持选择，或 `tools/dev/local-cargo.ps1` / `python -B -m tools.dev.local_cargo` 执行独立本地命令。需要 Python 时使用通用解释器。入口缺少能力或命令失败时报告实际限制；不启动服务、改历史状态或伪造通过来修饰结果。

所有编译产物、构建目录、编译器缓存及临时编译输出必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 的获准命名空间。拒绝 C 盘、repo-local `target`、旧 `targets`/`ZirconBuilds`、嵌套同名根、路径遍历和别名。保留物理身份检查、35 GiB 空间余量及原生 Cargo 锁。普通 dev/test、诊断和交付 profile 的存储约束见[编译存储规则](build-storage-policy.md)。

复用必须匹配仓库/源码输入、平台、工具链、架构、profile、features、链接及有效环境。兼容共享产物沿用排他 writer/原生锁；不要按会话或worktree复制完整 target，也不要搬动活动池。不兼容配置分开，不能用复制旧 EXE/PDB 充当新结果。最终交付包、符号、共享构建产物和增量缓存分别计量。

## 独立证据、异步工作与收尾

记录生成或验证的实际输入、命令、物理输出、退出码与日志。普通本地 Cargo 回执只记录观察到的根进程结果，不证明进程树终止、输入封存或产品产物来源；产品门要求的字节/哈希、原生终态等证据须由该门单独核验。可使用用户的 [Artifact Lifecycle](D:/CodexData/.codex/skills/artifact-lifecycle/SKILL.md) 清单格式；清单用于归属与保留，不授予删除或验收权限。

任务证据可放在获准的 `cargo-targets/zircon-local/reports/<task-id>/`，记录 owner、checkout、路径、依赖和保留条件。已有产物记录真实原位置，不为统一布局重新复制。`.codex/outbox` 仅放获准补丁/交接 metadata，`.codex/sessions` 仅放必要交接与证据链接；不另建协调器、租约库、后台队列或自动清理服务。

异步操作遵循[回执生命周期](../../.codex/skills/zircon-project-skills/cross-session-coordination/references/receipt-lifecycle.md)：保留原 job/request/attempt，继续独立工作，结果不明先对账，不能换身份重复执行。启动、queued、dry run、客户端超时和 cancel_requested 均不等于实际完成。需要验证原生进程树终态时，核对原进程/Job 及 stdout/stderr EOF，不能以 PID 或终端窗口消失代替。

独立命令结果只覆盖它实际执行的源码和范围。里程碑、全工作区、产品窗口/PNG、性能、设备和发布验收仍按各自门槛记录；窄命令成功不能宣称广域通过。历史迁移验收保留原状态。

收尾时核对生成、验证、提交/集成与交付各自的实际状态，报告失败、保留项及待处理验收。清理只针对已授权的精确任务产物，先核验物理路径、所有权、保留引用、原生终态和锁。未知源码、共享依赖、外来产物、Git元数据、历史回执及仍有引用的数据保持保护；目录年龄、增长和缺少manifest不是删除依据。维护失败单独记录，不重写源码或提交的原结论。不得因任务结束自动 `cargo clean` 或创建定时维护任务。

## 历史资料与只读盘点

Jenkins 专属工具、部署配置、凭据、数据库、队列、构建历史和旧协调器数据保留。本文改变的是仓库开发默认，不表示任何主机服务已停止、卸载或数据已清除；服务/启动项/凭据变更和历史请求恢复需要单独明确授权。不重放未知队列，不启动旧本地协调器。历史操作说明见[原 Jenkins 指南](../../.codex/skills/jenkins-coordination/guide.md)与[协调器退役资料](../tooling/coordinator-retirement.md)。

[盘点工具](../../tools/dev/artifact_inventory.py)只读目录并向stdout输出JSON，不移动或删除内容，不创建源码副本：

```powershell
python -B -X utf8 -m tools.dev.artifact_inventory
```

盘点需使用当前授权的精确范围。逻辑字节不等于唯一物理占用或可回收字节；源码特征不等于违规副本。无归属内容标为 `owner-review-required`，保留精确路径、用途、来源、差异/哈希、引用和已知writer，交由实际owner核对。盘点输出不是删除、原生终结或接受证明；`zircon_runtime/build` 是规范源码，不得按目录名回收。
