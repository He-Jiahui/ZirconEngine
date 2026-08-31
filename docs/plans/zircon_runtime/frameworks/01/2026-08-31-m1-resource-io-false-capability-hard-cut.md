# M1 Resource I/O False-Capability Hard Cut

Status: `source_complete / static_green / managed_compile_green / milestone_not_accepted`

## Scope

This record covers only the dead public `ResourceIo` and `ResourceIoError` declarations that were
moved into `zr_resource` during the Resource crate hard cut. It does not implement the later
filesystem/source/mount architecture, does not change `AssetIoDriver`, and does not change the
working single-file or durable transaction writers.

The decision supersedes the earlier M1 statement that Runtime should retain the Resource I/O
contract. Current-source evidence proves that statement preserved a name rather than a capability:

- the trait was sealed by a private supertrait, so no Runtime, Asset, plugin, or product crate could
  implement it;
- the whole Runtime/Interface/Editor/Plugins Rust union contained no implementation and no call;
- `ResourceIoError` existed only to type the dead trait and stored unstructured `String` values;
- Asset source, artifact, project, pack, and importer owners still perform real filesystem work
  directly, while `zr_resource::io::{atomic_write, atomic_write_new}` and the private durable
  transaction assembly are the actual I/O infrastructure used today.

Keeping the declarations would violate the no-placeholder rule and would let a public type be
mistaken for an implemented MVP service. The future filesystem provider is therefore a new hard-cut
contract owned by the filesystem/asset-source milestone. It must have a real local provider,
source/mount lifecycle, typed operation errors, queue/cancellation/shutdown, and actual Asset
consumers before publication. It will not revive this three-method synchronous trait or add a
compatibility re-export.

## Reference review

Unreal is the primary reference. Its physical and wrapped file access is a real `IPlatformFile`
implementation chain selected by `FPlatformFileManager`; `FileExists`, `OpenRead`, `OpenWrite`,
directory operations, lower-level composition, and initialization are executable contracts rather
than reserved product types.

- `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformFile.h`
  SHA-256 `4e46f6dd2cc9caa5ac6440304bf5ace3b585ba7b77788f63c409d6e20a49d742`;
- `dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/PlatformFileManager.h`
  SHA-256 `3f93d999ee6d7674dfeaa7c4ce24b9e90f5ec299755857f131bfc1694dbefb53`.

Fyrox is the secondary resource-loading reference. Its async `ResourceIo` is constructible,
implemented by `FsResourceIo`, injected into `ResourceManager`, and passed into concrete loaders.
That evidence supports provider injection, but Zircon does not copy Fyrox's weak `exists -> bool`
or default empty directory behavior.

- `dev/Fyrox/fyrox-resource/src/io.rs`
  SHA-256 `49101a9e3b52a0166e1889157d94b96b0296444742e8604ac814dcd44a8e828e`;
- `dev/Fyrox/fyrox-resource/src/manager.rs`
  SHA-256 `d3a61d933c13441c3793f91cbdaf0314cdd527fa2074c43341a1864c21d4b822`.

This conclusion also agrees with Runtime25/Runtime160 finding `FILESYSTEM-P1-011`: remove the false
surface and require the replacement provider to be consumed by the asset runtime before claiming
the capability.

## TDD and result

The boundary test was added first. RED failed because
`zircon_runtime/crates/zr_resource/src/io/resource_io.rs` still existed. Production then hard-cut:

- deleted `zr_resource/src/io/resource_io.rs` and `zr_resource/src/io/error.rs`;
- removed their module declarations and exports from `zr_resource::io` and the crate root;
- removed both Runtime `core::resource` projections;
- retained the public `atomic_write`/`atomic_write_new` pair and private durable transaction
  assembly unchanged.

GREEN evidence on the resulting source:

- focused boundary test: 1/1 passed;
- complete `test_frameworks_01_resource_crate_boundary` suite: 8/8 passed in 17.106 seconds;
- current Runtime/Interface/Editor/Plugins Rust scan: 0 `ResourceIo` or `ResourceIoError` matches;
- scoped `git diff --check`: clean, apart from existing line-ending conversion warnings.

Final source fingerprints:

- `tools/tests/test_frameworks_01_resource_crate_boundary.py`
  `87c1cf030ad225f57e1fb57c914de4e28a1abe7ebdbdd6f5773b1dfe28ba2330`;
- `zircon_runtime/crates/zr_resource/src/io/mod.rs`
  `afc86629b54d155f84700f2f28ea26b2ec269f5a25331c19491e4ccd90823ef4`;
- `zircon_runtime/crates/zr_resource/src/lib.rs`
  `7dfdca54d631d223d4384e414111c256518a8edd099425300845e6a77d924dc7`;
- `zircon_runtime/src/core/resource/io/mod.rs`
  `1a376207a8cb0556eaf5b97985551a990365c13ce090ab3d2bc1f395e677c4ed`;
- `zircon_runtime/src/core/resource/mod.rs`
  `7a6002fe9b3599d1c3556277a53324ba6a7ebab58b80dc1651e9509b013dcfff`.

This is an API-truth and structure correction, not a performance optimization. Managed R9 job
`f2f3280096d64ca699bdd9c9e4800e97` subsequently compiled the resulting `zr_resource` test crate and
ran the current profile successfully, providing a compile GREEN for this deleted dead surface. It
does not replace the full package behavior suite and makes no ResourceIo latency, RSS, power, parity,
or optimal-scale claim. M1 is not accepted, so this slice is not a milestone commit and is not
eligible for WeCom notification.
