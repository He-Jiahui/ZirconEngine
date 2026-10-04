---
related_code:
  - .codex/coordinator-retirement.json
  - .codex/hooks.json
  - .codex/hooks/zircon_session_sync.py
  - .codex/hooks/pre_tool_use_cargo_guard.py
  - tools/dev/zircon-session.ps1
  - tools/setup/install-session-tray-startup.ps1
  - tools/jenkins/install-coordinator-worker.ps1
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/audits/check_conventions.py
  - tools/jenkins/jenkins_coordinator.py
  - tools/jenkins/tray/
implementation_files:
  - tools/dev/local_cargo.py
  - tools/dev/local-cargo.ps1
plan_sources:
  - "user: 2026-10-02 retire the coordinator, preserve data for Jenkins migration, and remove repository integration requirements"
tests:
  - tools/tests/test_local_cargo.py
  - tools/tests/test_check_conventions.py
doc_type: workflow-detail
---

# 协调器停用与 Jenkins 迁移资料

## 当前工作流

本仓库的旧协调器已弃用。后台服务、托盘、自启动和 Codex 自动登记挂钩停用；普通工作不再要求协调器登记、心跳、路径租约、归因接口、验证票据、候选提交或里程碑关闭接口。旧命令、源码和历史说明保留为迁移资料。

`AGENTS.md`、项目技能及生成的 OpenCode 适配器采用当前规则。保留实际验证、必要审阅、文件归属、原有修改、原生 Cargo/进程锁及提交和外部通知授权要求。独立 Jenkins 协调器使用 [Windows 托盘与激活入口](jenkins-tray.md)；只有当前 `active.json` 及其验收证据核验通过，才按 [jenkins-coordination 技能](../../.codex/skills/jenkins-coordination/SKILL.md) 开始协调器功能测试和有边界的命令提交。全工作区、里程碑和生产迁移仍需各自接受证据，旧票据和本地命令结果不能自动授予这些验收。

## 数据位置

协调器数据库、对象、补丁、票据、回执和历史任务数据保留在 `E:\Git\ZirconEngine\.codex\state\session-coordinator`。

本次迁移归档位于 `F:\ZirconCoordinatorArchive\20261002-01a0f599`：

| 资料 | 用途 |
| --- | --- |
| `migration-database/coordinator.sqlite3` | 停机后复制并通过 SQLite 官方检查点恢复的独立数据库；仅归档副本执行检查点，原数据库保持原状。 |
| `migration-database-verification.json` | 完整性检查、表清单、核心记录数量和状态分布。 |
| `coordinator-before-stop.sqlite3` | 停机前固定 WAL 读快照备份；完整性检查记录在 `before-stop-backup-verification.json`。 |
| `state-parts/part-*.tar.gz` | 原状态目录分片归档，包含数据库、对象、补丁、票据、回执、日志和先前清理证据；所有分片共同组成完整备份。 |
| `frozen-state-archive.json` / `state-part-inventories/` | 分片、源文件数量和原始路径清单；核验见 `state-archive-verification.json`。 |
| `frozen-state-metadata.jsonl` / `frozen-state-links.json` | 原始文件属性清单及链接关系；归档不追随链接到目录外。 |
| `unavailable-bytecode-caches.json` | 初始扫描后源路径已不存在的 10 个历史 Python 字节码缓存及其对应源文件记录。 |
| `changed-candidate-versions.json` / `changed-candidate-versions/` | 27 个扫描期间发生变化的离线候选源码的扫描属性、归档内容及观测到的现文件版本。 |
| `candidate-added-files.json` / `candidate-additions/` | 3 个变动候选源码树中扫描后新增的 10 个文件副本及校验值。 |
| `source/tools/` | 停用前的协调器、托盘、控制端及 Jenkins 试点源码副本。 |
| `pre-retirement-config/` / `rules-before-edits/` | 原挂钩、启动项、任务 XML 和规则副本，含原有修改。 |
| `supplemental-state/` | Windows 应用数据中的协调器日志、挂钩状态及启动项移除记录。 |
| `checksums.json` / `handoff.json` | 归档校验值、停用核验和迁移范围清单。 |

以清单和校验报告判定归档是否完整。运行描述符、凭据及锁文件也保留为历史证据；迁移到 Jenkins 时用其凭据管理替换认证配置，避免将整个数据库作为公开构建产物发布。

初始扫描记录 334,186 个文件。归档核验范围为 334,176 个文件及 805 个空目录；另有 10 个历史离线候选目录中的 `.pyc` 缓存路径已不存在，其对应 `.py` 源文件保留并纳入哈希核验。原始扫描清单和缺失记录一并保留，分片生成时的原始告警回执也保留。

停机数据库与 WAL 的归档内容和冻结时原文件的 SHA-256 一致。离线候选源码树在扫描期间仍有编辑活动：27 个变动文件保存归档与观测版本，另补存 10 个新增文件。归档核验使用已保存版本的实际哈希；这些候选目录属于逐文件采集的诊断资料，恢复时结合版本清单审阅，不自动作为已验收源码或执行队列使用。

## 历史状态的迁移处理

迁移保留原始 ID、来源、哈希、状态、时间和回执，不批量改写历史记录：

| 原数据 | 迁移要求 |
| --- | --- |
| Session、计划与 Failure | 保留归属和规范 Markdown 链接；按当前工作范围重建 Jenkins 项目/任务归属。 |
| Patch、Snapshot、Objects | 保留对象哈希和原始引用。主对象库由 `snapshots.ObjectStore` 保存 zlib 压缩内容，对象地址哈希针对解压后的内容；归档文件的 SHA-256 针对原始压缩文件，两者用途不同。核对对应源码版本后再创建迁移变更。 |
| Validation ticket、Cargo job、Copy | 保留原命令、输入/源码身份和结果，以及 durable task 的取消状态、控制请求和事件；需要重跑的任务生成新的 Jenkins 构建身份并关联旧 ID。 |
| Command、Action、Lifecycle receipt | 保留原结论及未终结记录，结合冻结时的进程和端口证据解释状态。 |
| Review、Milestone、Integration | 保留验收边界和审阅来源；新 Jenkins 验收须提供新流水线实际证据。 |

冻结时实际运行与历史状态分别记录在 `service-stopped-verification.json` 和 `frozen-pending-state.json`。尚未终结的旧生命周期意图保留供迁移审阅，不伪造成功回执，也不自动恢复执行队列。

40 条旧验证票据的原始 `queued` 状态保留为历史字段，其 durable task 均已取消且有已接受的取消控制请求。迁移时结合这些记录判断状态，不能只根据票据表的 `queued` 字段重新入队；核验见 `historical-ticket-cancellation-verification.json`。

## 本地命令证据

独立入口不启动协调器，检查实际输出路径和 35 GiB 空间保留量。所有目标、构建目录、编译缓存及临时编译产物必须物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下。原协调器池保持保留，新入口默认使用单独的 `zircon-local` 命名空间。

```powershell
.\tools\dev\local-cargo.ps1 -DryRun check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 check -p zircon_runtime --locked
.\tools\dev\local-cargo.ps1 test -p zircon_runtime --lib <focused-filter> --locked
```

命令范围和执行节奏遵循 `docs/plans/milestone-validation-policy.md`。原生 Cargo 锁继续生效；入口返回实际 Cargo 退出码。Dry run 不创建目录、不执行编译，也不形成验收证据。Jenkins 请求通过独立入口封存和接受，不能把本地结果升级为 Jenkins 接受回执。

## 恢复方法

本次不恢复旧服务。未来明确授权恢复时：

1. 核对 `checksums.json`、数据库完整性报告和数据包校验结果，选择完整归档；保留当前原始数据。
2. 在隔离目录恢复 `source/tools/`，将全部 `state-parts/part-*.tar.gz` 解包到同一隔离状态父目录；恢复后的子目录为 `session-coordinator`。使用独立数据库副本时，仅用 `migration-database/coordinator.sqlite3`，不要混用另一快照的 WAL/SHM。核对同版数据库 schema、对象引用和源码。当前入口已硬停用，不能仅移除 JSON 标记来恢复服务。
3. 先离线核对记录。若授权需要运行隔离实例，使用归档的启动源码、隔离状态目录和独立端口，禁止自动回放旧队列或写入共享主仓库。
4. 仅在明确授权回切主仓库后，结合冻结的租约、回执和实际进程身份制定恢复步骤。不要删除仍有所有者的锁，也不要用状态改写替代验收。
5. 启动项和任务 XML 作为恢复参考保留；逐项获得恢复授权后再应用，尤其不要自动启用旧的定时清理任务。

旧历史的映射、旧队列的迁移和生产切换继续需要独立验收；托盘及独立命令协调器启用不回放旧服务队列，也不改写本次停用与数据保全证据。
