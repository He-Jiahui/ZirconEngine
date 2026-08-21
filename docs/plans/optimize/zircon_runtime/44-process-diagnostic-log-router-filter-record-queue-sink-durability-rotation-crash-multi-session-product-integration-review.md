---
related_code:
  - zircon_runtime/src/diagnostic_log
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling
  - zircon_app/src/bin/editor.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_editor/src/core/logging
  - zircon_plugins/plugin_sdk/src/native.rs
tests:
  - zircon_runtime/src/diagnostic_log/diagnostics/tests
  - zircon_runtime/src/diagnostic_log/sink/tests
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/product_teardown.rs
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-19-diagnostic-log-synchronous-sink.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDeviceRedirector.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceRedirector.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDeviceFile.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/OutputDeviceFile.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/OutputDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Misc/OutputDeviceRedirectorTest.cpp
  - dev/bevy/crates/bevy_log/src/lib.rs
  - dev/godot/core/io/logger.h
  - dev/godot/core/io/logger.cpp
  - dev/godot/tests/core/io/test_logger.cpp
  - dev/Fyrox/fyrox-core/src/log.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 44 · Process Diagnostic Log Router / Filter / Record / Queue / Sink / Durability / Rotation / Crash / Multi-Session 工程化差距

## 1. 结论

`zircon_runtime::diagnostic_log`已经具备可保留的异步底座：过滤规则会预编译，lazy API能避免被过滤消息的格式化，普通消息进入有界队列，worker支持批处理，flush和shutdown有显式命令，sink暴露基础计数器，动态Runtime用session lease避免首个session销毁时提前关闭共享日志器，测试也覆盖FIFO、批处理、写失败、同步失败和并发shutdown。这些实现说明它不是纯占位符。

但它仍不是可支撑Editor、Runtime DLL、headless server、插件和崩溃取证的进程级日志基础设施。`LOG_CONTROLLER`只是每个静态/动态链接映像各自拥有的`OnceLock`，App与Runtime DLL会建立互不相知的sink；`LogModule`只是descriptor，实际初始化和关闭散落在二进制入口及FFI session路径。第一个初始化者永久决定配置，后续请求只被静默忽略；多session只共享计数，没有session、project、world、frame或generation身份。

记录只有`level/scope/message`，时间戳由worker写出时才生成，缺少事件发生时间、单调序列、线程、span、callsite、结构化字段和隐私策略。队列只限制记录数而不限制字节数；Warn/Error在满队列上执行无时限`send`，会把渲染、任务或UI线程阻塞在慢磁盘/慢stderr之后，却仍只保证消息进入RAM，不保证落盘。单worker串行写console和file，任一输出的延迟、错误或panic都能拖累整个路由；flush控制命令与数据争抢同一满队列，崩溃hook又在写入panic内容之前flush，因此并不构成崩溃耐久性。

文件侧没有rotation、retention、总配额、压缩、跨进程独占、同秒碰撞消解或日志manifest。路径channel清洗有碰撞和Windows保留名问题，启动失败说明还会经过当前level filter而消失。Editor另有`EditorLogService`和rolling file，`tracing`又只有可选Tracy subscriber，plugin native callback多数发行描述为`None`；同一产品因此存在至少三套不可关联的日志事实。

本报告新增 **0项P0、52项P1、14项P2和36个资格门**。Runtime03继续拥有canonical diagnostics store与profiling/config父合同；Runtime07及其开放failure继续拥有脚本/插件执行与同步sink阻断；Editor11拥有Editor journal/output console；App01拥有进程启动/退出；Interface01与Plugins01拥有跨DLL/插件ABI。Runtime44不重复这些P0，只负责将进程日志收敛为单一router、结构化record、字节有界且不阻塞producer的admission、独立sink监督、可验证的flush fence、rotation/retention与crash artifact协议。

## 2. 审查边界、语料与 currentness

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试属性 / ignored / bench | 结论 |
|---|---:|---:|---|
| `zircon_runtime/src/diagnostic_log`完整实现与测试 | 31 / 3,885 / 126,766 | 39 / 1 / 0 | E3逐文件检查filter、record、queue、worker、metrics、file、panic、flush、shutdown及性能harness |
| Runtime/App/plugin真实集成面 | 18 / 7,023 / 263,499 | 静态反查 | E3核对module descriptor、动态session lease、二进制入口、退出码、Tracy与native host log |
| Editor独立logging owner | 13 / 1,889 / 59,980 | 静态反查 | E2核对journal、rolling file、output console与process log重复authority |
| 父计划、failure与跨owner报告 | 6 / 2,257 / 272,482 | 状态核对 | E2确认唯一owner、开放阻断和不得重复计数的P0 |
| Unreal、Bevy、Godot、Fyrox参考 | 11 / 3,878 / 130,016 | 静态测试反查 | E2/E3核对redirector/fence/panic device、tracing layer、rotation/composite与最低实现基线 |
| selected combined scope | 79 / 18,932 / 852,743 | 39 / 1 / 0 | 工作树fingerprint `0a5ebd1e47eb9a2ab144e693897ccb4f060e49747dbbc2ee332413fc027830f5` |

指纹按79个selected path去重排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。测试数字仅统计diagnostic_log owner中的Rust标记，不表示本轮已编译或通过。

### 2.2 当前工作树说明

1. 本轮冻结的是2026-08-19当前工作树，不是仓库提交态。
2. `sink.rs`含其他会话在途的“无active sink时shutdown成功”变更，lifecycle测试也有对应覆盖；本报告保留并评估该语义，不回退它。
3. App editor/runtime preview入口含其他会话在途的日志shutdown结果传播变更；本报告将其视为真实consumer，但不据此宣称终端耐久性已闭合。
4. 本轮只新增审查文档及索引，不修改Runtime、App、Editor、Plugin、Interface生产代码、测试或Cargo配置。
5. 未重新执行全workspace编译；既有Editor、Hub、WOC和plugin metadata阻断不属于本专项的动态证据。

### 2.3 检查方法

按`配置来源 -> filter compile -> callsite admission -> record ownership -> bounded queue -> worker batch -> console/file fanout -> flush fence -> shutdown/join -> panic/crash -> file naming/rotation/retention -> metrics/diagnostics -> App/DLL/Editor/plugin consumer`顺序逐段阅读，并反向搜索全部非`dev/`生产调用。每段核对authority、identity、ordering、memory、latency、failure、durability、security、teardown和qualification。

### 2.4 Runtime07开放failure的继承关系

`docs/plans/zircon_runtime/runtime/07/failure-2026-07-19-diagnostic-log-synchronous-sink.md`仍为`open`。现有异步队列、lazy API、batch worker、metrics与54-case性能harness只是静态修复基础；focused执行曾得到0 tests，性能矩阵仍ignored，rotation与shutdown/crash durability未完成。

该failure明确禁止用“caller同步critical fallback”宣称修复，而当前Warn/Error满队列时正是阻塞式`sender.send`，测试还把阻塞行为写成预期。因此Runtime44必须把它作为实施阻断继承，不能关闭、重命名为fixed或以“critical不丢”为理由豁免producer latency。

## 3. 必须保留的工程基础

1. 保留编译后的scope level规则和lazy闭包入口，扩展为统一callsite metadata与结构化field admission。
2. 保留有界队列方向，但边界必须同时覆盖record count、owned bytes、per-owner quota和在途sink bytes。
3. 保留单调FIFO基础语义，为priority、owner公平与flush fence定义清晰的全序或分区序。
4. 保留batch worker，但拆分router admission、format/encode和各sink delivery的监督边界。
5. 保留显式flush/shutdown API，将返回值升级为包含generation、fence、per-sink disposition和durability等级的receipt。
6. 保留process-wide metrics方向，增加per-sink、per-owner、queue age、bytes、retry、rotation与last-error状态。
7. 保留动态session lease方向，session只能持有router lease，不能在Runtime DLL内再创建第二个进程级事实源。
8. 保留本地用户目录fallback方向，路径解析应由App/process policy冻结并传给router。
9. 保留panic hook链式调用previous hook的意图，但panic记录必须先进入专用panic-safe writer，再执行有界flush。
10. 保留真实文件write/sync错误传播测试，扩展为真实filesystem、rotation、disk-full和跨进程冲突矩阵。
11. 保留Editor journal/output console作为产品视图，但它应消费统一record stream而非建立独立日志事实。
12. 保留Bevy-style环境过滤的用户需求，或者实现明确兼容子集并更名，不能继续做不兼容却同名的承诺。

## 4. 当前实现链与断路

```text
App editor/runtime_preview --------------------> static zircon_runtime LOG_CONTROLLER A
        | initialize_process_log("editor")                   |
        | shutdown_process_log()                             +--> one queue
        |                                                     +--> one worker
        +--> load Runtime DLL --> dynamic session lease ----> LOG_CONTROLLER B
                                                              +--> stderr then file

EditorLogService --> Editor store/rolling file/output console     tracing::* --> optional Tracy only
Plugin native API --> optional host_log callback, usually None
```

| 阶段 | 当前事实 | 工程断路 |
|---|---|---|
| authority | 每个链接映像各有`OnceLock<ProcessLogController>` | 同一OS进程可有A/B两个互不知情router |
| initialize | 首次配置固定，返回`Option<PathBuf>` | 无typed disposition、冲突、reconfigure或actual sink truth |
| admission | filter后构造owned record | 无callsite/schema/byte budget，Warn/Error可无限阻塞 |
| queue | 单个count-bounded FIFO | 无owner公平、priority reserve、age/deadline或byte bound |
| worker | 单线程批量格式化并串行fanout | console/file故障域耦合，worker无监督 |
| flush | 控制命令进入同一队列并等待reply | 满队列busy-yield，无显式cursor/fence或durability等级 |
| file | 同秒目录与固定channel文件 | 无rotation/retention/lock/manifest，路径可碰撞 |
| crash | panic hook先flush再调用previous hook | panic payload不在file，abort/SEH/OOM无artifact协议 |
| diagnostics | 周期性逐series转成文本 | 快照可部分丢失，sink健康没有进入canonical store |
| product | App、DLL、Editor、tracing、plugin各走不同面 | 无统一timeline、correlation、导出和支持包 |

## 5. P1：必须完成的工程重构

### 5.1 Authority、生命周期与多实例

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-01 | `LOG_CONTROLLER`按链接映像而非OS进程唯一 | App创建`ProcessLogRouter`并以稳定host service传给Runtime DLL、Editor和插件；DLL只持lease | App01 + Runtime44；单进程单router identity |
| R44-P1-02 | `LogModule`/`LogDiagnosticsModule`只有descriptor，未拥有start/drain/stop | 将module声明与router service依赖、启动阶段、shutdown fence连接，禁止入口散装生命周期 | Runtime01/42 + Runtime44；load report显示真实provider |
| R44-P1-03 | first initialize wins，后续settings被忽略 | 定义`InitializeDisposition::{Created,Attached,Conflict,Reconfigured,Degraded,Failed}`及effective config hash | Runtime44；并发初始化结果可解释 |
| R44-P1-04 | `Option<PathBuf>`混淆disabled、console-only与失败 | 返回typed initialization receipt，包含router generation、sink状态、路径和错误链 | Runtime44；无布尔/Option猜测 |
| R44-P1-05 | dynamic session只有数量，没有session身份 | lease携带session/project/world/build/generation metadata，并写入每条record context | Runtime43 + Runtime44；多session可关联 |
| R44-P1-06 | App和DLL关闭语义不同 | App为最终process owner；session只release lease；final shutdown按policy返回per-sink terminal receipt | App01 + Runtime44；无双关停竞态 |
| R44-P1-07 | feature关闭时catalog仍可声明Log模块 | composition compiler必须以真实provider availability和required capability裁决 | Runtime42；禁用feature时不宣称已装载 |
| R44-P1-08 | early bootstrap和late teardown日志绕过router | 增加有字节上限的early spool与terminal emergency writer，并在router接管时保序合并 | App01 + Runtime44；首条/末条错误均可取证 |

### 5.2 Record schema、时间、过滤与安全

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-09 | record仅有level/scope/message | 定义versioned `DiagnosticRecord`：event code、severity、message template、typed fields、context和schema version | Runtime03/Interface03 + Runtime44；JSON/binary/text同源 |
| R44-P1-10 | timestamp在worker flush时生成 | producer捕获wall time、monotonic tick和global sequence；worker不得重写发生时间 | Runtime22 + Runtime44；阻塞前后顺序可证明 |
| R44-P1-11 | 无process/thread/task/span/frame/session identity | 由context provider低成本附加稳定ID，缺失值显式Unknown而非伪造 | Runtime02/22/43 + Runtime44；跨线程timeline可关联 |
| R44-P1-12 | 每条scope/message都分配和复制 | 引入静态callsite ID、interned target、small-field encoding与批量arena；测量分配上限 | Runtime44；disabled/accepted热路径均有预算 |
| R44-P1-13 | count bound不限制超大message内存 | 配置per-record、per-field、queue-owned和process-total byte cap，返回truncate/reject disposition | Runtime44；恶意大消息不突破RSS门 |
| R44-P1-14 | 只转义LF，CR/ANSI/control/scope `]`可伪造日志 | formatter按sink做完整控制字符策略、ANSI剥离/保留策略、字段边界和UTF-8修复标记 | Runtime44；line injection corpus通过 |
| R44-P1-15 | scope filter是原始字节前缀，`asset`误匹配`assets` | 使用segment-aware target matcher和明确wildcard语法，建立canonical normalization | Runtime44；边界表驱动测试 |
| R44-P1-16 | public settings可绕过parser产生空/重复/无界规则 | 所有来源进入单一validated config compiler，限制rule count/bytes并确定性消歧 | Runtime03 + Runtime44；同配置同hash |
| R44-P1-17 | 声称`RUST_LOG`兼容但只解析极小子集，单条无效可丢整个override | 完整支持承诺语法，或改名并返回逐directive诊断；禁止silent fallback | Runtime44；与声明兼容矩阵一致 |
| R44-P1-18 | 无运行时filter reload、compile-time max和隐私redaction | 提供原子filter generation、静态最大级别、secret/PII字段分类与release默认策略 | Runtime03 + Runtime44；reload无数据竞态 |

### 5.3 Admission、队列、worker与sink监督

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-19 | Warn/Error满队列时无限阻塞producer | 禁止无时限`send`；按callsite policy执行reserved lane、bounded wait、emergency sink或drop receipt | Runtime07 failure + Runtime44；frame/job/UI线程p99有硬门 |
| R44-P1-20 | critical入队被误认为durable | 区分Accepted、Buffered、Written、Synced、Mirrored；只有receipt fence可声明相应等级 | Runtime44；断电模型有明确语义 |
| R44-P1-21 | 单FIFO无owner quota和公平性 | 按owner/session/severity配置容量、token bucket与fair scheduler，保留全局reserve | Runtime44；noisy tenant不能饿死critical owner |
| R44-P1-22 | `.write`丢弃enqueue结果 | 所有API返回轻量`LogAdmission`，并能按策略聚合drop/truncate/degrade原因 | Runtime44；caller和诊断可观测 |
| R44-P1-23 | 仅有per-level process计数 | 增加per-owner/per-sink bytes、records、drops、queue age、high-water、retry和latency histogram | Runtime03 + Runtime44；canonical diagnostics可查询 |
| R44-P1-24 | `max_batch_bytes`仅软限制且首条可超限 | 用真实encoded size、oversize path和checked/saturating arithmetic保证硬边界 | Runtime44；任意输入不突破batch cap |
| R44-P1-25 | worker逐record生成时间戳、逐batch新分配buffer | producer时间戳，worker复用bounded buffer/arena并以基准证明分配和CPU预算 | Runtime44；batch size曲线可复现 |
| R44-P1-26 | console和file在同一worker顺序写 | router与各sink拥有隔离队列/预算/health；必要时共享encoder但不共享阻塞域 | Runtime44；慢stderr不拖慢file |
| R44-P1-27 | sticky output error后持续无策略重试 | 定义sink state machine、退避、reopen、failover、disable和operator notification | Runtime44；故障恢复不热循环 |
| R44-P1-28 | worker panic/输出panic无监督 | catch boundary、mark terminal、supervisor重启策略和不可恢复receipt必须完整 | Runtime02 + Runtime44；注入panic不永久挂死 |
| R44-P1-29 | flush/control与数据争抢同一满队列并busy-yield | 独立control lane或monotonic fence cursor；flush只等待其前序，不靠自旋抢槽 | Runtime44；满载flush延迟有硬上限 |

### 5.4 文件、rotation、耐久性与崩溃路径

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-30 | 日志无限增长且无rotation | 实现size/time/session rotation、原子close/publish和可配置backup count | Runtime44；Godot式真实文件轮转测试 |
| R44-P1-31 | 无总配额、age retention、压缩和GC | 定义per-product/project/channel总预算、年龄策略、后台压缩与可中断GC | Runtime44；长期运行磁盘有界 |
| R44-P1-32 | 同秒启动可append同文件 | 文件身份加入UTC高精度、PID、process generation和随机nonce，并使用exclusive create | Runtime44；并发进程零碰撞 |
| R44-P1-33 | channel sanitizer有碰撞、长度和Windows保留名问题 | canonical safe name + stable hash后缀 + 长度限制 + platform reserved-name校验 | Runtime25 + Runtime44；恶意channel corpus通过 |
| R44-P1-34 | 路径策略硬编码company/product并尝试exe/cwd | App冻结product/project/user/session log root；canonicalize并定义symlink、ACL和portable policy | App01/Runtime25 + Runtime44；路径receipt可审计 |
| R44-P1-35 | 无session header/manifest | 每个artifact写schema、build、commit、platform、process、project、config hash、clock anchor和sink generation | Runtime44；support bundle可自描述 |
| R44-P1-36 | startup candidate诊断经过filter，worker spawn失败只`eprintln` | initialization diagnostics走不可过滤的bootstrap channel并进入receipt/emergency artifact | Runtime44；失败原因不可被Off隐藏 |
| R44-P1-37 | spawn失败会残留空文件/目录，`file_enabled`只反映请求 | transactional open/start/publish，失败回滚临时artifact；effective state来自worker ack | Runtime44；无幽灵文件/假健康 |
| R44-P1-38 | periodic flush只有`Write::flush` | 明确Buffered/OSFlushed/DataSynced/MetadataSynced策略、周期和成本，按sink实现 | Runtime44；故障注入证明耐久等级 |
| R44-P1-39 | panic hook先flush，panic内容未写入日志 | panic-safe preallocated writer先写最小record，再执行有界fence并链式调用previous hook | App01 + Runtime44；panic artifact含payload/backtrace disposition |
| R44-P1-40 | 仅覆盖Rust panic，不覆盖abort/SEH/signal/OOM | 建立平台crash reporter接口、emergency fd/handle、hang watchdog和external collector边界 | App01/Runtime06 + Runtime44；每平台资格矩阵 |

### 5.5 产品集成、诊断桥与一致性

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-41 | App静态logger与Runtime DLL logger分裂 | 通过版本化host service/ABI让DLL提交record和lease，不在DLL创建process sink | Interface01 + Runtime44；同process单sequence |
| R44-P1-42 | Editor另有store/rolling file | Editor journal订阅统一router stream，保留UI retention但不重复落盘authority | Editor11 + Runtime44；同record ID不重复写 |
| R44-P1-43 | `tracing::*`只在可选Tracy路径被消费 | 建立统一tracing subscriber/layer桥，process log与profiler各消费结构化event | Runtime03 + Runtime44；span/event不丢失 |
| R44-P1-44 | plugin `host_log`多数为`None` | host API暴露稳定、有预算、带plugin identity的logging callback及capability disposition | Plugins01/Interface04 + Runtime44；首方插件均闭环 |
| R44-P1-45 | diagnostic store每秒逐series写文本，部分drop破坏快照 | 发布带snapshot generation/completeness的批记录或artifact引用，禁止逐series无事务喷发 | Runtime03 + Runtime44；541-series快照原子可辨 |
| R44-P1-46 | schedule大delta重复减法且折叠missed period | 用常数时间计算miss count，记录skipped/coalesced periods和采样时间 | Runtime22/03 + Runtime44；大stall不循环退化 |
| R44-P1-47 | metric文本缺frame、tag、有效性，unit直接拼接，允许NaN/Inf | 使用typed metric sample/schema并定义non-finite策略；文本只是sink formatter | Runtime03 + Runtime44；roundtrip无歧义 |
| R44-P1-48 | sink健康未进入canonical diagnostics、Editor或ABI | 发布router/sink generation、backlog、age、drop、last error、durability与rotation状态 | Runtime03/Interface03/Editor25 + Runtime44；产品可诊断 |

### 5.6 测试与资格证据

| ID | 差距 | 必须重构的内容 | 唯一owner / 验收 |
|---|---|---|---|
| R44-P1-49 | 54-case性能矩阵ignored且focused曾为0 tests | 将正确target/package/feature命令固定进validator，性能套件成为非默认但必跑资格作业 | Runtime44；receipt记录实际case数 |
| R44-P1-50 | 性能只测instrumented output和RSS粗上限 | 增加producer p50/p95/p99/max、alloc bytes、CPU、queue age与真实NVMe/慢盘/console矩阵 | Runtime44；预算按profile/平台冻结 |
| R44-P1-51 | 测试把Warn/Error阻塞写成正确行为 | 改为验证bounded admission与明确degrade receipt，加入frame/job/UI线程不阻塞门 | Runtime07 failure + Runtime44；旧断言被硬切删除 |
| R44-P1-52 | 缺少rotation、真实FS、worker panic、dual-image、crash测试 | 建立unit/property/fault/process/integration/crash五层套件和可归档artifact | Runtime44；36门均有证据owner |

## 6. P2：重要但不阻断首个工程闭环

| ID | 改进项 | 验收方向 |
|---|---|---|
| R44-P2-01 | 为常见callsite生成compile-time event code和field schema | schema collision与升级测试 |
| R44-P2-02 | 支持binary chunk sink和离线索引，避免所有生产日志先格式化文本 | 与text/JSON同record ID roundtrip |
| R44-P2-03 | 提供按scope/owner/session的动态sampling与重复消息折叠 | 保留first/last/count且不隐藏critical |
| R44-P2-04 | 增加remote telemetry sink，但默认关闭且受隐私/网络策略控制 | offline、consent、retry和spool预算测试 |
| R44-P2-05 | 支持内存ring buffer供崩溃前N秒回溯 | 固定字节、无堆panic read路径 |
| R44-P2-06 | 为日志artifact构建轻量索引和seek table | 大文件按时间/sequence快速定位 |
| R44-P2-07 | 跨进程Editor/child runtime用IPC汇聚而非文件尾读 | 断线重连、背压和身份认证测试 |
| R44-P2-08 | release构建支持可审计的field-level redaction token | secret corpus零明文泄漏 |
| R44-P2-09 | 增加clock calibration事件以关联CPU/GPU/remote peer时间线 | drift与clock reset测试 |
| R44-P2-10 | 允许sink按capability热挂载/卸载 | fence后卸载且不丢owner状态 |
| R44-P2-11 | support bundle导出日志、manifest、metrics和crash artifact | 一键导出仍遵守大小与隐私预算 |
| R44-P2-12 | 为Editor output console提供server-side query/index contract | 过滤不需要全量载入内存 |
| R44-P2-13 | 对高重复字符串和字段值做有界intern/compression | 命中率、CPU和内存退化可测 |
| R44-P2-14 | 增加长期soak与磁盘磨损预算 | 24h/7d artifact规模和写放大受控 |

## 7. 参考引擎证据与适用性

### 7.1 Unreal Engine：进程redirector、线程与panic边界

`FOutputDeviceRedirector`拥有多output device、backlog、primary/dedicated log thread、flush选项和显式teardown。它的异步flush fence保证调用前的记录已broadcast，而不是只保证控制消息挤进同一普通队列；panic模式只访问声明为panic-safe的device。`FOutputDeviceFile`进一步处理异步writer、已有文件备份与文件名碰撞。

Zircon应吸收的是多sink redirector、线程能力声明、fence和panic-safe device边界，不应照搬Unreal的全局宏体系或历史兼容复杂度。Runtime44的目标仍是typed Rust service与稳定host ABI。

### 7.2 Bevy：统一tracing生态与layer组合

`bevy_log::LogPlugin`以一个tracing subscriber组合EnvFilter、format、custom layer、Tracy/Chrome和平台层，并用LogTracer桥接传统log。全局subscriber重复安装被显式处理。Zircon当前custom process log与`tracing::*`分裂，且所谓Bevy-style filter并不兼容实际EnvFilter语义。

应吸收统一structured event入口和可组合layer，而非把Bevy plugin本身当成Zircon多DLL进程owner。多链接映像、稳定ABI和crash artifact仍需Zircon独立设计。

### 7.3 Godot：Composite与真实rotation测试

Godot的`Logger`、`StdLogger`、`RotatedFileLogger`和`CompositeLogger`把输出职责拆开，轮转会限制backup数量并删除最旧文件，ANSI处理和flush错误也有显式路径；测试会创建真实文件并验证rotation数量和删除结果。

Zircon应至少达到其真实filesystem证据与composite fanout基线，再增加字节预算、结构化schema、per-sink监督和多session identity。

### 7.4 Fyrox：可借鉴的最小监听面，不是目标上限

Fyrox以全局mutex logger、listener、kind/content/relative time和write-once去重提供简单可用面。listener和重复消息治理值得参考，但同步全局file flush正是Runtime07已指出的性能下限，不能作为Zircon工程级实现终点。

### 7.5 Unity Graphics的适用性边界

本地`dev/Graphics`是渲染管线与Editor package语料，不拥有完整Player/Editor进程日志基础设施。它可为shader/render诊断字段提供消费者需求，却不能证明process router、rotation或crash durability；Runtime44不为了满足“所有参考引擎都出现”而引用无关代码。

## 8. 目标架构与owner边界

```text
zircon_app ProcessHost
  owns ProcessLogRouter + ProcessLogPolicy + CrashArtifactCoordinator
             |
             +-- EarlySpool / EmergencyWriter
             +-- RecordAdmission
             |     +-- schema/filter/redaction/context
             |     +-- byte/count/owner budgets
             |     +-- global sequence + disposition
             +-- RouterScheduler
             |     +-- ConsoleSink worker + health
             |     +-- RotatingFileSink worker + health
             |     +-- RingBufferSink
             |     +-- optional TelemetrySink
             +-- Fence/DurabilityCoordinator
             +-- Canonical diagnostics publisher
             |
             +-- App binaries / Runtime DLL leases / Editor journal / plugins / tracing layers
```

| Owner | 唯一职责 | 禁止继续拥有 |
|---|---|---|
| `zircon_app` | 进程policy、router实例、路径、crash coordinator、最终shutdown | 自己格式化第二份runtime日志 |
| `zircon_runtime::diagnostic_log` | record/filter/admission/router/sink/fence实现 | 进程路径猜测、产品退出码、第二个全局实例 |
| Runtime03 diagnostics | canonical metric/schema/query | 文本日志文件生命周期 |
| Runtime DLL / Runtime43 | session context与router lease | 初始化/关闭process sink |
| `zircon_editor` | journal、查询、UI retention、导出交互 | 独立rolling file事实源 |
| plugin ABI | 有预算的record submission | 任意裸callback生命周期或无身份stdout |
| platform/crash layer | panic/SEH/signal/OOM捕获和emergency handle | 常规filter、UI或业务日志策略 |

## 9. Hard Cutover 约束

1. 不保留“旧process logger + 新router”双写兼容期；先提供adapter，再一次性切换所有生产caller。
2. 不保留Warn/Error无限阻塞语义；旧测试必须删除并替换为bounded disposition测试。
3. 不继续公开`Option<PathBuf>`作为初始化真相；调用者必须消费typed receipt。
4. 不允许Runtime DLL、Editor或plugin创建自己的process file sink。
5. 不把`eprintln!`作为常规fallback；只允许经过审计的bootstrap/emergency writer使用。
6. 不将enqueue success命名为flush/durable；API和指标必须使用准确等级。
7. 不以shim继续支持非segment filter；配置迁移失败必须有逐项诊断。
8. 不把Editor rolling file静默保留为第二份authority；迁移后只保留统一artifact和UI retention cache。
9. 不在crash path分配无界内存、等待无界锁或调用未声明panic-safe的sink。
10. 不关闭Runtime07 failure，直到性能矩阵、rotation、shutdown和crash gates均有可复现artifact。

## 10. TDD实施里程碑

### M0：合同与失败语义冻结

- 先写`DiagnosticRecordV1`、`LogAdmission`、`InitializeReceipt`、`FlushReceipt`、`SinkHealth`的schema/property tests。
- 冻结severity、sequence、time、context、field type、truncate和redaction语义。
- 将Runtime07 failure映射到本报告gate，不改其状态。

### M1：单进程authority与host ABI

- 先写同一App加载Runtime DLL并创建多个session的单router/单sequence失败测试。
- App创建router，通过版本化host service传入DLL和plugin。
- 删除DLL内部process sink初始化/最终关闭authority。

### M2：结构化record与filter compiler

- 先写segment matcher、invalid directive、rule budget、callsite ID和injection corpus。
- 合并custom API与tracing layer，支持原子filter generation。
- 记录producer timestamp、monotonic sequence和context。

### M3：字节有界admission与公平调度

- 先写超大record、noisy owner、queue full、frame/job/UI producer latency测试。
- 实现count+bytes+owner quota、reserved critical lane和明确degrade receipt。
- 删除所有无时限producer send。

### M4：sink隔离、监督与fence

- 先写慢stderr、file error、sink panic、reopen和满载flush测试。
- 每个sink独立worker/queue/health，router维护sequence cursor。
- flush receipt报告每sinkwritten/synced/failure位置。

### M5：RotatingFileSink与artifact manifest

- 先写真实目录、exclusive create、rotation、retention、disk-full、permission和crash residue测试。
- 实现安全文件身份、manifest、size/time rotation、quota GC和耐久策略。
- transactionally publish effective file sink state。

### M6：crash、Editor与diagnostics集成

- 先写panic payload、bounded timeout、previous hook、Editor journal single-record和sink health发布测试。
- 引入panic-safe emergency writer与platform crash coordinator。
- Editor、diagnostics store和plugin host callback切到统一stream。

### M7：资格与删除旧路径

- 激活54-case及新增performance/fault/process矩阵，保存命令、case count、平台和artifact hash。
- 删除旧global initializer、独立Editor file writer、阻塞critical fallback和伪Bevy filter路径。
- 复核所有87个已知custom log call token及21个`tracing::*`生产调用，证明无旁路。

## 11. 资格门

| Gate | 通过条件 | 当前 |
|---|---|---|
| R44-G01 | 单一App进程加载Runtime DLL后仅有一个router generation | 未通过 |
| R44-G02 | 多session记录携带稳定session/project/world identity | 未通过 |
| R44-G03 | 初始化冲突返回typed disposition与effective config | 未通过 |
| R44-G04 | Log模块availability与实际provider一致 | 未通过 |
| R44-G05 | early bootstrap与terminal teardown错误进入artifact | 未通过 |
| R44-G06 | record schema可版本化且text/JSON/binary同源 | 未通过 |
| R44-G07 | producer发生时间、monotonic sequence和线程上下文完整 | 未通过 |
| R44-G08 | disabled log热路径不格式化且分配预算通过 | 部分基础，未资格化 |
| R44-G09 | accepted record的单条/字段/队列总字节硬有界 | 未通过 |
| R44-G10 | CR/ANSI/control/UTF-8/scope注入语料通过 | 未通过 |
| R44-G11 | segment filter边界和声明语法兼容矩阵通过 | 未通过 |
| R44-G12 | filter reload原子且release redaction默认安全 | 未通过 |
| R44-G13 | Warn/Error在满队列时无无界producer阻塞 | 未通过 |
| R44-G14 | admission disposition区分accepted/drop/truncate/degrade | 未通过 |
| R44-G15 | noisy owner不能耗尽其他owner和critical reserve | 未通过 |
| R44-G16 | batch实际encoded bytes不超过硬上限 | 未通过 |
| R44-G17 | console慢/失败不拖慢file sink | 未通过 |
| R44-G18 | file慢/失败不拖慢console和producer | 未通过 |
| R44-G19 | sink panic可检测、可终止或按策略恢复 | 未通过 |
| R44-G20 | 满载flush fence延迟有上限且覆盖全部前序record | 未通过 |
| R44-G21 | receipt逐sink报告Written/Synced位置 | 未通过 |
| R44-G22 | rotation按size/time/session正确关闭和发布 | 未通过 |
| R44-G23 | retention/总配额/GC在长期运行下有界 | 未通过 |
| R44-G24 | 并发进程/同秒/channel碰撞不会共享文件 | 未通过 |
| R44-G25 | Windows保留名、长路径、symlink和ACL矩阵通过 | 未通过 |
| R44-G26 | artifact manifest包含build/process/project/config/clock身份 | 未通过 |
| R44-G27 | 初始化失败不可被filter隐藏且无幽灵artifact | 未通过 |
| R44-G28 | flush、data sync、metadata sync等级与故障模型一致 | 未通过 |
| R44-G29 | Rust panic artifact含payload且hook链/timeout安全 | 未通过 |
| R44-G30 | Windows/Linux/macOS crash/abort/OOM策略有平台证据 | 未通过 |
| R44-G31 | Editor journal只消费统一record，不重复落盘 | 未通过 |
| R44-G32 | tracing与custom process log共享event identity | 未通过 |
| R44-G33 | plugin host log具identity、预算和capability truth | 未通过 |
| R44-G34 | diagnostics snapshot批次具generation和completeness | 未通过 |
| R44-G35 | 54-case及新增性能矩阵实际运行且case count非零 | 未通过 |
| R44-G36 | Runtime07 failure的rotation/shutdown/crash/perf条件全部满足 | 未通过 |

## 12. 测试与证据矩阵

| 层级 | 必需测试 | 关键断言 | Artifact |
|---|---|---|---|
| unit | filter、record、size、escape、sequence、receipt | 无歧义、硬边界、确定性 | test JSON + seed |
| property | 任意UTF-8/bytes、规则组合、oversize、counter边界 | 不panic、不注入、不越界 | failing corpus |
| concurrency | producer flood、shutdown、flush、sink panic、reconfigure | 无死锁、无无限等待、fence正确 | timeline + loom/model结果 |
| filesystem | rotation、retention、collision、permission、disk full、sync | 文件数/大小/manifest/恢复一致 | 临时目录清单与hash |
| process | App + Runtime DLL + 多session + plugin + Editor | 单router/单sequence/正确lease | process receipt |
| crash | panic、abort、SEH/signal、hang、OOM策略 | 有界写入、payload、previous handler | crash bundle |
| performance | disabled/accepted、1-N producer、slow sinks、真实磁盘 | p50/p95/p99/max、alloc、CPU、RSS | benchmark JSON |
| soak | 24h/7d、rotation/GC、filter reload、sink flap | 磁盘/RSS/handle/thread有界 | trend artifact |

### 12.1 现有测试应保留

1. FIFO和batch顺序测试，迁移为sequence/fence断言。
2. lazy closure不执行测试，扩展到结构化field builder。
3. write/sync failure注入测试，拆成per-sink health和receipt断言。
4. concurrent shutdown与dynamic session lease测试，扩成App+DLL真实process测试。
5. performance case生成器和54-case矩阵，修正target并纳入资格作业。

### 12.2 现有测试必须纠正

1. Warn/Error queue-full阻塞测试当前把缺陷固化为合同，必须先写bounded replacement再删除旧断言。
2. source-string lazy guard只能防少数已知producer，不等于callsite分配资格；改为allocator/benchmark证据。
3. instrumented output不能证明真实filesystem、stderr、rotation或sync成本。
4. 128 MiB RSS最大值过粗，不能证明单record、queue-owned bytes或多owner公平。
5. ignored测试和0-case运行不得计为通过。

## 13. 逐文件检查台账

### 13.1 根、配置、级别与平台

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `diagnostic_log/mod.rs` | 全局controller、公开write/lazy/flush/shutdown/panic入口；caller看不到admission | service handle、typed receipt、统一structured API |
| `diagnostic_log/settings.rs` | 默认级别、queue/batch/flush/file设置和环境解析；配置来源未统一验证 | versioned policy compiler、byte/owner/rotation/crash设置 |
| `diagnostic_log/level.rs` | level枚举、排序、显示和规则入口 | schema severity与兼容解析 |
| `diagnostic_log/level/compiled.rs` | 预编译raw prefix规则，边界和规模策略不足 | segment matcher、rule budget、generation |
| `diagnostic_log/platform.rs` | 候选路径、channel清洗和panic hook | App path policy、安全名称、emergency writer |
| `diagnostic_log/timestamp.rs` | 本地秒级目录/行时间格式 | UTC高精度artifact identity；producer clock anchor |

### 13.2 Diagnostic store bridge

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `diagnostic_log/diagnostics.rs` | 定期snapshot并逐series写文本，schedule折叠周期 | typed batch、generation/completeness、O(1) miss计算 |
| `diagnostic_log/diagnostics/tests/mod.rs` | 只做三个测试owner的声明 | 保持薄入口并按语义分组扩展 |
| `diagnostic_log/diagnostics/tests/format_schedule.rs` | 覆盖文本格式与基础周期，未覆盖partial drop/大delta/schema | transaction、miss count、non-finite与backpressure测试 |
| `diagnostic_log/diagnostics/tests/lazy_callsite_guards.rs` | 以source string约束四类producer使用lazy API | allocator/callsite基准替代脆弱字符串守卫 |
| `diagnostic_log/diagnostics/tests/ownership.rs` | 以目录扫描守卫snapshot bridge唯一owner | 保留owner守卫并验证typed batch service wiring |

### 13.3 Sink、worker与metrics

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `diagnostic_log/sink.rs` | `OnceLock`controller、count queue、blocking critical send、flush/shutdown/session lease | process router、bounded admission、fence、typed lifecycle |
| `diagnostic_log/sink/worker.rs` | 单worker批量格式化并顺序写console/file；timestamp晚生成 | sink隔离、supervision、buffer reuse、producer timestamp |
| `diagnostic_log/sink/metrics.rs` | 原子计数按level聚合，output状态混合且无generation | per-owner/per-sink bytes/age/latency/health snapshot |

### 13.4 Sink测试根与共享fixture

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `diagnostic_log/sink/tests/backpressure.rs` | best-effort drop；critical阻塞被当作预期 | bounded critical lane、deadline与degrade receipt |
| `diagnostic_log/sink/tests/batching.rs` | count/soft byte batching | encoded hard byte cap、oversize、buffer reuse |
| `diagnostic_log/sink/tests/durability.rs` | write/flush/sync错误基础 | per-sink cursor、rotation、disk-full、crash durability |
| `diagnostic_log/sink/tests/fixtures.rs` | instrumented output、故障注入与等待fixture | 保留确定性注入，增加真实FS/process/crash fixture |
| `diagnostic_log/sink/tests/lifecycle.rs` | initialize/filter/lazy/metrics/panic/flush/shutdown/session lease集中于单文件 | 拆分unit/concurrency/process/crash并增加typed disposition |
| `diagnostic_log/sink/tests/mod.rs` | 聚合六个sink测试owner并定义测试锁 | 保留隔离入口，减少全局锁掩盖的真实并发问题 |

### 13.5 性能语料

| 文件 | 已检查事实 | 重构落点 |
|---|---|---|
| `diagnostic_log/sink/tests/performance/case.rs` | case参数与命名 | 增加sink、payload、owner、severity、disk维度 |
| `diagnostic_log/sink/tests/performance/configuration.rs` | case配置和环境读取 | profile/platform预算、版本hash和typed parse receipt |
| `diagnostic_log/sink/tests/performance/critical.rs` | critical level/backpressure case | bounded wait与producer max latency门 |
| `diagnostic_log/sink/tests/performance/mod.rs` | 54-case ignored矩阵入口 | qualification job强制实际case count |
| `diagnostic_log/sink/tests/performance/output.rs` | instrumented output与写入成本控制 | 增加真实console/file/device矩阵 |
| `diagnostic_log/sink/tests/performance/pacing.rs` | producer pacing | latency histogram、burst与steady-state模式 |
| `diagnostic_log/sink/tests/performance/report.rs` | 结果模型与报告 | p50/p95/p99/max和per-sink/owner字段 |
| `diagnostic_log/sink/tests/performance/resources.rs` | worker/thread与资源观测 | allocations、handles、threads、IO bytes、CPU |
| `diagnostic_log/sink/tests/performance/rss.rs` | process RSS probe | queue-owned bytes与platform baseline |
| `diagnostic_log/sink/tests/performance/rss/windows.rs` | Windows工作集采样 | commit/private bytes与错误receipt |
| `diagnostic_log/sink/tests/performance/validation.rs` | 阈值和case验证 | 分profile/platform基线与非零case证明 |

### 13.6 直接集成文件

| 文件组 | 已检查事实 | 重构落点 |
|---|---|---|
| `module/log.rs`、`module/log_diagnostics.rs`、`module/builtin/core_modules.rs` | descriptor与实际service lifecycle脱节 | composition required capability和service wiring |
| `dynamic_api/construction.rs`、`ffi.rs`、`state.rs`、profile/lifecycle测试 | DLL内独立controller与session lease | host-owned router ABI和session context |
| `profiling.rs` | tracing subscriber只在Tracy特性路径 | unified subscriber/layer graph |
| App editor/runtime preview与entry runner | 入口初始化、panic、shutdown和`eprintln`旁路 | ProcessHost唯一owner、early/terminal receipt |
| `zircon_editor/src/logging` | 独立store/rolling file/output console | 订阅统一record stream，保留UI模型 |
| `zircon_plugin_sdk/src/native.rs`及dist consumer | optional `host_log`能力常为空 | versioned、budgeted、identified host logging ABI |

## 14. 父报告与failure回写要求

| Owner | 本报告新增的精确回写 | 不得改写为 |
|---|---|---|
| Runtime03 | diagnostics sink健康、typed metric batch、tracing bridge依赖Runtime44 router | Runtime03自己拥有file rotation |
| Runtime07 failure | 保持open，挂接G13/G20/G22/G28/G29/G35/G36 | “异步队列已存在，所以fixed” |
| Runtime42 | Log模块provider availability和service lifecycle进入composition receipt | descriptor存在即capability可用 |
| Runtime43 | dynamic session只持host router lease并附加session context | DLL拥有第二个process logger |
| App01 | App拥有router/path/crash/final shutdown；退出失败消费typed receipt | 任一DLL关闭全局sink |
| Editor11/25 | journal/output console和性能视图消费统一record/health | Editor另写一份rolling authority |
| Interface01/04 | host logging service ABI与owner/generation/foreign lifetime | 裸Rust trait或无版本callback |
| Plugins01 | plugin identity、配额、capability truth | `host_log: None`仍宣称logging完成 |

## 15. 审查状态与输出记录

- 审查状态：`review_complete`。
- 实施状态：`pending`。
- 新增finding：0 P0 / 52 P1 / 14 P2。
- 新增资格门：36，当前0项通过，1项仅有部分基础但未资格化。
- 既有开放failure：Runtime07 diagnostic-log synchronous sink，保持`open`。
- 本轮没有修改生产代码、测试、Cargo、feature、资产或发行配置。
- 本轮没有运行Cargo测试；静态报告不能替代Runtime07性能、rotation、shutdown和crash动态证据。
- 实施顺序受MVP L0-L5依赖约束：先收敛App/Runtime authority与合同，再切caller和产品视图，最后执行性能与崩溃资格。
- 实施前必须重取源码fingerprint，尤其复核当前在途的sink shutdown和App退出语义改动。
