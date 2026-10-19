# S68 当前运行证明消费者审查（2026-10-18，待原生验收）

本轮按最低 MVP00 门槛继续审查 S56 `CR-APP-ENTRY-0017` 的当前消费者。三路 Luna 并行阅读，Sol 独立裁决参数合同；当前 `main` 为 `a80b4c1e23d7d4910782112c2cdfc8025851262b`。没有修改生产源码、执行测试或取得新的产品验收。

当前执行入口已迁至 [MVP 权威索引](E:/Git/ZirconEngine/docs/testing-and-validation/mvp-acceptance-index.md#current-execution)和[验证政策](E:/Git/ZirconEngine/docs/testing-and-validation/milestone-validation-policy.md)。`00` 仍为 `in_progress`，旧协调器命令仅作历史记录；产品与清理门槛继续有效。

三路正文为 2211 / 2006 / 967 行，共 5184；Root 重读 351，Sol 独立读 720，合计 6255 reader-body 行。Sol 的 675 行与主审重复，45 行是主审范围外的 headless 上下文；guidance 和 locator 分列。已全文审查 9 个与 S56 指纹不同的当前版本，共 3381 行。其余 15 个候选哈希仍匹配 S56；本轮未重写历史 572-pin 记录，也未声称全局净增或模块完成。范围与逐文件四字段见[实际审查报告](E:/cargo-targets/zircon-local/reports/engine-audit-20261009-01a12097/s68-runtime-proof-current-consumers-review.json)。

| 环节 | 当前证据 | 验收状态 |
| --- | --- | --- |
| CLI → host | [参数解析](E:/Git/ZirconEngine/zircon_app/src/entry/entry_runner/runtime_session_args.rs)保留 GeneralProject 默认与显式 F2；runner 把同一 typed policy 交给产品 host | 静态；原生待验 |
| 生产完成与失败 | [完成路径](E:/Git/ZirconEngine/zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs)在 snapshot / 校验成功后才提交状态与计数，错误登记后退出 | 静态；实际 present / capture 待验 |
| 输入 | input probe 比较本批 before / after，逐类要求新增消费 | 静态；当前产品输入待验 |
| workflow / 工具 | workflow 显式 `f2-lit-mesh`；Stage 默认 `none` 且仅 Runtime 传证明选项，Acceptance 拒绝错误 policy 并保留全部阈值 | 静态；外层退出码、执行与上传待验 |

参数合同候选输入为：

```text
--project=fixture --play-report-pipe --runtime-product-proof=f2-lit-mesh
```

当前解析会把最后一项作为 outlet，得到 GeneralProject 且无 remaining。出口值现只要求非空，已读 help / 注释 / 测试没有规定 option-like 值为保留词，因此该输入也能表示逻辑出口名称。Sol 独立裁决为**未裁决参数合同候选**，没有确认 S56 回归或产品缺陷；当前 Stage F2 argv 未带 pipe。本轮保留现有行为，27 项问题队列逐条不变。

如果以后收紧文法，应先明确五个空格形式必需值入口的统一边界，并保留 `-outlet` 及等号形式的字面名称。回归需覆盖普通值、相邻选项、equals 消歧、正常 pipe 加独立 F2、串联 diagnostics / runtime 和 headless 拒绝合同。该方案尚未实施或执行。

原 S50 request / operation 仍为 `engine-audit-20261017-mvp00-runtime-default-s50-r1`，原 runtime 为 `start-1792141433756007100`；没有重复查询、重提或换身份。当前 installed 指纹与 S66 一致，任务回复文件没有更新。PID 20236 可观察为 python，但启动时间读取仍失败，进程名称不构成 owner / runtime 身份证明。正式原请求终态与 S67 fixture 生命周期迁移仍开放。

[Jenkins 本轮交接](E:/Git/ZirconEngine/.jenkins/records/engine-audit-20261009-01a12097-s68-current-proof-consumers-followup.md)已保存到用户指定目录；未获已读或 ACK。[S67 fixture 合同](E:/Git/ZirconEngine/docs/plans/performance/2026-10-18-mvp-fixture-migration-contract.md)继续等待真实 runner / 登记 / 物理根 / 清理接口和实现回归。所有 native、产品、性能、模块及里程碑验收仍为 false。

本轮 Root 与四位审查代理均无显式源码写入、测试、构建、产物清理或性能采样。必要证据、foreign / 未知归属产物及 S67 两条未恢复的只读进程终态继续保留；实际释放 0 B，cleanup 未关闭。预检 E 盘可用约 43.907 GiB，空间增长不归因于本任务。
