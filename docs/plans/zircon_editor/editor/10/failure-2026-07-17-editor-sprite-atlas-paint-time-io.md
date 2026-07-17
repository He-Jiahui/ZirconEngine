---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: editor-sprite-atlas-paint-time-io
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets
  - zircon_runtime/src/asset
---

# Editor10 failure handoff: sprite atlas paint-time I/O

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 sprite atlas resolver/cache/tests 9文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 共同责任：`docs/plans/zircon_runtime/render/13-texture-pipeline.md`、`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 交接原因：Editor10拥有project library、asset identity和file-watcher generation，必须在资源变化时建立atlas index；paint consumer与Render13不能用逐draw stat替代资源权威。

## 失败现象与复现证据

PERF-MVP-180静态审查确认sprite atlas resolve在paint/image consumer中逐请求扫描项目`.zircon/cache/editor-sprite-atlases`目录，canonicalize/stat每个manifest，cache hit仍深clone完整manifest，entry命中后再次打开并解码整张atlas。Manifest cache以path+mtime+length为key且无淘汰；首个manifest缺entry还会提前终止后续candidate。

PERF-MVP-181的41文件审查确认普通visual assets也在cache hit前重建alias/source/MUI candidate并逐path `exists()`；SVG tree hit前stat source，retained preview复制/hash整图，MUI fallback从`dev/material-ui`读取JS。Editor10的asset registry必须覆盖普通icon/image/MUI导入与canonical resource generation，而不只处理atlas manifest。

## 最低共享层根因

Atlas discovery/index没有进入asset-generation owner，manifest metadata被当作每次paint lookup的失效协议；decoded image和draw payload也没有稳定resource handle，导致Editor asset、paint与Render13三个生命周期混在同步consumer中。

## 架构修复验收

Project library/file watcher随asset generation一次构建immutable`source key -> atlas handle + UV` index，manifest或texture变化精确失效，删除/坏文件回收旧entry。Paint不能以filesystem stat维持新鲜度。Decoded CPU/GPU texture生命周期交给Render13，Editor10只拥有asset identity、discovery和change generation，不另存第二份像素权威。

- Stable 1/100/10,000 atlas icons的paint-thread read_dir/canonicalize/metadata/read/parse/decode/manifest clone均为0。
- 首次/changed generation每manifest parse≤1；multi-manifest later-candidate可命中，删除/坏manifest/hot reload精确更新。
- Index/cache entries与metadata bytes有硬上限，旧mtime generation可回收。
- 与Render13对拍同atlas generation decode/upload≤1，resource key、UV、dimensions与GPU/Softbuffer pixels一致。
- Stable普通visual asset lookup的candidate/path alloc、exists/canonicalize/stat、dev-source read/parse为0；production不依赖`dev/material-ui` runtime fallback，resource change由file watcher generation发布。

## 禁止临时方案

- 不得只在paint函数外再包一层无界全局PathBuf cache。
- 不得以每frame/每draw metadata轮询代替file watcher generation。
- 不得让Editor10与Render13各持一份decoded atlas像素权威。

## 修复结果与回传

完成后在本目录写`fixed-*`或return记录并附current-source Windows Cargo、focused tests和规模counter；此前保持open。
