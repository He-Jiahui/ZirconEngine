---
related_code:
  - .jenkins/deployment-spec.json
  - .jenkins/plugins.lock
  - .jenkins/workflow-spec.json
plan_sources:
  - "user: 2026-10-02 repository-local Jenkins runtime and sole build entry with identical-input artifact reuse"
  - docs/cli-and-tooling/jenkins-session-coordination-architecture.md
tests:
  - "JSON, plugin inventory, approved-path and Git ignore structural checks"
doc_type: deployment-specification
---

# Jenkins 工程内部署配置

当前为 `runtime-offline-recovery-pending`。Home、专用 JDK／WAR／Python、79 个插件和托盘已迁入工程 `.jenkins`；待启动地址为 `http://127.0.0.1:18080/`。旧端口 53748 落在本机 Windows 排除区间，启动器已增加端口预检。三类 job 已创建，纯注释、模块修改、跨模块修改和失败修复已有历史 Jenkins／Cargo 回执。同输入共享、后续消费者重新测试及有界 GC 也已实测；源码变化后的增量、独立并行、联合输入和最终入口切换仍待验收。旧协调器持续退役。

根据用户于 2026-10-04 的最新部署要求，Jenkins 流水线的封存输入、构建工作区、Cargo target、编译器缓存、临时文件和大型产物统一放在工程内 `E:\Git\ZirconEngine\.jenkins\builds\zircon-jenkins`。该要求覆盖此前计划中使用驱动器根级 `D:\cargo-targets` 的 Jenkins 构建默认值；此前 D 盘路径仅作为历史执行身份和回执证据保留，不能继续作为当前 Jenkins 默认路径。运行组件仍使用 `.jenkins`，Home 仍为 `.jenkins\jenkins_home`。

最新并行验收暴露生命周期 broker 的完成路径错误，导致宿主及 Jenkins 退出。错误已修复；完整回归与封存驱动的状态见 `.jenkins/state/migration-baseline/acceptance-index.json`。遗留执行缺少可信的全树终止证明，当前保持 writer 和预约保护，服务恢复需核验真实 Windows 重启边界。历史通过回执保留其原 driver 身份，不代表最新驱动已经完成运行验收。

正式实施顺序、测试阶段与回退门槛以[单机多 Session 实施计划](../docs/plans/jenkins-session-coordination/01-single-host-jenkins-coordination.md)为准；模块、状态、共享执行与资源协议见[详细架构方案](../docs/cli-and-tooling/jenkins-session-coordination-architecture.md)。新支持模块已接入归属、准入、进程和验收协议；旧 owner 仅保存在历史参考字段中，不能恢复或绕过旧 runtime。

流程目标见 [workflow-spec.json](workflow-spec.json)。它包含步骤契约、组合模板、单机资源选择、冲突与失败处理，以及正式接入的验收案例；当前状态为 `implemented-runtime-acceptance-pending`。多机器协调暂缓，前置功能验收后再考虑。

| 变更或任务类型 | 所选流程 |
| --- | --- |
| 经证明的纯注释 | 补丁 → 语法 → 格式 → 验收 → Git 提交 |
| 模块内部修改 | 补丁 → 语法 → 格式 → 按需构建 → 模块单测 → 验收 → Git 提交 → 消息 → GC |
| 跨模块或公共契约修改 | 模块流程加受影响接口与调用方的集成测试 |
| 失败修复 | 模块流程加原失败回归及受影响的上层验证 |
| 缓存维护 | 独立的受管 GC 流程，记录实际回收量 |

组合器按可信变更分类和覆盖范围生成依赖图；步骤可以独立调用，但必须携带相同的输入身份、前置证据和权威回执。恢复时只复用仍匹配的回执，失配后重新执行受影响步骤。语法校验使用对应语言的解析器，不以 Cargo 编译代替轻量语法检查。Rust doctest 或影响构建的注释指令不能进入纯注释模板。未知影响范围先解析，不自行跳过验收或默认重建全部 workspace。

首版选用 Scripted Pipeline，通过固定版本的 `load` 组合步骤；flow、execution 和 maintenance 三类 job 分别承担任务、共享执行和维护。已启用的插件包含 Shared Library、Declarative 和 pipeline-build-step，首版流程继续采用 Scripted Pipeline；共享执行通过受控 CLI 与 Jenkins 参数化 API 提交。Pipeline 等待资源时退出 node、释放 executor。[Jenkins 的 load 步骤](https://www.jenkins.io/doc/pipeline/steps/workflow-cps/)提供加载依据。三个 job 已创建并读取内容寻址的封存步骤库；实际流程接受仍须通过 M3–M8。

保持一个共享开发 checkout，补丁以新状态库路径归属及修改前 hash 排他应用，Git 集成保留外来编辑和暂存区内容。编译读取稳定池中的封存输入；不新增 Git worktree，也不按 Session 创建 target。相同完整输入共享独立执行任务；不同输入仅在作者、覆盖及联合验证视图核对后合批。独立模块并行消耗同一 CPU／内存预算。

增量入口 `patch register` 接收 UTF-8 Git unified diff、每个改动文件的 `beforeHashes`、声明依赖、覆盖范围及已有会话授权，登记时不修改 checkout。`patch submit` 通过认证 API 向 `zircon-flow` 发送 `PATCH_REQUEST_REF`；Pipeline 的首阶段核对驱动、Git HEAD、依赖及前后 hash，排他应用补丁后自行封存输入，再登记完整构建身份。冲突时阻断，结果不明时对账原请求，禁止模糊套用补丁或换编号重试。部分应用或封存失败不进入编译；恢复按原 journal 和 hash 保护外来编辑及活跃 execution／native 引用。对应接口及测试已实现，真实 Jenkins 增量入口仍待恢复后运行验收。合并补丁不授予 Git 提交、推送或通知权限。

历史 `validation_reuse.py` 区分终态结果和 preparation：终态复用匹配完整源码及覆盖；源码改变可继续使用兼容的增量池。历史 `validation_groups.py` 和 `validation_priority.py` 分别提供合批与持久化公平队列的设计和回归参考。新支持库只提取必要策略与受控操作；Jenkins 流程接管步骤调度，源码归属、租约、准入和验收保持单一权威。

构建范围从变更及依赖图推导，只选择必要 package、target、features 和测试。测试需要编译时由 managed test 计划处理；无需无条件先执行一次 `cargo build` 再完整编译测试。[Cargo 的 `--no-run`](https://doc.rust-lang.org/cargo/commands/cargo-test.html)支持分离测试编译与执行，但正式拆分仍须保留受管的测试回执和终止证明。

构建位置先判定合法产物／结果复用，再比较热池、合法新池和有界 GC 的预计验收时间；库存不完整或预算不足就等待。CPU 紧张时限制下一 recipe 的 Cargo jobs 和测试线程，使用持久化公平与老化；资源预约原子建立，取得 agent 后才申请重步骤资源。热增量池保留，单体体积按用途与真实测量优化，禁止自动降低验收配置。

补丁、校验或测试失败后，阻断后续验收和提交，证明执行进程终止，再按修改前后 hash 仅补偿本任务拥有的改动。出现其他 Session 后续修改时停止补偿并保留证据；已提交改动采用获准的正向 revert。消息或验收后的 GC 失败单独重试，保留已接受源码和提交。

Git 提交和消息步骤使用会话已有的明确动作授权。纯注释模板不包含消息或 GC；新提交实现不得继承历史 `integration_candidates.py` 的隐式通知副作用。通知交付不明时先对账，不能通过重新提交 Git 来重发消息。

接入顺序为：步骤契约与分类 → 纯注释正式验收 → 模块按需验证 → 结果及增量复用 → 单机资源调度 → 多 Session 合批与恢复。步骤、调度器和回滚适配器已接入；此前 B03 驱动及旧源码版本的完整支持层回归覆盖 217 项，其中 216 项通过、1 项环境跳过，该记录是历史证据，不能直接代表当前目录策略或新驱动已经完成全量回归。针对本次工程内 `.jenkins\builds` 存储策略及相关入口，当前源代码绑定的聚焦回归覆盖 62 项，其中 61 项通过、1 项环境跳过；该结果同样只证明已覆盖的聚焦范围。尚未关闭的运行门槛保留在规格和计划中，静态检查不能替代端到端验收。

| 用途 | 目标路径 |
| --- | --- |
| Jenkins Home：配置、用户、密钥、插件包、任务及构建记录 | `E:\Git\ZirconEngine\.jenkins\jenkins_home` |
| 控制器日志、进程状态 | `E:\Git\ZirconEngine\.jenkins\logs`、`state` |
| agent 控制工作目录与 remoting 缓存 | `E:\Git\ZirconEngine\.jenkins\agent`、`cache\remoting` |
| Jenkins 解包缓存与 JVM 临时文件 | `E:\Git\ZirconEngine\.jenkins\cache`、`tmp` |
| 封存输入、编译工作区、Cargo target、编译器缓存与二进制产物 | `E:\Git\ZirconEngine\.jenkins\builds\zircon-jenkins`；当前 Jenkins 不接受 D/E/F 盘根级替代路径 |

`.gitignore` 仅放行本目录的规格、版本清单与说明；Home、密钥、日志和状态排除在 Git 之外。运行目录迁移时保留原用户与加密密钥，不把密码或 token 放入这些配置文件。

`plugins.lock` 锁定 79 个目标包的版本；`plugin-checksums.json` 记录 SHA-256、最低 Core 和依赖闭包。包校验和此前启动时的逐项启用状态已核对；当前离线，启用状态须在恢复后复验。控制器版本为 2.580.1，Java 为 Microsoft 21.0.12.1。

部署应用需将 `JENKINS_HOME` 设置为表中工程路径，保持 controller 零执行槽；agent 的控制工作目录留在工程 `.jenkins\agent`，编译进入 `E:\Git\ZirconEngine\.jenkins\builds\zircon-jenkins`。启动时显式配置 `java.io.tmpdir`、`hudson.PluginManager.workDir` 和 WAR 的 `--webroot` 到 `.jenkins` 批准目录内，并将 agent 的临时目录、工作目录及 remoting 缓存放在批准目录内。Controller 的 `jenkins.model.Jenkins.workspacesDir` 只控制 controller workspace，不能代替 agent 的目录设置。[Jenkins 系统属性](https://www.jenkins.io/doc/book/managing/system-properties/)与[WAR 启动说明](https://www.jenkins.io/doc/book/installing/war-file/)提供配置依据。

所有构建目录须先登记 managed 存储所有权、租约及索引。启动和每次 Cargo 准入均须校验真实物理路径必须等于 `E:\Git\ZirconEngine\.jenkins\builds` 下的 `zircon-jenkins` namespace；拒绝 C 盘、工程内 `target`、D/E/F 盘根级 cargo-targets、嵌套仿冒根、junction/symlink 或其他别名。新检查器及进程回执正在验证；规格布尔值不能代替真实路径准入证据。

## 唯一入口与产物复用要求

后续构建提交统一进入 Jenkins；工具入口须转发同一请求身份，新正式准入层校验调度身份与预约。新支持库接管源码归属、Cargo 准入、native 终止证明和正式验收的必要职责；Jenkins 的 SUCCESS 只有在这些证据核对通过后才形成正式接受结果。

入口转发保留 State 中已登记的原请求载荷，辅助发送记录须匹配其源码、覆盖及引用身份；表单参数不能替换这些身份字段。默认使用正式部署的 API token 通道。Flow 保留历史发送编号，结果不明或载荷不匹配时先对账，不能换编号重新发送。已登记请求复用、身份替换拒绝、托管认证和历史未知发送保护已有支持层回归证据；完整命令、功能矩阵、产物及调用方接受结果仍属于 M8 运行验收范围。

产物身份由封存源码及外部／生成输入、依赖锁定、工具链和 wrapper、host/target、完整命令、profile/features 与影响构建的环境共同确定。同身份只有一个执行者，其他请求消费登记产物；失败和部分输出不发布为可复用成品。兼容源码版本复用 PreparationKey 池，默认重新运行测试；只有证明固定输入、环境和有效性匹配的测试结果才允许复用。

产物本体由新支持库持有。Home 的 archive/stash 只保存小型 metadata、必要日志、receipt 和产物引用；大型封存输入、编译二进制、Cargo target、编译器缓存和构建临时目录统一位于 `E:\Git\ZirconEngine\.jenkins\builds\zircon-jenkins`。可变增量工作集与不可变成品可能同时驻留，实际空间核算包含两者；多个 Session 只保存引用。Jenkins 记录删除与实际 GC 分开核对，活跃引用和身份不明对象受到保护。

## 应用与验收边界

正式切换前按当前里程碑计划完成新支持库、真实 Cargo、恢复与复用验收；保留外来改动、历史状态及原有账号。若需迁移既有 Home，drain 并证明原生进程终止、停止 controller 后再复制一致内容。[官方 Home 迁移说明](https://www.jenkins.io/doc/book/managing/system-configuration/)提供运维依据。

最终核对规格加载、路径检查和统一准入，以及 Home、插件、agent、同输入复用、取消／重启及 GC 回执。实现已接入，唯一入口约束尚未启用；历史试点仅供参考，不能替代新计划 M0–M8 的运行验收。

Windows 托盘读取正式部署规格，提供状态、打开页面、启动、停止、重启和日志；退出托盘保留 Jenkins 服务。CLI 使用工程专用解释器，例如 `.jenkins\runtime\python\python.exe -B -m tools.jenkins.deployment --spec .jenkins\deployment-spec.json health`。启动、停止及插件维护会核验持久化的服务身份，不采用 PID 单独判断归属。

`failed` 或 `stopped` 标签不能单独许可再次启动。正常重试须核对与该 Home 及实例标识相符的 controller／agent 全树终止和日志 EOF 证明；只有明确记录尚未尝试启动 agent 时，才允许省去其证明。证明缺失时继续保护原实例，恢复入口按真实启动边界对账。

执行宿主位于具名外层 Job，阶段子进程位于嵌套 Job。阶段命令退出后，先清理其自有残留子进程并核验全树终止、日志 EOF，再登记输出。宿主异常只能凭可信 Job 终止证明形成失败恢复；证明遗失时保留隔离状态。真实 Windows 重启后，恢复器核对所有旧组件的出生时间早于 `LastBootUpTime`，仅解除失败执行的保护并按 hash 补偿，不生成成功验收或可复用成品。具名句柄和嵌套终止语义见 [Microsoft Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)；启动时间字段见 [Win32_OperatingSystem](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-operatingsystem)。

旧 `boot-recovery-preparation.json`、`resume_after_windows_restart.py` 和登录恢复包装器保留为历史记录，其原启动操作与测试源码绑定已过期，不能直接用于本次恢复。一次性 RunOnce 恢复项已移除，移除回执位于 `.jenkins/state/deployment/recovery-startup-removal-1791091770606998500.json`；当前没有已核验的新登录恢复项。

新恢复入口为 `.jenkins\runtime\python\python.exe -X utf8 -B .jenkins\state\migration-baseline\start_prepared_patch_driver.py`。准备器 `prepare_patch_driver.py --regression <passing-receipt>` 要求完整 pytest 发现、源码稳定及输出哈希核对通过，再封存步骤库和支持代码，生成 `.jenkins/state/deployment/prepared-driver.json` 指向的准备记录，不切换当前驱动。启动入口默认只检查；合法终止证据满足后加 `--start` 才可选择准备好的驱动、启动并核对 controller、agent、79 个插件和三类 job。启动继续保持 quiet，历史执行预约及 writer 保护不解除，不重发未知请求，不生成编译接受结果。

最新失联部署 `start-1791091874631246000` 的 host／controller 出生时间晚于当前 Windows 启动时间，且缺少完整终止与 EOF 证明。因此仍需真实整机重启边界；服务启停授权不包含 Windows 重启。当前聊天的系统启动时间查询会被严格权限拒绝，启动器返回 `boot_evidence_unavailable`；受批准的只读系统查询只用于诊断，不替代恢复器自身的可信证据，不授权在沙箱外启动服务。目录授权、静态回归及准备记录均不代表 Jenkins 已上线。

M8 当前入口审计记录于 `.jenkins/state/entry-inventory.json`，逐项绑定源码哈希，暂覆盖 22 处已知入口及子执行路径。主要 Python／PowerShell 入口已有转发适配；运行时特性检查、Rust 产品构建、Export／SourceTemplate、Tauri 原生构建及两个 hosted CI 流程仍有直接执行或待迁移路径。产品 target／profile／features、生成输入、产物发布及调用方的接受结果还须完整对齐。现有 hosted CI 位于本机 controller 的边界外，多机器及 GitHub 自动触发继续暂缓；这些记录保持 M8 待验收，不能通过修改清单开启唯一入口。
