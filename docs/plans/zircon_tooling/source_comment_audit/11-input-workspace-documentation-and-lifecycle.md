# 11：输入工作区文档与生命周期边界

## 范围与规范

承接 [10 阶段](10-input-generation-and-recorder-documentation-contracts.md)，按 [04 文档规范](04-documentation-contract-and-review.md) 完成输入求值工作区及所属测试的系统文档。首段说明用途；每个函数都有参数、返回、生命周期和参考章节。类型、字段、私有方法、测试辅助函数、属性和语句块分别保留审查记录。

本计划记录阶段状态。定义、调用、原字节范围、逐对象结论、工具错误和问题证据存于独立封存产物中。程序、公共 API、现有注释和业务归属保持原状。

## 已完成的文件批次

| 批次 | 文件数 | 起始／最终行数 | 新增注释段／行 | 函数文档 | 主审／核心记录 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 输入工作区及测试 | 2 | 279／832 | 39／553 | 22 | 92／92 |

两个文件的 Rustfmt 检查和限定范围 diff 空白检查均通过。新增注释精确移除后恢复起始源码字节，所有对象记录及函数文档已绑定最终内容哈希。未运行 Cargo、动态测试或 Rustdoc；这些检查不代表 Jenkins、性能或里程碑验收。

文档说明工作区与编译代际的配对关系、独立求值器与默认管理器提供独占访问的路径，以及输入切片、索引、结果副本和公开投影的生命周期。跨帧复用容量、映射替换后的逻辑清空、上下文空列表的外层规则、饱和增长计数的含义和计时区间均有明确边界。

### 封存证据

- [主审定义与调用复核](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-current-source-review-20261009-r1/seal.json)，SHA256：`e18554624ad11c290b0ecad0fa098fcb37b9b8a4ce816a92e34fb0d255ff81ab`。
- [核心独立复核](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w24_workspace_core_sol_r9/seal.json)，SHA256：`ee207fb736fcbc4a91e42353a8cad8fc56debcc10a8b477fa4ba3eee7f4abfe7`。
- [逐项采纳及生命周期补充](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-core-and-documentation-addendum-20261009-r1/seal.json)，SHA256：`db36d6609850ebe849c04e58fbb58e845e0b04fd2a00cb72ff845b39c5acab3e`。
- [Markdown 类型呈现修正](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-markdown-refinement-20261009-r2/seal.json)，SHA256：`0883224c5f5751071c05a86e64dd8451a902f033c55a743f9953c08417e7399a`。
- [安装配置补全](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-installation-metadata-20261009-r3/seal.json)，SHA256：`a2d19510861dbeacf6ffc4cffc20cc477f96371779b17f5d61459d9b45dbdd8b`。
- [注释写入与语言检查](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-declaration-docs-review-and-language-20261009-r3/seal.json)，SHA256：`707615fafcc64bb7f673612353fb4122e449f329d895d48245a5e1b68191f40d`。
- [最终覆盖绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-workspace-final-coverage-20261009-r1/seal.json)，SHA256：`975d000a6567e85f6a851a9a7e0cbbc5b3c1cbcebc7a358f2c6aa0065a81f7f9`。

## 问题与证据边界

既有 `CR-INPUT-0001` 继续开放；原标签、原账本行和来源保持不变。工作区文本守卫的既有私有观察 `OBS-W24-WORKSPACE-DELIMITER` 也保持开放：实际声明中的 `#[path]` 使第一处精确分隔符缺失，`expect` 先于后续形状断言失败。新注释未伪造函数标记或让该守卫误报通过；静态模式存在不计为动态测试通过。

旧 Luna 单元、未封存修正及无效 JSON 附录分别保留原哈希和证据缺口。新核心复核与主审结论没有改写原封条，也没有将当前业务内容重新标注为审查作者。安装配置缺项的一次失败在源码写入前终止；修正后的实际成功回执单独保留。

## 并行审查进展

默认动作管理器及测试、Picking 指针值类型、命中记录与目标、指针语义事件的四组 `gpt-6.1-sol max` 核心复核均已终止并封存，源码修改为零。它们仍待主审逐项复核、语言检查和最终哈希绑定；子审查封存不计为模块验收完成。

下一批轴索引、消费输入索引及两份所属测试已按四条精确路径分派核心审查，允许跨仓只读追踪，私有结果目录独占。每个文件先封存，再聚合；本批工作区新增注释的前后哈希继续与其他审查的实际读取分开记录。

Luna 消息载荷不可读及未交付工具证据的记录保持原状，不计为有效覆盖。独立模块继续推进。

## 账本、交接与提交门槛

正式账本目录写入授权尚未进入本会话的生效文件授权，私有证据仍待发布。用户已授权告知既有 Jenkins 会话；[交接材料](E:/Git/ZirconEngine/.codex/outbox/source-comment-audit-01a0f168-jenkins-boundary-20261009-r1.md) 已准备，SHA256 为 `fdec705e4e5ea24f453de178c065ed053dc51f1e6a672f4d978b27dea8f2f304`。当前工具没有会话发送入口，状态为未发送、未送达；没有创建重复会话或启动服务。

最近只读检查显示分支为 `main`、暂存区为空，既有 0 字节索引锁仍保留且归属未证实。阶段提交继续待处理。没有暂存、提交、push 或外部通知，没有调用已退役协调器，也未提交新的 Jenkins 验证请求。

后续继续复核核心契约、相关测试和其他第一方源码。只有正式覆盖记录与所有最终内容哈希匹配、问题标签及账本一致、待处理文件清零，并完成适用提交与验收门槛后，才报告全仓完成。
