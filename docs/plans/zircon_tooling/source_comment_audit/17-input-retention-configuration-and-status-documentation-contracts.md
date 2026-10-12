# 17 — 输入事件保留配置与状态文档契约审查

日期：2026-10-09。全仓目标仍进行中。本阶段完成四个文件的独立主审、中文文档、语言检查和最终哈希覆盖绑定；正式账本发布、验证交接与提交仍待处理。

## 范围与完成状态

按 [04 统一文档规范](04-documentation-contract-and-review.md) 审查定义、公共导出、派生能力、构造入口、管理器调用、缓冲所有权、条件模块及关联测试。原源码共 58 行，新增 21 段、262 行文档注释，最终共 320 行。程序行为、公共 API、原有四行注释及其来源保持不变。

| 精确文件 | 原行数 → 最终行数 | 最终 SHA256 |
| --- | ---: | --- |
| [event_retention/mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/event_retention/mod.rs) | 7 → 22 | `7557f3eccdad8d8f78fe355008bab48b9e3063abf76d323bf8d6eb770c6dfc45` |
| [queue_status.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/event_retention/queue_status.rs) | 8 → 54 | `b30e066eb86b5c41ca62cf67c3b365fae7b445e707b7a23bb83b7de031faf1e5` |
| [recording_config.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/event_retention/recording_config.rs) | 33 → 157 | `68137b730a54fe9577cdbce406a0d4792863bbb1d87409fd57d82f9cf2fe35f2` |
| [recording_status.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/event_retention/recording_status.rs) | 10 → 87 | `44ff02f8e29b4b39ef718e8488d362f47830e4b61639375fc25422881a3bdb9c` |

40 项主审记录与 40 项 Core 记录均绑定到最终文件和对象范围。文档覆盖三个函数、三个数据类型、八个字段、一个常量、两个实现块及四个模块职责。三个函数分别说明参数、返回、使用时机、约束、生命周期和参考；无参数入口也有明确参数章节。21 个 Core 候选均经主审细化，原候选字节、封条和作者来源保留。55 个文档引用定义完成静态作用域与最终行号核对。

## 本阶段明确的契约

- 即时事件缓冲合并相邻同类指针样本。保留数是当前槽位数；合并数在窗口内饱和累计，排空不重置，下次 `begin_frame` 才清零。两字段不能相加作为全程提交总数，队列长度转为 `u32` 也不保证任意规模精确。
- 原始录制与帧窗口独立。容量按记录条目计，允许零，不限制即时队列或事件负载总字节。录制配置的构造只生成值，管理器应用配置时才启用、清理或裁剪。
- 默认记录器关闭或从关闭切换到开启时重置记录、序号和丢弃数。持续开启的配置更新保持当前周期；缩容淘汰最旧记录并累计丢弃，增容保留已有记录。零容量仍推进序号并统计丢弃。
- 组合排空入口在默认管理器的同一个状态锁内先排空再查询，因此返回状态中的保留数为零；本次取得的记录数量来自同时返回的向量。累计丢弃数跨排空保留，多帧不能求和；现有聚合辅助函数取最大观察值。
- 状态是拥有标量的快照，不携带记录借用、实时订阅或自身的同步能力。默认状态的零容量与默认配置携带的 8,192 不同。公开构造、Serde 和零状态不能证明真实记录器或全部输入历史完整。
- 核心输入契约无条件公开，运行时 `crate::input` 受 `native-host` 挂载条件控制。文档链接使用无条件核心定义，条件实现使用普通代码路径；构造器链接明确为函数，避免与同名字段混淆。

## 验证与证据

四个文件均通过 Rustfmt 标准输入解析和精确格式比较。主审逐段阅读全部有效文档及实际 diff；逆向删除本任务的精确注释段后，与四个审查起始对象的原字节完全一致。最终绑定覆盖对象范围、注释段、引用定义和原支持版本。`git diff --check` 通过。

未运行 Cargo、Rustdoc、动态测试、性能测试或 Jenkins 验收。24 个主审支持路径、26 个精确范围有实际读取证据；子审支持版本另外保留，哈希未改贴为新增注释证据。逐项日志放在私有证据包：

- [独立主审证据](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-input-retention-current-source-and-contract-review-20261009-r3/seal.json)：`67ec3aba11165136e88151ecff4164360ccf50e647c63144d888a74932ea4c36`。
- [语言与精确注释差量](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-input-retention-declaration-docs-review-and-language-20261009-r3/seal.json)：`dee400c240b84c869ff6bcbb2ff9bc2de01b8ae1956e0f77e50eda5960b61026`。
- [最终逐项覆盖与引用绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-input-retention-final-coverage-20261009-r1/seal.json)：`2439c3fa158a0a71c94ad65892f29fac8e9f3cc2446bed90f07b148ea13eca3f`。

原定位、读取截断和脚本预检错误均保留失败事实与恢复证据。r2 记录脚本因缩进锚点预检失败，源码、结果目录和配置写入均为零；r3 恢复后才封存主审证据。没有创建完整源码快照，也没有以私有证据代替正式账本发布。

## 问题与原归属

本阶段没有新增 `BUG:`、`TODO:` 或 `SAFETY:` 标签，也未作业务修复或问题关闭。三个私有观察属于已确认的计数、配置和排空契约，已反映在文档中。

关联 recorder 源码文本守卫期待测试条件属性后紧邻模块声明；当前路径属性使该分隔缺失。原测试的代码、注释和归属保持不变，其问题仍交由精确支持文件所属批次处理。默认忽略的计时夹具没有执行或验收结果。既有输入问题标签保持开放。

## 并行审查与待处理门槛

核心分区使用 gpt-6.1-sol max，只读审查并仅写各自私有结果目录：轴与缩放七文件／188 行；公共输入入口及边界测试五文件／226 行；输入模块配置与工厂五文件／157 行。辅助 gpt-6-luna max 核对清单和哈希，不把哈希相同当作对象审查完成。这些路径互不重叠，主审和语言检查保持后续独立阶段。

- 正式账本目录 `docs/_data/source-comment-audit` 已获用户增加授权的指示，但保存配置和当前聊天范围仍缺少该目的地 grant，私有最终覆盖尚未发布。未修改权限配置或宣称探针验证通过。
- `.git/index.lock` 存在且为 0 字节，所有者未确认，保持原样。未暂存、提交、推送或发送企业微信通知。
- 退休本地协调器继续保持退休；不调用旧服务、注册、租约、归属、队列或提交接口。既有 Jenkins 交接仍未发送，未启动服务或新增验收。
- 产品排除清单、各历史注释作者和覆盖来源保持原样；后续业务内容不作为本任务的新注释作者或证据。
- 全仓最终逐文件覆盖、待处理对象清零、问题标签与账本一致性、正式发布和阶段提交仍未完成。

## 下一步

继续独立复核已封存的按钮、设备与震动、输入快照和其它核心候选，接收当前三个分区结果。指针与窗口 r2 恢复汇总已封存，六个原 FIRST 和新精确范围切片仍待主审，不以旧撤回汇总充当有效证据。满足真实写入、索引及验证交接门槛后，再发布账本和按精确归属提交。
