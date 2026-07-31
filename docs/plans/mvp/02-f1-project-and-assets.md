---
related_code:
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/ui/host/startup
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/registry/asset_registry_index.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager
  - templates/projects/renderable-empty
related_tests:
  - zircon_app/src/entry/entry_runner/editor/tests/gui_startup.rs
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_runtime/src/asset/tests/pipeline/manager/project_open.rs
  - zircon_runtime/src/asset/tests/project/manager/artifact_cache_imports.rs
  - zircon_runtime/src/asset/tests/registry_index
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
status: blocked_by_f0
gate: F1
last_refined: 2026-07-24
---

# F1 项目与资产 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans`。项目格式或 migration 变化使用 Runtime 04/Editor 11 owner；测试阶段使用 Windows coordinator validation。

**Goal:** 通过一个受支持的 editor 产品入口创建 `RenderableEmpty` 项目，关闭 editor 后从同一路径重新打开，并证明 manifest、project settings、asset registry 和必需资产来自持久权威而非 fallback-only state。

**Architecture:** `ProjectAuthority` 是创建/打开权威，Runtime 04 manager 负责 scan/import/registry generation，App entry 只解析请求并调用同一 owner。测试和产品运行共享一个新建项目目录，F2-F5 继续使用该目录。

**Tech Stack:** Editor GUI startup args、project manifest TOML、asset registry/index、template pack、staged Windows editor。

---

## 1. 入口条件

- [ ] F0 阶段退出清单完成，staged editor 可从独立目录启动并首帧退出。
- [ ] F1 Session 已领取 project authority、startup、template contract 和本子计划所需 lease。
- [ ] Runtime 04 中直接影响 project open/registry generation 的 failure 已分类；非阻断性能 failure 不进入本门槛。
- [ ] validation lane 为 F1 分配全新项目根；目录在测试前不存在，避免复用旧 registry/cache。

## 2. 唯一支持路径

F1 固定以下产品路径，其他 Welcome/recent/Hub 路径不作为首个 MVP gate：

1. validation driver 先设置 `$freshRunRoot = Join-Path $evidenceRoot 'project'` 并创建该空目录；staged `zircon_editor` 使用 `--create-project --project-name ZirconMvpFixture --location $freshRunRoot --template renderable-empty`。
2. 创建成功后正常关闭 editor。
3. validation driver 设置 `$mvpProjectRoot = Join-Path $freshRunRoot 'ZirconMvpFixture'`；staged `zircon_editor` 使用 `--project $mvpProjectRoot` 重新打开。
4. 后续 F2-F5 使用同一绝对项目根和同一 manifest identity。

`$evidenceRoot` 是 coordinator validation run 分配并写入 manifest 的绝对证据目录；`$freshRunRoot` 和 `$mvpProjectRoot` 都由上面的固定表达式派生，不接受调用者另传第二个项目路径。

## 3. F1 必需持久输入

- `zircon-project.toml`：项目名、format/version、default scene、asset roots 和启用插件/profile 均可解析。
- `assets/scenes/main.scene.toml`：作为 default scene 存在并可进入 registry。
- `assets/models/cube.obj`：作为模型 source 被扫描/import。
- `assets/materials/default.zmaterial`：作为 material asset 被扫描/import。
- `assets/shaders/pbr_shader`：compound shader package 及其 `.zmeta`/source 可解析。
- `.zircon` 派生目录：由 project owner 创建，不能作为缺失 source 的替代品。
- project/editor settings：加载结果必须标记真实 source/version；默认值只可填充未配置字段，不能把整份缺失/解析失败伪装为成功。

## 4. 非目标

- 不要求 default scene 已渲染 primitive；场景内容归 F2。
- 不验收 Recent Projects、Hub、模板选择 UI 或多个 project roots。
- 不要求 watcher/hot reload、远程资产或 package registry。
- 不把 cache/registry 文件存在等同于其 generation 与当前 manifest/source 一致。

## 5. M2.1 Template 与创建事务

### 目标

`ProjectAuthority::create_project` 以 staging + atomic publish 创建完整项目；任何失败不留下可被误打开的半项目。

### 实现切片

- [ ] 扩充 `template_creation.rs`，断言 template pack 复制所有 F1 必需输入、manifest project name 重写、default scene URI 保持 canonical。
- [ ] 断言创建目录必须原先不存在或为空；非空目标、unsafe project name、只读位置和目标冲突都在 publish 前失败。
- [ ] 让 rollback 删除本次 staging 产物但不删除调用者原有目录/文件。
- [ ] 创建成功立即通过 `ProjectManager::open` 建立第一代 project generation；禁止再从 disk 重开第二份 manifest authority。
- [ ] 记录 settings load result 的 source/version/diagnostics，使产品测试可以区分真实加载和 fallback-only。

### 测试阶段：F1 Template Transaction Gate

- [ ] 运行 Editor project template focused tests，包括复制、manifest rewrite、unsafe path、read-only probe、冲突 rollback。
- [ ] 运行 Runtime manifest/ProjectManager open focused tests，覆盖 future version、unsafe roots 和 default scene URI。
- [ ] 在 Windows 普通路径和包含空格/非 ASCII 的项目父目录各创建一次项目；两者都不得改变 URI canonicalization。
- [ ] 失败时从 project authority/manifest 最低层修复，不在 App parser 添加路径特例。

### 退出证据

- [ ] 成功创建产生完整 F1 输入；失败创建没有半项目。
- [ ] 第一代 project generation 的 manifest identity 与磁盘一致。
- [ ] settings load result 能区分 persisted source 与 fallback。

## 6. M2.2 Registry/import generation

### 目标

项目打开后完成一次有界 scan/import，发布与当前 project generation 对应的 registry；所有 F1 必需资产处于明确 Ready 或 typed Failed 状态，不允许静默缺失。

### 实现切片

- [ ] 让 project open 只注册 manifest 声明的 asset roots，并在同一 generation 中完成 source scan、import result 和 registry publish。
- [ ] 为 scene、OBJ、material、compound shader 建立精确 URI/kind/assertion；duplicate URI、unsafe root、corrupt sidecar 返回 typed diagnostic。
- [ ] cache restore 必须验证 source/artifact revision；stale cache 触发 reimport，不得把历史 Ready 当作当前 Ready。
- [ ] product diagnostics 暴露 project generation、scanned asset count、ready/failed count、settings source 和 default scene resolution。
- [ ] 添加回归测试：删除派生 registry/cache 后重开仍可从 source 重建；损坏 persisted index 后可诊断并重建。

### 测试阶段：F1 Asset Registry Gate

- [ ] 运行 Runtime project open、artifact cache import、registry persistence/query 和 compound shader focused suites。
- [ ] 从刚创建且无派生 cache 的 F1 项目完成首次 scan/import。
- [ ] 关闭 manager，删除验证副本中的派生 registry/cache，再次打开并比较 logical asset identity。
- [ ] 断言 scene/model/material/shader 均有唯一 canonical URI 和非 fallback load state。

### 退出证据

- [ ] 首次打开和无 cache 重开产生等价 registry identity。
- [ ] 所有必需资产均有明确状态，failed asset 具有可操作诊断。
- [ ] default scene reference 在 registry 中唯一解析。

## 7. M2.3 Editor 产品创建与重开

### 目标

staged editor 通过固定 CLI 路径创建项目，关闭后通过 `--project` 打开同一目录并显示 project-open 状态。

### 实现切片

- [ ] 保持 GUI startup parser 的 create/open 参数互斥、必填字段和 template 枚举测试。
- [ ] App entry 把 create/open request 交给同一 `ProjectAuthority`，不在 entry 层复制 template 或 manifest logic。
- [ ] editor startup snapshot/diagnostics 包含 canonical project root、manifest identity、project generation、registry counts 和 settings source。
- [ ] 首帧退出仅在 project open、registry/settings 初始投影和 window present 完成后触发。
- [ ] 对不存在项目、future manifest、缺失 default scene、registry import failure 提供非零退出或明确 degraded state；fallback-only 不得返回成功 F1。

### 测试阶段：F1 Product Create/Open Gate

- [ ] 从 F0 staging 执行固定 create-project 命令，等待首帧后干净退出。
- [ ] 检查持久目录和诊断，再用固定 `--project` 命令重开同一项目。
- [ ] 对比两次运行的 canonical root、manifest identity、default scene URI 和 registry logical entries。
- [ ] 将派生 cache 清空后第三次重开，确认成功来自 source 而非 fallback-only cache。
- [ ] 运行结束确认项目目录可重命名，证明 editor/runtime 没有残留句柄。

### 退出证据

- [ ] 创建、首次打开、无 cache 重开均由 current staged editor 完成。
- [ ] registry/settings 诊断明确来自 persisted project source。
- [ ] 同一项目根已冻结为 F2-F5 的 canonical fixture。

## 8. F1 阶段退出清单

- [ ] M2.1、M2.2、M2.3 全部通过。
- [ ] 项目由产品入口创建，未由测试 helper 预造后冒充产品创建。
- [ ] source rebuild 和 persisted reopen 都能解析相同 logical identity。
- [ ] 不存在 fallback-only 成功、静默丢失资产或半项目。
- [ ] canonical F1 项目根、manifest hash 和 registry summary 已交给 F2 validation lane。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|
