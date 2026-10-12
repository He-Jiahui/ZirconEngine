# 09 — 拾取契约文档与当前并行审查

## 阶段状态

全仓目标保持 **进行中，未完成**。本阶段完成拾取命中数据及后端接口、阶段标签、开关配置共 4 个文件的声明文档、主审复核、独立核心复核、对应语言检查与最终哈希覆盖绑定。正式账本发布和阶段提交继续待处理。

统一规范沿用 [04 — 声明文档与审查契约](04-documentation-contract-and-review.md)，前批状态保留在 [08 — 物理查询与输入缓冲声明文档](08-physics-query-and-input-buffer-documentation.md)。对象、语句块、候选决定、调用证据和问题关系保存在独立证据包中。

## 本批已核验结果

| 项目 | 命中数据 | 拾取接口与配置 |
| --- | --- | --- |
| 精确范围 | `hit_data.rs` | `backend.rs`、`schedule_label.rs`、`settings.rs` |
| 主审记录 | 14 条 | 49 条 |
| 独立核心记录 | 14 条 | 49 条，另有 3 条手写 `Default` 补充复核 |
| 新增文档 | 7 段、65 行 | 31 段、248 行 |
| 最终行数 | 27 → 92 | 75 → 323 |
| 函数文档附着检查 | 1 个声明 | 7 个声明，含手写 `Default::default` |
| Rustfmt 格式与解析 | 1 个文件通过 | 3 个文件通过 |
| 原内容核验 | 精确移除新增片段后逐字节恢复审查起点 | 精确移除新增片段后逐字节恢复审查起点 |
| diff 与最终覆盖 | 限定范围检查通过；对象范围与最终哈希绑定 | 限定范围检查通过；对象范围与最终哈希绑定 |

声明文档说明职责、使用时机、必要约束、所有权及生命周期，包含适用的 Markdown 参数、返回和参考章节；字段与枚举变体分别说明其身份、单位、允许状态及实际用途。无参数函数保留真实契约，不制造参数说明。原有中文、英文注释及程序字节完整保留。

[命中数据](../../../../zircon_runtime/src/core/framework/picking/hit_data.rs)明确生成方与使用方的边界、两个独立可选几何字段、相机标识与编辑器占位值、排序键的实际含义，以及按压和拖拽快照可以跨帧保存的时效性。类型和构造函数不会证明实体仍然存活，也不会执行几何验证。

[后端接口](../../../../zircon_runtime/src/core/framework/picking/backend.rs)区分声明能力、元数据顺序与命中批次的实际排序字段。文档说明共享输入、按值返回和实现者责任；同步返回不授予后台完成保证，名称的自定义转换也保留其实际副作用与 panic 边界。

[阶段标签](../../../../zircon_runtime/src/core/framework/picking/schedule_label.rs)说明报告标签、计数单位和实际控制流的关系。[开关配置](../../../../zircon_runtime/src/core/framework/picking/settings.rs)区分总关闭时的状态清理与仅关闭射线时继续推进悬停和事件，并说明构造配置与实际生效的时机。手写 `default()` 的旧零新增决定保留在封存证据中，另以独立补充复核完成函数声明文档。

## 证据与来源

私有证据根目录为 `E:/cargo-targets/zircon-local/source-comment-audit/20261003-01a0f168`。封存后的原子审包保持不可改写；本批作者记录只指向新增的精确注释片段，不接收外来业务内容的作者身份。

| 证据 | 私有相对路径 | 封条 SHA256 |
| --- | --- | --- |
| 命中数据注释与语言检查 | `results/root-picking-hit-data-declaration-docs-review-and-language-20261009-r1` | `0f49d543f2f9eb1b4348867fd62a365a2e813f9eea790694c7424c6dbb143c83` |
| 命中数据最终覆盖 | `results/root-picking-hit-data-final-coverage-20261009-r1` | `aa0dfa9979a4f19a206dc6abe7b25811fa64a965f366d783ce5997344bcf9b3f` |
| 拾取接口独立核心复核 | `results/w24_picking_pipeline_core_sol_r8` | `337fe7c55127a0bb6c9181b0189b044a802ba731cf08087de178edcd95ec7692` |
| 手写 Default 补充复核 | `results/w24_picking_settings_default_core_sol_r8_addendum` | `e7af50f770c6852849484fd461f59cb2d3193b11494661f832a22a187266402b` |
| 拾取接口注释与语言检查 | `results/root-picking-pipeline-declaration-docs-review-and-language-20261009-r1` | `8466b91345673ae5d2f9920e2f0bbb6b7269e2cc0099a333db98184e4c0fdb47` |
| 拾取接口最终覆盖 | `results/root-picking-pipeline-final-coverage-20261009-r1` | `67c2d436fa096e64a6789c7f706c423acf812c78f2d878862b5a04f6b59e12dd` |
| 当前 r8 原生指派回执 | `results/root-wave24-r8-native-dispatch-receipt-20261009-r1` | `03470887a4781eb9cf96db7a578c4b538d4111b226541283b8ca5897dc543b08` |

既有 `CR-PICKING-0001` 的源码标签、问题来源和打开状态保留，本批新增 TODO／BUG 标签为 0。核心复核对候选文字的修订意见与源码缺陷分别记录，不把文档复核问题计为新源码 BUG。

## 当前并行审查

当前采用主审加 3 个 `gpt-6-luna max`、1 个 `gpt-6.1-sol max` 的安排，总并发上限为 5。Luna 负责互不重叠的文件范围和调用证据；核心契约与最终候选决定交由 Sol 独立复核，主审继续逐批复核及最终绑定。

当前独立范围为输入录制模块及测试、动作求值工作区及测试、RPC；核心复核已转入动作求值代际及其测试。每个完整文件及时封存，未封存草稿、成功指派及工具消息均不算完成证据。前阶段 r7 指派回执保持历史事实，当前无法取得终态的 r7 单元保持待处理。

子审只读取源码并写获授权的精确私有目录。跨模块读取不扩展编辑范围；所有安装前检查以当前精确哈希为准。已确认的工具失败另存错误记录，修复工具载入或脚本编码不授予源码完成状态。

## 验证与剩余条件

本批已完成静态调用契约、对应 Rustfmt、原字节还原、限定 diff 和覆盖检查。Cargo、动态测试、性能接受和 Rustdoc 构建未运行；全工作区、里程碑及业务修复接受条件继续独立保留。

阶段边界仍在 `main`，缓存区为空；零字节 `.git/index.lock` 的所有者尚未证明，锁及共享索引保持原样。阶段提交及其附带通知待处理。

正式账本写入范围虽已获用户授权扩大，当前实际写入条目仍未匹配 `docs/_data/source-comment-audit`，因此最终覆盖保存在获授权私有目录，正式发布待处理。遵循[文件权限规则](../../../tooling/codex-file-permissions.md)与[协调器退役规则](../../../tooling/coordinator-retirement.md)，继续保存历史归属、既有回执、外来业务修改及产品排除范围。

全仓完成仍要求全部第一方源码逐项审查记录匹配最终内容、待处理文件清零、源码 TODO／BUG 与规范问题账本一致，并完成必要复核、验证和阶段提交。
