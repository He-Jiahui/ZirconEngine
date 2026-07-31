# Editor09 asset content generation projection

## 目标与边界

- 修复 `asset-pane-projector-repeated-model-scans`：Activity/Browser 稳定 paint 不再扫描通用 node model 或重复解析 control identity。
- 几何、解析后 identity、固定节点、滚动组、viewport 与 content extent 由资产模型 generation 一次发布。
- retained host 只消费共享元数据，并按 damage clip + scroll 生成固定节点与可见滚动组的精确行计划。
- 不新增 painter cache，不保留旧 identity parser，不以截断节点或兼容 fallback 伪造常数时间。

## 实现清单

- [x] `ModelRc<T>` 支持 clone-shared typed metadata，并在 view DTO → host DTO 映射时保留同一 `Rc` 元数据分配。
- [x] Activity 与 Browser builder 在最终布局完成后，把 View DTO 映射为中立 `AssetContentPaintNodeInput` 并构建 `AssetContentPaintMetadata`；workbench metadata owner 不反向依赖 layouts DTO。
- [x] 元数据一次解析 Activity list、Browser list/thumbnail identity，并发布 viewport、extent、folder row count、fixed rows 与排序 scroll groups。
- [x] Activity/Browser projector 构造和 transform 只查元数据；稳定 paint 的 `row_data` 与 identity parse 为 0。
- [x] scrollbar 从同一元数据读取 viewport/extent；asset tree row count 同时改为 borrowed iteration，避免 DTO clone。
- [x] template draw pipeline 支持 transform 提供 exact row visit plan，只 clone/投影计划中的节点。
- [x] 删除 painter-owned `asset_content/identity.rs`，不保留 alias、wrapper 或双解析 owner。
- [x] 加入共享元数据 identity、DTO 投影保留、精确行像素行为、Activity 可见组与 10k Browser thumbnail 可见组边界测试。

## 验收状态与剩余项

- `python -m unittest tools.tests.test_editor09_asset_content_generation_projection -v`：6/6 通过。
- exact Rust 文件已由 Rust 1.94.1 `rustfmt` 解析并格式化；scoped `git diff --check` 通过。
- managed Cargo、Activity/Browser 产品像素等价、1/1k/10k row_data/parse/clone/alloc/CPU p95 与独立 review 尚未完成；在这些门禁完成前 failure 保持 open，不写 fixed return。
- 当前 exact21 Session：`editor09-asset-content-generation-projection-r2-20260719`；不得吸收 Editor09 其他三个 open performance failures 或其外部 dirty scope。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据/剩余项 |
|---|---|---|---|
| 2026-07-19 15:02-15:22 +08:00 | `source_complete_static_green_validation_pending` | generation-owned typed paint metadata、中立 generation input、DTO metadata preservation、Activity/Browser zero-scan projector、metadata scrollbar、exact visible-row draw、旧 identity owner 删除、10k visible-group guard | 静态合同 6/6、production workbench→layouts 反向依赖 0、Rust 1.94.1 rustfmt、scoped diff-check 通过；待 managed Cargo、产品像素/规模数据、独立 review、failure return 与 exact-manifest managed commit。 |
