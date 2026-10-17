# Runtime160 strict asset-relative path source record

Date: 2026-10-16. FILESYSTEM-P1-021. Source implementation and scoped static byte proof complete; native, guard and performance acceptance OPEN.

Actual source executor: editor06_owner_core_peer20 / gpt-6.1-sol max. Packet/design owner: runtime160_path_core22. Root granted these 21 source paths and four post outputs after the revised notice writer exited 0.

Both runtime asset resolver APIs now require borrowed AssetRelativePath. The parser uses original UTF-8 spelling and rejects roots, drive/colon forms, backslashes, dot/parent components, empty segments, controls and non-UTF8. root() is the deliberate root-query entry. Explicit from_assets_prefixed strips one rootless assets/ prefix then validates. The old implicit normalizer was removed.

Actual font and editor callers use the typed API. Invalid references use existing InvalidAssetReference, UiAssetError::InvalidDocument, UiV2AssetError::InvalidDocument and preview error paths. Two named UI adapters map only exact /assets/ before strict suffix validation, preserving existing identity producers. Lexical admission does not close secure-open, symlink/hardlink identity, reserved-name, URI codec or other Runtime160 findings.

All 21 fresh before pins passed before any write, including absence of new relative.rs. Source writer 0684f2 actually exited 0 (PowerShell 30956 / Python 492, zero children). Every actual after was read twice, matched the fixed pin, and passed exact forward/reverse normal-hunk proof. Original LF, CRLF and mixed EOL bytes outside hunks were preserved.

HEAD stayed f28ba39a5095f4964c046ecc702778627f80cfd5. The index stayed 7478958 bytes / 8ae41eb940729cf4f1ea1468d72fdbe257581bc1fc38ace9dce024c481400893. No staging, commit, control write or root manifest/log update occurred. Concurrent E06 work has disjoint path ownership.

The authorized foreign support.rs before was 28377 bytes / 60c3a619521ec0523a87f2e90baef369d8fcc2dfec6ed07f39ceae7c6028c819. Every byte outside the caller hunk was preserved; external actor and semantic delta remain UNKNOWN.

Original 41 test identities (36 ordinary, 5 ignored) and attributes are preserved. Twelve new ordinary source names are present; Unix/Windows non-UTF8 cases are mutually exclusive, adding 11 per host. Tests were not executed. The static JSON contains the full old/new FQNs and cfg.

- `asset::runtime_asset_path::tests::strict_relative_path_preserves_valid_spelling`.
- `asset::runtime_asset_path::tests::strict_relative_path_rejects_root_prefix_and_traversal_components`.
- `asset::runtime_asset_path::tests::legacy_assets_prefix_is_explicit_and_stripped_only_once`.
- `asset::runtime_asset_path::tests::asset_root_query_requires_the_explicit_root_value`.
- `asset::runtime_asset_path::tests::strict_relative_path_rejects_non_utf8_spelling_on_unix` (`cfg(unix)`).
- `asset::runtime_asset_path::tests::strict_relative_path_rejects_non_utf8_spelling_on_windows` (`cfg(windows)`).
- `text::font::source_manifest::tests::res_font_manifest_rejects_invalid_paths_before_runtime_resolution`.
- `text::font::source_manifest::tests::res_font_manifest_does_not_implicitly_strip_an_assets_segment`.
- `ui::template_runtime::runtime::build_session::tests::builtin_import_rejects_invalid_relative_paths_before_filesystem_loading`.
- `ui::asset_editor::node_projection::tests::ui_source_adapter_preserves_bootstrap_namespace_and_rejects_invalid_suffixes`.
- `ui::layouts::views::view_projection::store_cache::tests::ui_source_paths_preserve_the_explicit_assets_namespace`.
- `ui::layouts::views::view_projection::store_cache::tests::invalid_ui_source_paths_are_not_published_to_the_cache_or_watcher`.

Unexecuted scoped recipes:

- `cargo test -p zircon_runtime --lib asset::runtime_asset_path::tests -- --nocapture` (not run).
- `cargo test -p zircon_runtime --lib text::font::source_manifest::tests -- --nocapture` (not run).
- `cargo test -p zircon_runtime --lib ui::tests::boundary::asset_fixture_projection -- --nocapture` (not run).
- `cargo test -p zircon_editor --lib ui::template_runtime::runtime::build_session::tests -- --nocapture` (not run).
- `cargo test -p zircon_editor --lib ui::asset_editor::node_projection::tests -- --nocapture` (not run).
- `cargo test -p zircon_editor --lib ui::layouts::views::view_projection::store_cache::tests -- --nocapture` (not run).

No Cargo, build, Rust/native tests, benchmark, Python unittest, change guard, CI execution, or Jenkins status query ran. Neither workspace-wide nor narrowed package compilation ran. Source/static evidence does not establish a native floor or acceptance. Five existing ignored performance identities remain unchanged. The original M6 provider comparison is not yet due; overall performance acceptance remains OPEN.

Evidence files under .codex/outbox/reports/optimize-20261013-01a13451: runtime160-strict-asset-relative-path-v1-implementation-receipt.json and runtime160-strict-asset-relative-path-v1-static.json. Fixed patch: 38410 bytes / 89c968a1b2b5b675de0125ad8ac0cdfd4e9dfe30976e21a5a70696423061ff4b. Fixed binding: 22501 bytes / f047db1bc0f9fa0e786df58c9bdad950945da7772104d04c7c854fa4f5a2ad9b.

Reader 854c97 exited 1 (PowerShell 12132 / Python 29212): incorrect reporting guide path, zero writes. Corrected reader a3b2dd exited 0 (PowerShell 29444 / Python 484). A JavaScript parse error before the postwriter started created no process or file. Original packet failures and the spawn rejection with no PID/session/terminal remain preserved, including UNKNOWN.

Released space: 0 bytes. No helper, source copy, temporary/build/cache artifact was created. Keep fixed packets and four delivery outputs for later coalesced Jenkins handoff. Foreign/shared data is untouched.

| Source path | Actual after bytes | Actual raw SHA-256 |
| --- | ---: | --- |
| `zircon_runtime/src/asset/runtime_asset_path.rs` | 9173 | `83c3e8166123a0417857bc082e435cffd7ead7fade84108a518529da1a59fc44` |
| `zircon_runtime/src/asset/runtime_asset_path/relative.rs` | 3181 | `89996f2029e6c00267482c63b78e48fd66b92b5a242ed7415d9eb1c3516079ad` |
| `zircon_runtime/src/asset/mod.rs` | 12608 | `0144db33534284925e2a7a315e3fc3eebb472614fd6f478729e7ca1f1a45c75b` |
| `zircon_runtime/src/asset/prelude.rs` | 981 | `dba26e76cd5462fe7fb07f534d386c13081c45e90f44c9df17a8e74e0042a949` |
| `zircon_runtime/src/asset/tests/runtime_asset_path.rs` | 19362 | `34e9239fbf6aba4fd47a33745c07875415ea526d6b88fb64079de65ebf41d4d2` |
| `zircon_runtime/src/text/font/source_manifest.rs` | 8440 | `b21ba82830e6bec55174247583b61c614fd1914eb53ee23dae879c9958967cdd` |
| `zircon_runtime/src/text/font/source_manifest/tests/cases.rs` | 13372 | `e535617fa62bd9931029d07563acf32a5b42f1d52f55a10ac06e94e7fb651d33` |
| `zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_fixture.rs` | 1773 | `9618ef30ed13ad622b8e108dc582b92700f6e19aaea4b262f6fd0c1f9d365226` |
| `zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs` | 5895 | `d8a0a05d290b8d2d2c7724a996ac22ddce1f4c2ae43ebfc712a5bde5fc0e53cd` |
| `zircon_editor/src/ui/runtime/builtin/documents.rs` | 7720 | `86d3d06e11bc96d8652a876c5b0314c245f61d04473850997b11c267f72a8566` |
| `zircon_editor/src/ui/runtime/runtime/build_session.rs` | 12366 | `e5d04ea7d43123a2a5f24de8d2156e8a88218c260ead04d61cff5dd1fbb65d9c` |
| `zircon_editor/src/ui/runtime/runtime/tests/build_session.rs` | 9966 | `d50fb5fd85b98486b27f9c598225a51c0a27fef55e32a60f1d0ae15c145e19a7` |
| `zircon_editor/src/ui/runtime/runtime/runtime_host/module_workspace.rs` | 4786 | `f8c0c8bf8beb5bc583c323a9e621bab074d72efe0ef9bd297e963bcc3810f839` |
| `zircon_editor/src/ui/runtime/runtime/runtime_host/generated_bottom_pane.rs` | 31303 | `ea4888d4b4bf7dd3b4636103f01146801cafed16bd57f6f3189753301ff0fab6` |
| `zircon_editor/src/ui/asset_editor/preview/preview_host.rs` | 20372 | `5617c35a2d7679737931546f4ffe22d9609510bf9b9c3e06e30090963d3f6b18` |
| `zircon_editor/src/ui/asset_editor/node_projection.rs` | 25190 | `6b26bd658cd9c69c8dcf861a21bdb0791c347cebe3347f5ad02db4854d3d89f9` |
| `zircon_editor/src/ui/asset_editor/tests/node_projection.rs` | 1180 | `49ced3507a226cf7cdea9ce922a7515bc234a4796544af586b20e2d53b61e76e` |
| `zircon_editor/src/ui/layouts/views/view_projection/store_cache.rs` | 7679 | `b8741f666b72c5e9129a1f6e30376525af837d53b792cbf3691dce36af4e726f` |
| `zircon_editor/src/ui/layouts/views/view_projection/tests/store_cache.rs` | 7070 | `7bbb9df6177da38f0f02e05e5040c67116f2a3b62d22044e657faa0f8a8e09ea` |
| `zircon_editor/src/ui/retained/contract/paint/visual_assets/candidates/paths.rs` | 1672 | `679db4d5f14343b49a8fed3885e24eafaef6d0e7bf691d6adfed64e4bd0fba4e` |
| `zircon_editor/src/ui/retained/app/tests/module_tools_menus/support.rs` | 28487 | `0550d5cc8f9feb8ef45fbf5bd83952186e4dcac2b96e029c4dd9ebc1209f4141` |
