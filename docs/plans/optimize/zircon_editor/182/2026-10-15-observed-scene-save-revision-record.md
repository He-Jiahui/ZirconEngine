# Editor182：已观测 Scene 版本与保存结果

日期：2026-10-15。任务：`optimize-20261013-01a13451`。对应 [Editor182 原计划](E:/Git/ZirconEngine/docs/plans/optimize/zircon_editor/182-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document.md) 的 `ED182-P0-01 / ED182-M0`。

本轮完成源码、字节级增量证明、scoped guard 和独立源码复核。**原生测试、性能及 Editor182 P0 验收仍未完成。** 当前实现提供已观测版本的提交前校验；非协作外部 writer 在校验与 replace 之间的竞态仍然存在，不能把它登记为完整跨进程 CAS。

原问题是本地版本 A 对应的冲突提示已经观察到外部版本 B，提示等待期间外部又发布 C，普通 Save 仍可能覆盖 C。现在普通 Save 提交加载时的版本；确认覆盖消费绑定提示所观察 B、精确文档身份和 activation 的一次性 `OverwriteLease`。B 已变成 C 或 activation 已过期时拒绝本次保存，保留外部内容、dirty、checkpoint 和未发布 Saved 的事实，下一次覆盖需要新决策。

本次改动保留现有 project generation guard、transaction owner lock、journal、recovery 和逐文件原子发布。Resource 持有内容摘要与文件对象身份；Runtime 解析同一个 `SourceSnapshot` 的字节并返回 typed publication receipt；Editor 生命周期持有已加载或已持久化的 baseline。缓存 artifact 不再与稍后重新观察的 source baseline 拼接。

主要源码行为：

- Scene 保存必须携带 expected revision。新 Scene 明确携带 Missing，最终发布始终使用 create-only primitive，不能因为 staging 期间出现目标文件就转为覆盖。
- receipt 的内容和文件身份来自已经同步的 staged 对象，不能用发布后重新读取路径得到的另一对象冒充结果。普通 `expected=None` 写入保留原路径，不增加身份、摘要和 witness 工作。
- Durable receipt 可以推进磁盘 baseline；只有完整 `Persisted` 才发布 `DocumentMessage::Saved` 并执行 `mark_saved_if_unchanged`。
- `PersistedProjectionFailed` 保留 receipt 与已推进的磁盘 baseline，同时保持 dirty、checkpoint 和失败结果；`RecoveryDeferred` 保留旧 baseline，不发布 Saved，也不标记 clean。
- Windows 使用 volume 与完整 128-bit File ID，保留 metadata witness；Unix 使用 dev/ino。身份获取失败返回 typed I/O，未知平台返回 Unsupported。拒绝叶子与已有祖先中的 reparse/non-directory，不回退到 mtime、长度或路径字符串。

冻结范围为 69 个文件：60 个既有文件、9 个原先不存在的新文件；66 个 changed、3 个 unchanged，原始 foreign semantic diff 为 0。原始 before 共 724,710B，after 共 798,252B。root 从磁盘 patch 独立逆向恢复全部 60 份原始 before 与 9 份原始不存在状态，并再次正向重建全部 after；69 项前后 raw 双读一致。原混合换行的未修改片段保留。

证据保留在同一任务目录：

| 证据 | 字节数 | SHA-256 |
|---|---:|---|
| [决策](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1-decision.json) | 6654 | `a6b4e9332bd6f86db35db32d16d725756820869e4e48614fbdb90159d6d4b4b4` |
| [source binding](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1-source-binding.json) | 90352 | `2f93a29bf94d9e0d5c2de9d81ba9ca6cd8781628da0c7eca4c58e269435536d4` |
| [增量 patch](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1.patch) | 160685 | `183c3f243fce285797f0bec1e35c6136aadd6b25be75e53d66ce5bc0c110fc68` |
| [root 静态核验](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1-static.json) | 42826 | `3cb36192cb857af1dcc43d5bd4027ed57e438830a594ed061ae57d2c08c685fc` |

root 静态 writer 的真实终态为 root130 → root134 `a7dd7c` exit 0，Python PID 6944，guard PID 9748 communicate/join exit 0，findings 为空。HEAD 为 `f28ba39a5095f4964c046ecc702778627f80cfd5`；共享 index 的字节、SHA 和 Windows last-write ticks 均未变化。Python 显示的 100ns 舍入差异不计为 index 修改。

独立复核由 `gpt-6-luna max` 完成，结果为 `PASS_static_only`，没有发现可操作的 P1/P2 源码缺陷。复核覆盖 69 项 raw/hash、同字节加载、Missing、staged receipt、普通/覆盖保存、错误映射、dirty/checkpoint 和 20 个新增病例的真实断言。其只读 session 14095、93906、98638 均已 join exit 0；shell PID 未暴露，保留 UNKNOWN。猜测路径不存在的 rg exit 1 与历史 spawn_ready failure 保留，未作为通过依据。

新增病例是源码库存，共 20 个：Resource 身份 3、transaction 7、Runtime 同源加载 2、Editor lease 2、真实 reload/save 4、publisher disposition 2。其中 18 个要求 Windows，2 个为跨平台纯逻辑病例。未执行这些病例，不能把 20 写成 native passing floor。完整 FQN、原有病例和 cfg 在 binding 中。

后续统一 Jenkins 批次须保留既有验证命令、ignored producers、全部样本和资源门槛。Resource 用独立 `zr_resource --lib --features test-support` 配置；Runtime 同源加载归入 `target-editor-host,test-support` lib 配置；Editor 用 `--no-default-features --lib` 配置。只合并兼容配置；由真实 native listing 核对新增与原有名称。所选文件 hash 不代表完整 compiler input/config/lock/features/toolchain/environment seal。

仍需完成：

- Windows metadata-only identity witness 与 `ReplaceFileW`、真实文件/目录 symlink/reparse 的原生验证；不得跳过失败病例后宣称通过。
- 普通 Save、提示后二次写入、delete/recreate、发布前后故障、Conflict 后重试的真实执行证据。
- 非协作 writer 的 check-to-replace、namespace substitution 和 recovery 条件发布闭环；`ED182-P0-01` 继续 OPEN。
- 非 Windows 编译/原生行为、原有性能、RSS、Runtime03 绝对 P95 及产品验收，均继续 OPEN。

没有创建源码副本、helper、native fixture、编译产物或缓存；本切片释放空间 0B。保留冻结 packet、增量 patch、失败证据及 [产物清单](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/artifacts.json)，Jenkins 负责后续原生夹具和构建生命周期。对应 [Astra 完成列表](E:/Git/ZirconEngine/docs/plans/astra/features/editor/2026-10-15-editor182-observed-scene-save-revision-completion-list.md)。
