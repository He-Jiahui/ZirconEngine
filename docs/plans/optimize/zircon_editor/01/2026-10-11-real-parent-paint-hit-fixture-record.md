# 真实父级 paint/hit 测试修复记录

日期：2026-10-11

## 已采用的代码

完整两文件候选 `1c6feb5f878279060d27bbc2292837881ac81228798925fe9808b64783de886c` 已一次采用。六个测试场景通过实际 Window → Ability → BlendSpace 路由取得当前 Source、viewport 父级和共享 paint/hit owner，替换旧 aggregate 的人工注入。十二个原 Rust 用例全部保留：六个迁移场景、六个未改主体，没有新增用例或 ignored 变更。

保留正向画面与命中、越界 clip、未拥有高层节点排除、Search 至少四种颜色、借用 streaming workset、typed Apply，以及回到 Scene 后整 viewport sentinel、完整画面基线和无旧命中的退休断言。六个涉及环境的场景持有实际 MutexGuard。

## 共享输入与实际检查

原同序 100 个输入保留：94 个未变、6 个外来输入经独立兼容追加分类。五个外来变化可完整逆证；Level 资产按当前入口、父级和 clip 合同审查接受，其历史 6 字节差异未完全归因，未宣称历史全文相等。共享 index 和临时 lock 的外部生命周期独立记录。

采用前两次 Source 检查通过；采用后两文件 LF 后像精确、98 个只读输入不变。实际 rustfmt check、限定 whitespace check 和完整候选 reverse-check 均通过。没有运行 Cargo 或原生测试。

[实际采用收据](../../../../../.codex/outbox/editor-real-parent-paint-hit-fixtures-v1/actual-env-lock-guard-v3/actual-canonical-apply-v1/actual-canonical-apply.json)  
[独立兼容审查](../../../../../.codex/outbox/editor-real-parent-paint-hit-fixtures-foreign-guard-compat-v1/review.json)  
[完成列表](../../../astra/features/editor/2026-10-11-editor01-real-parent-paint-hit-completion-list.md)

## 待验证

十二个既有精确名字与原 V9 第 15 项 OR 条件无交集，将并入原命令。同批 Generated 五个名字合并后最低数量为 191；原 15 项、14 个性能 producer、features/profile 和原性能阈值保留。终态需核真实执行的用例名和数量。性能、产品、Jenkins 与完整组件覆盖仍未验收。
