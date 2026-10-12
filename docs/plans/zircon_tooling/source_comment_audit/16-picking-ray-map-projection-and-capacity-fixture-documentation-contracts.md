# 16 — 拾取射线、映射与容量夹具文档契约审查

日期：2026-10-09。全仓目标仍进行中；本阶段完成三个文件的独立主审、中文注释、语言检查与最终哈希覆盖绑定。正式账本发布、验证交接和提交仍待处理。

## 范围与完成状态

本阶段按 [04 统一文档规范](04-documentation-contract-and-review.md) 复核定义、直接调用、派生、模块挂载、测试条件、输入配对和生命周期。源代码共 337 行，新增 49 处注释、619 行，其中 609 行文档注释、10 行私有顺序与设计说明；最终共 956 行。原程序、公共 API 和原有注释字节完整保留。

| 精确文件 | 原行数 → 最终行数 | 最终 SHA256 |
| --- | ---: | --- |
| [ray.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/picking/ray.rs) | 78 → 211 | `6e138eb42421707988ed2d380df41919d30bc5921c8dba5165bd42e0ffb6499d` |
| [ray_map.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/picking/ray_map.rs) | 122 → 427 | `7571f4c5e24d22c85102b4eca23a83591626f794ba847cf12f676f0b99796849` |
| [rebuild_capacity_tests.rs](E:/Git/ZirconEngine/zircon_runtime/src/core/framework/picking/ray_map/tests/rebuild_capacity_tests.rs) | 137 → 318 | `01b8b8af441079abdd0bc14e33d4c20e55e38469325631cf724188533e79c105` |

145 项主审记录和 145 项 Core 记录均绑定最终文件及定义范围。21 个函数、4 个数据类型、11 个字段、4 个常量获得相应文档。45 个 Core 主候选和 4 个旋转前提补充保留原来源，主审另补 4 个字段文档；25 个候选采用原文，20 个经主审补充或整理。93 个文档引用定义完成当前声明与作用域核对，105 个支持范围保留原审查版本、当前版本及字节范围证据。

## 本阶段明确的契约

- 射线受检构造在归一化之后检查原点、方向有限性和非零方向；公开字段与直接插入仍由调用方维持数值约束。
- 投影使用配对的视口局部像素、当前尺寸和相机快照，说明实际闭区间、宽高比、投影分支及未参与计算的快照字段。结果拥有数值，后续输入或相机变化需要重新采样。
- 相机旋转的单位四元数前提、底层条件断言和检查顺序已核对。有效 Cargo 特性图未验证；文档描述条件行为，未声称当前构建启用了该断言。
- 重建先丢弃旧结果，再估计容量并依输入顺序写入。同键保留最后成功值；被跳过的组合不擦除本轮值，容量或旋转失败不回滚。存储文档也明确扩容可能重新分配和销毁释放仍持有的存储。
- 普通测试分别覆盖空输入结果和源码预留表达式。忽略性能夹具每轮新建并销毁映射，计时包含销毁；31 组样本的秩、交替先后顺序、容量报告及阈值均按实际实现说明。

原 `CR-PICKING-0001` 标签及问题账本继续开放，本阶段仅引用其已有归属和调用证据，没有新问题标签、业务修复或关闭声明。

## 验证与证据

三个文件通过 Rustfmt 标准输入解析和精确格式比较。主审完整阅读实际 diff；存储生命周期的一处新增说明另经独立修订和受影响文件格式检查。最终逆向删除本任务注释后，三个文件与各自审查起始字节完全一致；最终范围、文档、引用和测试文本守卫已核验。当前三个文件的 `git diff --check` 通过。

未运行 Cargo、Rustdoc、动态测试、性能测试或 Jenkins 验收。引用核对为静态定义与作用域证据。逐项日志置于私有证据包：

- [独立主审证据](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-ray-map-current-source-and-contract-review-20261009-r3/seal.json)：`80b3a168a3b675e262fb8035e3523027e474cd2ff61b81224051f7a13cbc5dc2`。
- [语言与精确注释差量](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-ray-map-declaration-docs-review-and-language-20261009-r5/seal.json)：`8e2e4d38832c736f61e494133fc1ede607557690d729b5051ab4055b87884d27`。
- [存储生命周期修订](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-ray-map-storage-lifecycle-refinement-20261009-r1/seal.json)：`15b976111c29945e7e1486a8707eded1d7b4c458ca1f440a31922229a2eb46e4`。
- [最终逐项覆盖与引用绑定](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-ray-map-final-coverage-20261009-r1/seal.json)：`d929116a34ef14316aa369c83fe5b1d03379037888818559586b758c9e125895`。

原定位错误、预检失败和恢复版本完整保留。数学重导出的原错误行号与纠正后的 16／49 行证据分别保存，未改写旧封条。历史指针位置支持版本仅在内存中按既有精确注释差量恢复，原作者和覆盖来源保留。

## 并行审查与恢复

三个 SourceReadonly 分区继续使用 gpt-6.1-sol max：按钮五文件／160 行、轴与缩放七文件／188 行、指针与窗口恢复六文件／249 行。精确源路径互不重叠；子审查仅写各自私有结果目录，主审复核和语言检查保持独立阶段。

指针与窗口旧汇总误将范围读取的完整 raw 字节存入三个 hex 字段，实际生成三份完整源码副本。该汇总已撤回。主审核对 115107 字节附件的精确哈希后，先记录无源码的错误元数据，再仅删除该误生成附件；旧 seal、terminal 和六个逐文件 FIRST 保持原字节。新 r2 恢复目录复用原 FIRST，保留错误归因和撤回事实。恢复汇总仍待完成与复核。

[撤回与精确清理回执](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-pointer-window-aggregate-withdrawal-20261009-r1/seal.json)：`e8d28aebff12ef416c8f2b5e68907e8133dfcabc99249fdc27af15f7bb631f63`。本阶段主审没有创建完整源码快照，误生成副本已清理；该错误未影响源码。

## 待处理门槛

- 正式账本目录 `docs/_data/source-comment-audit` 已获用户增加授权的指示，但现行配置及当前聊天写入范围仍缺少该目的地 grant；私有覆盖与问题证据尚未发布。没有修改权限配置或宣称执行探针已验证。
- `.git/index.lock` 当前存在且为 0 字节，所有者未确认，保持原样。当前分支 `main`，暂存区为空；未暂存、提交、推送或发送企业微信通知。
- 退休本地协调器保持退休，不调用注册、租约、归属或队列接口。Jenkins `active.json` 当前不存在；原交接 outbox 仍未发送，没有启动服务、功能测试或额外验收。
- 产品排除清单和所有历史注释来源保留。后续业务内容不作为本任务新的注释作者或覆盖证据。
- 全仓逐文件覆盖、未处理对象清零、问题标签与账本一致性、正式发布及阶段提交仍未完成。

## 下一步

继续复核已封存的输入保留配置与其他核心候选；接收三组当前分区结果后逐批独立复核。满足实际写入、索引和验证交接门槛后，再发布精确覆盖与阶段提交。完成条件仍按全仓最终内容逐文件检查。
