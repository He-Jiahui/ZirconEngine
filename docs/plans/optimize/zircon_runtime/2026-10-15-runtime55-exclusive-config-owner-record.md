# Runtime55：同一路径独占配置写入者

日期：2026-10-15；任务 optimize-20261013-01a13451。exact16 源码冻结、静态检查和独立源码复核完成；原生、性能及产品验收仍待合批 Jenkins 实际结果。

## 问题与修复

同一路径的第二个独立 manager 过去能发布新 epoch，使仍存活的第一个 manager 随后提交失效。现在先按物理路径取得独占 owner lease；同 Core 和不同 Core 都在恢复、解析及 worker 创建前报告 typed owner conflict。manager 的 clone 共享已有 worker，不会重新注册写入者。

registry 的全局锁只覆盖查找、预留及回收，释放后才进入路径 gate。准备失败或 panic 不发布 epoch，候选取消并释放未成功的预留；强 owner 引用的最终 Drop 保持在 registry 锁外。CoreWeak 失效和取消提交不会提前释放租约，真实 worker join 和最后 fence 释放决定写入者结束。

路径准入先把输入一次转换成绝对路径，再用同一输入进行最终 symlink 检查和物理身份解析；恢复、加载、worker、staging、commit 使用捕获的 operation_path。默认相对路径语义保留，转换或身份解析失败会返回错误。Windows 盘符相对路径 `C:foo` 不能只靠 `current_dir().join` 保证绝对化；实现采用 Rust 1.79 起提供的 `std::path::absolute`。这是对静态复核发现的 CWD 窗口的修正，原生行为仍需验证。[Rust absolute](https://doc.rust-lang.org/std/path/fn.absolute.html)、[Windows 路径转换](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew)。默认用户路径选择行为保留，SDK 新增可选的显式 host 文件参数，相关 animation、physics、Runtime62 与 shader 夹具使用独占物理 E:/cargo-targets/zircon-local 路径，在全部 holder 关闭后清理。

此前 Runtime03 记录中的成功替换语义和词法路径键，在本轮增量中分别由活跃 owner 拒绝与 ProjectPaths 物理路径身份替换；旧封包保留作历史证据，现行源码合同以本轮封包为准。

这项修复处理进程内同时存活写入者。跨进程、硬链接、外部命名空间竞争，以及 Runtime55 其余 typed Foundation、配置分层、产品消费与 lifecycle 门禁继续 OPEN。

## 回归与性能

新增六个 ordinary 的精确 FQN 由冻结 source-binding 列出，必须在真实 Windows 运行中逐项通过。原有 34 ordinary、3 ignored 的名称和标记保留。SDK 的3例、animation physics_tick 的40例、physics 默认与 backend-jolt、shader 的26 ordinary，以及三个 Runtime62 各1 ordinary/1 ignored 纳入对应合批配置。shader 的11项 ignored 库存保留，沿用各自的手工验证与能力条件，不统一强制执行。词法库存和 cfg 推导不等于原生通过数。

Core commit/cancel 与三个 Runtime62 profile 测量函数体保持；原31样本、深度/规模矩阵、allocator 与计时窗口、Release ignored 参数、CPU/allocation/RSS/延时断言不放宽。Runtime03 原 helper 70%/75%、60原始样本矩阵、资源门和冻结绝对 P95 门槛全部同时验收。2秒关闭等待预算不代表整个 join 的耗时上限。

## 完成与验收

- [x] exact16 按授权实施，保留先前 Runtime03 增量、共享 index 及其他会话修改。
- [x] 冻结 source-binding/增量 patch，16项修复前后原字节可逆；未知来源的 fence API 适配单独归属，不计为本任务自己的变更。
- [x] V1 scoped guard 实际 exit0，0 errors、0 warnings；Luna 完整复核发现 CWD 窗口，V2 修复后再次完成相应静态检查与独立复核。
- [x] 写入本优化记录与 Astra 对应完成列表。
- [ ] Jenkins 多任务同批执行全部 required FQN、相关 feature 配置和原16 Cargo/2 Python/16 producer/legacy11；0 failed，required项0 ignored。
- [ ] 实际源码、配置、双锁、features、工具链与环境执行前后封印，request/attempt/raw 结果可追溯；输入漂移拒绝本次验收。
- [ ] Runtime03、Runtime44 RSS 与原全部性能、资源、故障和产品门槛同时通过。
- [ ] Runtime55 FND-P0-002 的原生与产品证据；其余 Foundation 全量门禁继续 OPEN。

## 证据与异步交接

- [V2 源码绑定](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v2-source-binding.json)：194947 B，SHA256 `485e73f89dbea8c0f5853edba0a2d21f56cb10089f5e7d0212e21955cde59713`。
- [V2 增量补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v2.patch)：1573 B，SHA256 `25a44247c6568bc3af49ac01032ca1f486d0bd654ea4f0c25d94afb5abc719d1`。
- [V2 scoped guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v2-static.json)：7674 B，SHA256 `80b9ea0b3ea7d23a10f3a6c64983501057765b0345b107037698531919135c7c`。
- [V1 源码绑定](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v1-source-binding.json)：200375 B，SHA256 `13846d5ab340183598ad87f1d1220e6313836110bb18c16c2f85bc88008388c1`。
- [V1 增量补丁](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v1.patch)：97428 B，SHA256 `cdf40af677f508d2f2da448450d4a744f6198cfc35805e88b84af941cf9b537a`。
- [V1 scoped guard](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-config-owner-v1-static.json)：7593 B，SHA256 `09fed54bea22938aa0ac6431e346f128176f82d0854a9ed1b366f4e70f8a2271`。
- [未知 fence API 适配见证](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime55-exclusive-owner-unknown-fence-api-adaptation-reconciliation.json)：9478 B，SHA256 `0aa66d5d4424a142935e082ff95c08bb4abf555f570fb0617a9a239342c3f816`。
- [Runtime03 绝对 P95 冻结标准](../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)：10780 B，SHA256 `1075e7ac1195d9b8d5521eb20fda659506aa4759f4b9f5af7736dc1dcd4c77f5`。

V2 writer `1b4e52/session70281 → 581149` 实际 exit0，独立最终回验 `de47a8` 实际 exit0；root whole16 guard `root64/session83267 → root66/e37438` 实际 exit0，0 findings。Luna V1 完整审查发现 CWD 窗口，V2 定向复核确认该问题闭合，15个其余源码文件保持 V1 after。

Luna 的一次额外 V1 patch 直接逆向检查错误地使用了当前 V2 基准，实际返回1且没有修改文件；正确 V2 逆向检查返回0，Sol 在内存中验证 V2→V1 after→原始 before 的完整链。两类结果分别保留，未运行 native/Cargo/benchmark。peer 自有 cells1–9均已结束；历史 core cell13 仍 UNKNOWN。

[Astra 完成列表](../../astra/features/runtime/2026-10-15-runtime55-exclusive-config-owner-completion-list.md)。

原计划：[Runtime55](55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md)；[Runtime03](03-core-runtime-diagnostics-profiling-config-review.md)。

准备 B19 与其他冻结修复合批，通过 .jenkins/records 通知授权 Jenkins 会话。提交后继续后续功能修复，不等待或实时跟踪编译；文件交接和静态通过均不能替代原生性能验收。

本轮无自有 build/cache/native fixture/helper/sourcecopy；实际释放 0 B。必要源码归属、失败和静态证据保留。历史 core cell13 PID/终态 UNKNOWN，未知归属和其他会话活动产物不删除，不宣称全部历史进程关闭。
