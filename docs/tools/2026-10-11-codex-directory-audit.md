---
doc_type: implementation-record
created: 2026-10-11
---

# Codex 目录盘点与 Jenkins 生命周期管理

本次交付[管理标准](agent-artifact-management.md)、[项目技能](../../.codex/skills/zircon-artifact-management/SKILL.md)
和[只读盘点工具](../../tools/dev/artifact_inventory.py)。分支、工作树、合并、所有版本操作、
源码准备、文件生成、验证编译、产物复用与清理由 Jenkins 统一负责。AI 只编辑明确授权的
规范文件、准备增量输入并核对回执。已有用户例外授权保留其效力。

[归属与移交清单](E:/cargo-targets/zircon-local/reports/20261011-artifact-governance-01a12af6/artifacts.json)
使用用户 `artifact-lifecycle` 的格式，记录 15 个保留项和 85 个待审项（含旧工作树登记）。
状态为 `needs-review`，原位交给 Jenkins 所有者复核；没有伪造已提交的 request ID。

## 观测范围与结果

读取实际 `.codex/config.toml` 的 `zircon-strict` filesystem 表：**142 个显式 write
授权项，119 项存在，23 项尚不存在**。覆盖授权项的文件与目录，额外细分 D/E/F 的
`zircon-local` 一级内容；完整观测包含 **898,536 个文件、191.401 GiB 逻辑字节**。
一级输出有 91 项。盘点过程中未删除现有目录，也未创建或裁剪工作树。

全量初次观测于 2026-10-11 20:52:41（北京时间）完成，历时 478 秒。
[原始只读观测](E:/cargo-targets/zircon-local/tasks/20261011-artifact-governance-01a12af6/evidence/inventory-01.json)
保留为本次审阅输入；SHA-256 为
`4cf996d3a3f05763cd4dd7e8d1d38c33a006388e8c82d96a12a773e63cb2e133`。
这份本地产物来自用户明确 Jenkins 所有权之前启动的盘点；其路径原位保留、登记移交，
不将它伪装成 Jenkins 生成/验收的产物。最终版工具移除了报告文件写入入口，仅输出 stdout。

配置 SHA-256 为 `b31490a9954ca9cc2c89d006ef98e8b019b2ea1c64114f03500db9c09af031d9`。
本次没有修改 config、AGENTS、Git 数据、Jenkins driver、job 或定时器。
本聊天权限上下文为 disabled/unrestricted，不能声称 strict 已在聊天内强制生效。

| 物理输出命名空间 | 观测文件数 | 逻辑 GiB | 硬链接文件数 | 未追随 reparse point | 读取错误 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `D:/cargo-targets/zircon-local` | 598,076 | 145.785 | 255,611 | 8,142 | 3 |
| `E:/cargo-targets/zircon-local` | 124,817 | 17.244 | 82,456 | 2,508 | 1,321 |
| `F:/cargo-targets/zircon-local` | 33,148 | 23.109 | 632 | 0 | 0 |

逻辑字节不能作为唯一物理占用或可回收量。另有 `.jenkins` 的 109 个读取错误、`tools`
的 2 个读取错误和 Penpot 授权目录的 2 个 reparse point；报告保留这些缺口，未改变 ACL。
这是动态文件系统观测，目录已覆盖不表示全部内容可读或构成一个原子源码快照。

## 需要 Jenkins 接管的内容

| 目录或资源 | 观测 | 管理结论 |
| --- | --- | --- |
| `D:/cargo-targets/zircon-local/dependency-fix` | 48.354 GiB；4,366 个硬链接文件 | 原有维护记录明确缺少 owner/native termination 证明；保护，等待 Jenkins 合法接管 |
| `D:/cargo-targets/zircon-local/win32` | 38.587 GiB | 多构建叶子；核对完整配置、引用和 writer 后由 Jenkins 选择复用或回收 |
| `D:/cargo-targets/zircon-local/windows` | 28.005 GiB | 同上；不得按目录年龄清空 |
| `E:/cargo-targets/zircon-local/win32` | 14.764 GiB | 原维护记录保护项；不自行 adopt 或删除 |
| `F:/cargo-targets/zircon-local/penpot-workbench` | 22.566 GiB；观测到 Cargo 及其上游进程引用 | 活跃产物，保留；由原任务和 Jenkins 核对终结及依赖 |
| `D:/cargo-targets/zircon-local/penpot-public` | 12.541 GiB；服务进程、247,842 个硬链接文件 | 服务与共享依赖，保留；不能视为废弃临时目录 |
| `D:/cargo-targets/zircon-local/penpot-workbench` | 9.655 GiB；发现多个 `before/after/zircon_*` 树 | 源码副本与运行资料混杂；逐项保留来源、差异和依赖，交给 Jenkins 维护 |
| `D:/cargo-targets/zircon-local/astra-current-20261003-01a0f033` | 3.473 GiB、93,934 个文件；含 `foundation-r2/source/zircon_*` 树 | 源码候选；无命令行命中不证明可删除，保留并核对 owner |
| `E:/cargo-targets/zircon-local/penpot-migration` | 1.986 GiB；源码/依赖/生成物混合，有链接与读取缺口 | 不自动认领或按总体积清理 |
| `E:/cargo-targets/zircon-local/source-repairs` | 约 35.8 MiB；含旧 `before/zircon_runtime` 模块树 | 未知源码差异原文保留，准备所有者核对，不复制另一份 |
| `.codex/outbox`、`.codex/sessions` | 全量初次 1,156 / 710 个文件；随后只读复核为 1,162 / 711 | 正在变动；outbox 中发现 52 个源码/脚本扩展文件，sessions 中 178 个，需按用途治理 |
| 三盘的 `zircon-local/zircon-jenkins` | Jenkins 受管资源和历史身份 | 通过现有 Jenkins State/维护接口管理，盘点者不直接清理 |

outbox/sessions 的上述扩展文件主要是 Python 辅助脚本，不能直接断言它们都是生产源码
备份。存在 `.rs`、Cargo 或 `zircon_*` 特征也不独立证明违规，依赖缓存和 Jenkins 封存
输入有合法来源。完整候选目录清单见原始观测；归属/差异未决项登记 `needs-review`。

本会话删除既有产物 **0 个文件，实际回收 0 字节**。早期辅助工具行为检查产生的独占
合成 fixture 已由同步测试收尾删除，未把这部分字节计为存量回收；原 scratch 目录现在为空。
源码候选、共享缓存、Git/Jenkins 状态、退役协调器資料和其他会话的活动产物均保留。

## Jenkins 能力与配置缺口

只读核对时，selected driver 为
`7b94004801313ed7d03b47ba1e4f8b6c9aa4e076ec01e4aed412cf4d458480f5`，runtime 为
`start-1791708207030401800`。

| 所需能力 | 实际证据 | 当前结论 |
| --- | --- | --- |
| 增量输入与共享执行/产物复用 | `zircon-flow`、`zircon-execution` 及现有 State 身份；维护记录保留真实功能回执 | 使用原接口和回执，不能从 job 存在推定所有模板已接受 |
| 多工作树创建、合并和释放 | `workflow-spec.json.scope.createWorktrees=false`；当前共享 checkout 策略；selected driver 未见专用 worktree/branch 条目 | 尚未核验此项能力；要求 Jenkins 管理，不由 AI 自建 |
| 历史工作树登记回收 | `git worktree list --porcelain` 显示 main 之外 11 个 detached/prunable 登记 | 只读登记为候选；由 Jenkins 核对 SHA、独有提交及元数据后处理 |
| 任务结束缓存评估/独占 TEMP 清理 | 仓库有 `workflow/post_task_cache.py`；selected driver 不含它；运行 State 的 `postTaskCache` 缺省 | 有支持层源码，未在当前运行版本启用；不能宣称已自动清理 |
| 定时维护 | `zircon-flow`、`zircon-execution`、`zircon-maintenance`、`zircon-gap` 的磁盘 job XML 均无 trigger；State 无 scheduledMaintenance 策略 | 当前证据不能证明周期清理已启用；需核对实际 Jenkins API及获准触发配置 |
| 权限边界 | config 实际授予 `.git`、`.jenkins` write 且 network=true，与部分注释/权限文档不一致 | 行为规则已限制 AI；隔离的运行时写权限仍需单独设计和实测 |

没有修改 job 配置、开启定时任务、调用 GC intake 或恢复退役服务。磁盘 job XML 是本地
配置观测，不等于远端 API 的实时状态。多工作树能力须使用明确输入 SHA 和目标树，
验证合并结果并保持各作者范围；清理须使用原生终结、引用清单和实际删除回执。

Jenkins 后续验收顺序：核对版本/工作树管理入口 → 核对受管文件生成和完整 input 身份 →
核验已有构建池复用及原 writer → 部署并接受任务收尾维护 → 按授权配置周期维护 →
逐个处理遗留目录和工作树登记。支持层修复、部署和真实运行回执分别记录；本次规则
交付不代表这些迁移门槛已通过。

## 本次交付验证

只读工具的 10 项行为检查通过：stdout-only、输入不变、reparse 不追随、硬链接的逻辑
统计、读取失败、路径/保护分类、进程参数不外泄。新技能通过 skill-creator 的结构校验；
项目 catalog 与本地链接按当前源条目校验。没有执行 Rust 编译、Git 写操作或存量 GC。

新增项目技能与 `tools/tests/test_artifact_inventory.py` 命中现有 ignore 规则；交接须通过
精确新增路径/哈希包含它们，不能依赖整个 checkout diff 或自行修改 ignore/Git index。
`.opencode/skills` 当前不存在，现有技能维护器只维护 Codex catalog；未创建替代适配器。
