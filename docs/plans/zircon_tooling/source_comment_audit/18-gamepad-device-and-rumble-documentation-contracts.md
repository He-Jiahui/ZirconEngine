# 18 — 手柄设备与震动数据的文档契约审查

## 本阶段范围与状态

审查 `core/framework/input/gamepad/device` 与 `rumble` 的六个第一方 Rust 文件。六个文件的文档补充、独立主审复核、格式／解析检查与最终哈希绑定已完成，覆盖记录暂存于获授权的私有证据目录，正式账本发布与阶段提交仍待处理。全仓审查目标继续开放。

按[统一文档规范](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/04-documentation-contract-and-review.md)执行；承接[第 17 阶段](E:/Git/ZirconEngine/docs/plans/zircon_tooling/source_comment_audit/17-input-retention-configuration-and-status-documentation-contracts.md)。函数说明包含概要、使用时机、参数、返回值、约束、所有权与生命周期及参考。数据类型、变体、字段与常量说明各自的角色、有效域和状态拥有者。单行元组字段的契约集中在 `GamepadId` 的字段章节中，原声明字节完整保留。

## 调用关系与设计证据

主审完整读取六个起始文件，逐项复核 103 项 Core 记录，并补充四个具体声明文件的模块边界记录，形成 107 项主审记录。覆盖导入、私有模块挂载、公共重导出、类型／变体／字段、六个函数、派生 trait、参数、初始化与分支。

独立读取的 33 个支持路径、45 个范围连接了当前 App 的枚举及轮询、ABI 输入转换、默认输入管理器、帧快照、待交付队列、宿主请求准备／提交／回滚与 Gilrs 消费过程。结论限定于实际读取的实现：设备编号不含宿主命名空间或连接世代；连接通知拥有元数据；构造震动请求仅构造值；默认帧视图与待交付队列具有不同寿命；出站强度归一化与实际设备执行由后续拥有者承担。

明确解释当前宿主先解析目标，再处理零时长或量化零幅值的空执行路径，以及时长上限、效果准入和停止告警。没有把 DTO 构造、快照复制、排队或空执行当作设备完成确认。79 处文档引用已按实际定义与无条件 core 出口静态核对并绑定最终位置；Rustdoc 构建仍未执行。

## 注释改动与最终内容

新增 32 段、514 行文档注释，六个文件由 111 行变为共 625 行。原有三条文档注释保留。每个文件都在 RAM 中逆向移除本次精确插入，恢复起始哈希、字节数与行数；公共 API 和原程序字节没有改变。

| 文件（均位于 `zircon_runtime/src/core/framework/input/gamepad/`） | 起始／最终行数 | 最终 SHA256 |
| --- | ---: | --- |
| `device/mod.rs` | 4／21 | `12ee6d6d2a2b66b2cb8e30e8cb70919f193b6d395a2f2fc310b240be33b76590` |
| `device/id.rs` | 4／32 | `313f36c4b59f39a8f4d9bca206eca4424e06baea31e8397c0f2bda54e4d87394` |
| `device/connection_info.rs` | 13／107 | `03fda2e1a6987d1be16724c3672cc35eb905cced29927ada82f25b99d0999106` |
| `rumble/mod.rs` | 4／21 | `3d1090430db43083b99962aca4e3cc86479546ba8ff8747a305ba6f43eba8bf3` |
| `rumble/request.rs` | 41／221 | `87f9dbe2bf44f3aa2f1df8415a70256e43d2d2b4fee54a028e125097bc253c88` |
| `rumble/intensity.rs` | 45／223 | `d9341300259569448dfec650796640023de8efe65ae6bfee57f9c44c8657e556` |

## 复核、验证与证据索引

32 段最终候选与六文件实际 diff 均分组完整读取，超出输出预算的读取另留失败与恢复记录。六次限定范围的 Rustfmt stdin 解析／格式检查通过，行尾归一化后的格式精确匹配；限定文件的 `git diff --check` 通过。源码快照没有落盘。纯注释批次复用已有语言证据，没有例行 Cargo、动态测试或硬件验证。

- [Core FIRST aggregate](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/w25_gamepad_device_rumble_core_sol_r1/FIRST.aggregate.seal.json)：`ccf92b0df3761db7015b88935445318430a03436900d6ce0ca0c82207f5bb276`；六个逐文件 FIRST 保持冻结。
- [主审 r3 封存](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-device-rumble-contract-review-20261009-r3/seal.json)：`0cb3fac80ad7f190408ed0cac02a284127311bcad44e5e8e92d10feb42a6e057`。
- [注释安装与语言证据](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-device-rumble-declaration-docs-review-and-language-20261009-r3/seal.json)：`09200ddb89712d269992a6e2c2dbe603091b5c3c6cf22b7208148569e8cc46fd`。
- [最终覆盖绑定 r2](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-device-rumble-final-coverage-20261009-r2/seal.json)：`9e6b8dbb9ed7ae61ce5a5d5d4dadbf2e874b2ff2fbe730e584ad036a0e4bf629`。r1 的汇总常量计数使用了错误分类键；原包保留，r2 据实际三条 `constant` 对象更正计数，原源码与审查范围不变。
- [逐文件最终记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-device-rumble-final-coverage-20261009-r2/coverage-final.jsonl)保留起始／最终哈希、主审／Core 数量、字节还原与待发布状态。
- [问题边界记录](E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168/results/root-gamepad-device-rumble-contract-review-20261009-r3/issue-decisions.jsonl)保留六项支持测试或验证缺口观察。本批没有新增正式 `BUG:`／`TODO:` 标签，也没有关闭这些观察。

## 来源边界与待完成事项

Core 的 104 项首次支持读取保持原哈希和原范围。期间 `dynamic_api/session/state.rs` 与 `events.rs` 出现外部业务改动，主审单独重读当前相关范围并固定新哈希；旧全文证据没有变成当前全文审查，也没有转成注释作者证据。本阶段只编辑表列六个文件，未取得或宣称上述业务文件的作者归属、产品采纳或业务同意。

阶段提交仍为零。现存 `.git/index.lock` 的拥有者未知，数据保持原样；正式账本目录的用户授权持续有效，但当前权限清单未包含该目的地，私有记录暂未发布。旧协调器处于退役状态；已有 Jenkins 边界说明留在 outbox，尚未发送或获得接受回执。现有产品排除项及 Editor06、Runtime11、SDK、Sound 的历史边界保持有效。

独立分区继续推进；已封存的按钮、轴、输入服务与快照、模块构造和公共外观候选仍分别等待主审／语言检查。清单哈希抽样仅提供版本线索。只有全清单逐文件最终覆盖、问题账本一致性及实际提交／接受门槛均满足后，才报告全仓完成。
