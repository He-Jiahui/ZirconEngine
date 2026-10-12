# 20 — 手柄模拟轴与帧观察值的文档契约审查

## 阶段状态

状态：本批七个源码文件的 Core 审查、主审独立复核和范围内语言检查已完成，精确最终覆盖证据已封存；正式账本发布、阶段提交及全仓完成仍待处理。继续遵循 [声明文档规范](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/04-documentation-contract-and-review.md)，承接 [按钮接口阶段](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/19-gamepad-button-documentation-contracts.md)。本文件只记录阶段状态和证据入口，逐对象审查、错误及候选决定保存在独立证据包。

本批新增 49 段、616 行中文注释，其中 614 行为模块或声明文档，2 行为私有不变量说明；源码由 188 行变为共 804 行。没有改变程序语句、公共 API 或原有六行注释，也不宣称原注释或现存业务代码的作者身份。

## 审查范围

| 精确文件 | 起点行数 | 最终行数 | 当前 SHA256 |
|---|---:|---:|---|
| [axis.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/axis.rs) | 14 | 130 | a32530b74b5165f8dc6928d9bdc9eae1cef4ffd4e8f1c23dd6a2696a4e96f541 |
| [input.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/input.rs) | 17 | 87 | 47181c0ca10104db516a8aebd52a09b6e504541fe9f3569b7b65903024f60aaa |
| [mod.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/mod.rs) | 16 | 31 | 270ac2b707b3e1198c9298998bbb1deb59d8fb6207309b27e849e6ae0b2bf35b |
| [settings.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/settings.rs) | 107 | 365 | 9b00ae7cbc42549ad2066029b1f8d6b58e5e861c51370439c300606a8094d4b3 |
| [state.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/state.rs) | 12 | 67 | 66ec03ab2a1b3f89bc4cd4fb5ed7b0e6b5a884a0e3ec73b515ff03edcc4f23b5 |
| [transition.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/axis/transition.rs) | 13 | 79 | 644d2654bcdaf2609d4d9c3d94db196b05f8bbc5ef20ac6a36f03e512dd1d961 |
| [value_scaling.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/input/gamepad/value_scaling.rs) | 9 | 45 | cfa4cf9ca5b1181359f9c7ddf2dca200828fdc11f108aa1df82db357f4b85e14 |

131 项 Core 记录与 137 项主审记录均已绑定起点及最终内容；主审另补六个文件级模块职责。范围包括七个函数、五个类型、十四个字段、九个变体及一个元组载荷、五个常量、七个模块，以及派生 trait、实现、导出、现存注释和各语句块。无参数的默认函数没有虚构参数章节；字段和变体说明身份、尺度、状态约束及生命周期，不套用函数返回模板。

文档说明设备轴标签与设备复合键的关系、原始采样与处理后值的差别、构造和 Serde 的未验证边界、直接映射与带历史接受策略的不同，以及当前值跨帧保持和变化记录按帧清理的边界。常规采样的起点/终值合并、失焦归零、断开追加记录与动作索引取最后源项均限定为已读具体实现的行为。

## 证据与验证

- [Core FIRST 封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w25_gamepad_axis_scaling_core_sol_r1/aggregate.FIRST.seal.json)：SHA256 4b587cf8824eabeaeebad1498c980f6ff913927511049057b29cc43b75aee402。145 个封存成员与七个单元封条保持原样；早先失败的 helper 和恢复记录没有改写。
- [主审独立复核封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-axis-contract-review-20261011-r5/seal.json)：SHA256 246003e46c54a4445f1706d7bc3fa754e6f19f4f45b04fccaaff7527f9829f36。保留 116 条 Core 支持来源，主审采用 38 个实际支持路径、59 段读取范围；变化后的宿主与按钮支持哈希另行绑定，未将旧范围换名为当前证据。
- [范围语言与安装封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-axis-declaration-docs-review-and-language-20261011-r5/seal.json)：SHA256 465925f32612a8926478507633bec2351b9f2f27e9c019a6c376db537d192da9。七文件 Rustfmt stdin 格式与解析通过，换行归一化后输出一致；只含本批新增注释的增量已经逐文件独立复核，反向删除新增片段恢复起点完整字节，正向重建也逐字节匹配当前内容。
- [精确最终覆盖封条](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-axis-final-coverage-20261011-r1/seal.json)：SHA256 6d062f11d1feef472af1339564f99a1502b4e33004900b6a64e322aa38972d5c。137 项主审及 131 项 Core 范围、49 段新增字节、143 处全文 Markdown 静态引用均与最终内容匹配。引用已核对真实定义及作用域，没有运行 Rustdoc 构建或动态验收。

纯注释批次未运行 Cargo 或宽泛测试。语法预检、严格替换锚点、字段引用查询及错误路径读取的失败均有原始记录，恢复版本独立保存；截断输出没有算作全文证据。安装与绑定未保存完整源码快照，也没有写入正式账本。

## 并行与遗留问题

当前按用户指定模型继续并行只读复核：Luna 检查按钮链接、相关测试全文链接及轴文档作用域，Sol 复核输入服务与快照公共契约。具体范围与结果目录互不重叠，子 agent 不编辑源码。轴链接的 [精确任务说明](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/w27-axis-links-assignment-root-20261011-r1.md) 使用本表当前哈希；这些补充结果须经主审接纳，不能替代已经封存的来源或冒称动态通过。

四条 Core 私有观察仍保留：源文本测试结束标记与当前挂载形式不符、宿主桥测试路径定位不足、自定义设置及异常浮点输入的测试证据缺口，以及未验证公开配置的实际边界。本批没有新增正式 BUG/TODO 标签或关闭业务问题；后续动作与支持来源在独立问题记录中。有限相等边界并不能证明轴插值路径一定使用零分母，不把潜在算式风险升级为未经证明的业务缺陷。

正式账本目录虽有用户授权，当前会话的有效写入列表及 canonical `zircon-strict` 配置仍未列入 `docs/_data/source-comment-audit`。待发布证据继续保存在已授权私有目录；未修改权限配置或绕过文件边界。旧协调器继续退役，未启动服务、托盘或队列，历史租约和归属回执均保留。

共享索引在本批只读采样间出现过变化：早先记录的 0 字节锁在后续采样中未出现，缓存 diff 曾有 59 字节路径输出，之后独立读取为空。Root 没有删除锁、修改索引或推断其所有者。阶段提交仍待正式证据发布与当时共享索引、真实验收门槛复核。现有 [Jenkins 边界待发送材料](E:/Git/ZirconEngine/.codex/outbox/source-comment-audit-01a0f168-jenkins-boundary-20261009-r1.md) 保持原样且未发送；当前没有可验证的 Jenkins active 状态，未运行相关命令、推送或通知。

## 完成条件

本阶段只闭合七个精确文件的主审文档与范围语言证据。正式账本发布、补充复核接纳、阶段提交、其它模块和全部第一方源码的最终哈希覆盖仍未完成。待处理文件未清零，问题来源与源码标签的全仓一致性尚未验证；全仓扫描保持进行中。
