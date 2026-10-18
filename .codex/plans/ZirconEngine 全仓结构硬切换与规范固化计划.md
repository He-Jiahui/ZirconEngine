# ZirconEngine 全仓结构硬切换与规范固化计划

## 摘要
- 采用一次性 `workspace` 级 hard cutover：先冻结并吸收当前活跃切片，再统一做结构拆解、所有权重排、旧路径删除，最后只做一次编译与测试恢复。
- 最终结构以固定三包为准：`zircon_app` 只做宿主入口，`zircon_runtime` 只做 runtime 吸收层与受控公开面，`zircon_editor` 严格收束到 `core/scene/ui` 三域。
- 不允许中途求稳：结构拆解阶段允许编译失败；但不允许留下兼容 `pub use`、shim、bridge、legacy 目录或过渡 facade。
- 重构完成后，把结果固化为“Repo 内技能 + docs”双标准：repo-local skill 作为执行规范，`docs/` 作为人类可读权威说明。

## 关键改动
- **执行前协调**
  - 先冻结当前活跃重叠 session，把它们视为输入，不再继续并行扩散。
  - 执行开始时创建/更新一份当前任务 session note，并把已完成或失效的活跃 note 归档，避免后续多 agent 再次交叉改写同一区域。
  - 先产出一份“旧 owner -> 新 owner”映射表，作为所有移动、删旧、改引用的唯一依据。

- **第一批只做结构，不做修修补补**
  - 先完成文件/目录移动、模块下沉、owner 重命名、测试树拆分、资源目录迁移。
  - 这一步只允许做路径级搜索和结构级审计，不做编译恢复循环。
  - 所有旧路径在消费者切走后立即删除；不保留中转层。

- **`zircon_editor` 收口**
  - `zircon_editor/src/lib.rs` 降为纯结构入口：只保留 `core`、`scene`、`ui` 声明和极小 curated surface；不再做大面积扁平 `pub use`。
  - `core/` 只保留编辑状态机、intent/history、editor-event、与 UI 无关的宿主侧服务；从 `core` 清出 workbench、layout、window、view、UI asset session 等真实 UI owner。
  - `ui/host/` 成为唯一的 UI 宿主 owner：`EditorManager`、window host、builtin layout/view 注册、workspace/startup/persistence、asset-editor session orchestration 全部归这里。
  - `ui/slint_host/` 只保留 Slint adapter、native window、pointer bridge、viewport frame 导入这类平台/壳层 glue；不再承载 asset editor 或 workbench 的业务 owner。
  - `ui/asset_editor/` 彻底目录化，保留 `binding/`、`preview/`、`session/`、`source/`、`style/`、`tree/` 等子树为权威 owner；删除当前 flat + folder 双持有残留，尤其是 `session.rs`、`binding_inspector.rs`、`preview_mock.rs`、`source_buffer.rs`、`style_rule_declarations.rs`、`tree_editing.rs` 这一类重复入口与巨型聚合文件。
  - `ui/workbench/` 独占 layout/model/view/reflection/snapshot/startup/project；`ui/mod.rs` 仅做结构声明与受控导出。
  - `scene/` 继续只保留作者态场景与 viewport/gizmo/handle/pointer 路由，不把 UI host 或 runtime world 逻辑拉回这里。

- **`zircon_runtime` 收口**
  - `zircon_runtime/src/lib.rs` 降为结构入口：只保留顶层模块声明、模块注册面和极小必要公开面；移除 root flattening 和仅用于迁移期的内部便捷导出。
  - `graphics/mod.rs` 只保留稳定 runtime-facing surface；`ViewportRenderFrame`、icon-source seam、深层 renderer 构造辅助一律留在 crate-private 或子树内部。
  - 对 `graphics/compat`、`service`、以及其他带迁移味道的目录做强制审计：若只是旧路径延续就删除；若还有真实职责就改名并并入明确 owner 子树。
  - `scene/world/` 继续按行为族拆分大文件，把 `project_io`、`property_access/write.rs` 一类热点压回单一职责文件。
  - `ui/` 保持 namespace-first 结构；运行时正式 `.ui.toml` 入口资源统一迁出 `src/`，进入 crate `assets/ui/runtime/fixtures/`，删除从 `src` 读取生产资源的正式加载路径。
  - `src/tests/mod.rs` 与过大的 graphics/editor boundary 测试拆成按域分组的测试树，例如 `runtime_absorption/`、`graphics_surface/`、`scene_boundary/`、`ui_boundary/`、`extensions/`，不再让 umbrella test file 继续膨胀。

- **`zircon_app` 保持薄宿主**
- `zircon_app` 不再持有 runtime 内部排序知识，只通过 `zircon_runtime::builtin::builtin_runtime_modules()` 获取 runtime builtin module 列表。
  - `entry/` 只做 profile 选择、模块拼接、宿主启动与 handoff；不吸收 runtime/editor 的具体业务结构。

- **文档与标准固化**
  - 更新 `docs/engine-architecture/`：记录 workspace owner map、root file 规则、hard-cutover 规则、crate root/public surface 规则。
  - 更新 `docs/editor-and-tooling/`：记录 `core/scene/ui` 边界、`ui/host` vs `ui/slint_host` vs `ui/asset_editor` 的 owner 约束、巨型文件拆分规则。
  - 更新 `docs/assets-and-rendering/`：记录 runtime graphics/UI 公开面约束、运行时资源必须放 `assets/` 的规则。
  - 用 `skill-creator` 在 `.codex/skills/zircon-project-skills/` 下产出 repo-local skill，把最终目录规范、cutover 原则、root wiring 红线、禁止混域/禁止 shim/禁止生产资源放 `src/` 写成可复用技能。

## 公共接口与类型规则
- `zircon_editor` crate root 只保留顶层入口类型与少量高层入口；asset editor、workbench、binding、viewport、host 等 specialist type 必须从各自 owner path 直接访问，不再经由 crate root 扁平导出。
- `zircon_runtime` crate root 与 `graphics` root 停止暴露深层 renderer/visibility/frame assembly 内部类型；公开面只保留稳定模块入口、受支持 DTO、显式允许的 runtime-facing contract。
- `UiAssetEditorSession` 及同族类型只从 `zircon_editor::ui::asset_editor` 公开；workbench 类型只从 `zircon_editor::ui::workbench` 公开；scene 作者态类型只从 `zircon_editor::scene` 公开。
- runtime fixture 加载接口改为 crate `assets/` 路径驱动；旧 `src` 内 fixture 路径和任何兼容解析分支一起删除。

## 测试与验收
- **结构审计**
  - 搜索确认被移除的旧 owner path、`compat`/`shim`/`bridge`/`legacy` forwarding layer、以及 asset editor 的 flat/folder 双路径全部消失。
  - 搜索确认生产 `.ui.toml` 入口资源不再位于任何 crate 的 `src/`。
  - 检查 `lib.rs`、`mod.rs`、`main.rs`、`binding.rs` 仅剩结构声明与窄导出，不再承载混域实现。

- **定向回归**
  - `zircon_app`：builtin module bootstrap 与 profile handoff。
  - `zircon_runtime`：scene property/path access、graphics public-surface narrowing、runtime UI asset loading、boundary assertions 拆分后仍成立。
  - `zircon_editor`：asset editor session/replay/style/tree 流程、workbench layout/view/host orchestration、Slint host parity、viewport binding 与指针路由。

- **最终恢复门**
  - 全部结构完成后再做一次 compile/test recovery wave。
  - 最终验收以 `cargo check --workspace` 和 `cargo test --workspace` 为准。
  - 因 `E:` 盘空间压力，默认使用共享或 `D:/cargo-targets/...` 作为 target dir；剩余失败只能是环境问题或明确标注的无关旧问题，不能是迁移残留。

- **标准物验收**
  - 新 docs 只引用收敛后的路径，不允许继续描述旧路径。
  - 新 repo-local skill 只指向最终结构，不为旧结构保留兼容说法。

## 假设与默认值
- 当前活跃 session 在执行这次计划时先冻结并吸收，不继续并行推进。
- 本次 refactor 允许在“结构拆解阶段”保持工作区不可编译；但不允许用兼容层换取中途稳定。
- 没有任何迁移期 `pub use`、alias module、bridge folder、临时 facade 可以留到收尾之后。
- 规范主落点是 repo-local skill，`docs/` 是解释层而不是第二套相互竞争的标准。
- 现有脏工作区视为本次 cutover 的输入基线，不按“先清干净再开始”处理。
