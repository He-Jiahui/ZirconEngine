---
related_code:
  - zircon_runtime/src/ui/tests/asset_package_validation.rs
  - zircon_runtime/src/ui/tests/asset_prototype_store.rs
  - zircon_runtime/src/ui/template/asset/compiler/package
  - zircon_runtime/src/ui/template/asset/prototype_file_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - nine package and thirteen prototype-store semantic tests reviewed
  - one two-condition source-level RED to GREEN file-cache guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, package/prototype scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset package/prototype store测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_package_validation.rs` 558行/9测试与`asset_prototype_store.rs`原始728行/13测试，共2/2个tracked Rust文件；prototype测试加入1项双条件源码性能守卫后为738行/14测试。范围覆盖package header/dependency/binary/manifest/profile/error precedence，以及flat prototype/store/file cache/transitive imports/10k deep chain/surface projection。

## PERF-MVP-308/310/311：package小图与多次artifact序列化

package fixture固定1 root、2 widgets、1 style、3 resources；测试主动对同一artifact重复`to_bytes`、decode TOML、manifest write/import及before/validate/after cache key，锁定determinism但没有serialized bytes、tree/import visits、clone bytes或RSS预算。产品package仍需复用single parse/key/index与generation artifact，归PERF-MVP-308/310/311和EditorUI05。

## PERF-MVP-306/309：prototype file-cache局部复制

原transitive source扫描用`Vec + index`，每轮从queue clone `PathBuf`；import枚举又把widgets/styles全部复制为`Vec<String>`。新增守卫先确认RED，再改为`VecDeque::pop_front()`移动source path，并返回借用`&str`的chain iterator，守卫转GREEN。第54/55组局部优化保持FIFO、alias、fragment stripping和dedup顺序。file cache hit仍需对explicit+transitive sources执行canonicalize/metadata，prototype instancing环境/tree所有权仍归PERF-MVP-306/309。

10k深链测试只证明显式work stack并用`mem::forget`避开递归drop，不记录frame/node clone、token/param maps、compiled tree bytes、CPU或RSS；不能作为规模验收。

## 验收要求

对1/100/10k nodes/imports/instances/files记录path/reference clone bytes、canonicalize/metadata/read calls、parse/index builds、prototype frame bytes、artifact serialization bytes、CPU/RSS和compile/load p95。cache hit文件读取=0；source queue额外PathBuf clone=0；import reference String allocation=0；10k chain不泄漏且drop stack-safe。当前源码package 9项/prototype 14项Cargo、规模counter与F4 asset load/preview trace完成前，两文件留在`pending.md`。
