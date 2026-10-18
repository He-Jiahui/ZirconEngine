"""Execute the std-only production HighlightSet/store Rust tests in a managed copy.

This lower regression does not validate the session ABI or renderer frame bridge.
It includes the production modules verbatim and uses their existing Rust tests.
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
RUSTC_SHA256 = "21256c9767416cbc70120e7987449c6cc5a66e3e9f843d05392ee4fd5e617261"
SOURCE_HASHES = {
    "zircon_runtime/src/core/framework/render/highlight_set.rs": "aeaa4e95a3d953409f92bbd3e3a3d2a3e9f21e7e4442de52c7eeb7f7b4367c04",
    "zircon_runtime/src/core/framework/render/viewport_highlight_store.rs": "2d480867e1f6d66528f514034abaf60e993686bd4897a5c33dc145ad7c742128",
    "zircon_runtime/src/core/framework/render/viewport_highlight_store/tests/hash_index_tests.rs": "72437af38041d443d60971c9eba07b5ddbfe2447e275cd8b2d34e948f0d813ef",
    "zircon_runtime/src/core/framework/scene/mod.rs": "1910821f2522822803a75637add4ad768344265dbf986386920c5dbc7457111e",
}
BASE_GIT_BLOB_HASHES = {
    "zircon_runtime/src/core/framework/render/highlight_set.rs": "de48bb52b03d6428d714721710818f64374e57ca9f7c6d04f950f7ed60cf9556",
    "zircon_runtime/src/core/framework/render/viewport_highlight_store/tests/hash_index_tests.rs": "53ecb285b7789efa5b7c4852eb2d510019bac03742b12fe3664cc6f2a2f2a7fe",
    "zircon_runtime/src/core/framework/scene/mod.rs": "b6a6987069f90233578d54190a5657b3e6e5c04c6c0806fe898dea7e2aa1c67c",
}


def main() -> int:
    for relative, expected in SOURCE_HASHES.items():
        data = (ROOT / relative).read_bytes()
        actual = hashlib.sha256(data).hexdigest()
        if actual != expected:
            raise RuntimeError(f"frozen source mismatch: {relative}: {actual}")
        if relative in BASE_GIT_BLOB_HASHES:
            blob = hashlib.sha256(data.replace(b"\r\n", b"\n")).hexdigest()
            if blob != BASE_GIT_BLOB_HASHES[relative]:
                raise RuntimeError(f"frozen Git blob mismatch: {relative}: {blob}")
    target_value = os.environ.get("CARGO_TARGET_DIR")
    if not target_value:
        raise RuntimeError("run through the coordinator with its managed target")
    target = Path(target_value).resolve()
    if target.drive.casefold() not in {"d:", "e:", "f:"}:
        raise RuntimeError(f"managed target is outside approved storage: {target}")
    output = target / "runtime10-highlight-stdlib"
    output.mkdir(parents=True, exist_ok=True)
    rustc = Path(subprocess.check_output(
        ["rustup", "which", "--toolchain", "1.94.1", "rustc"], text=True,
    ).strip())
    actual_rustc = hashlib.sha256(rustc.read_bytes()).hexdigest()
    if actual_rustc != RUSTC_SHA256:
        raise RuntimeError(f"rustc binary identity mismatch: {actual_rustc}")
    version = subprocess.check_output([str(rustc), "-vV"], text=True)
    if "release: 1.94.1\n" not in version or "host: x86_64-pc-windows-msvc\n" not in version:
        raise RuntimeError(f"rustc version identity mismatch: {version}")
    alias_source = (ROOT / "zircon_runtime/src/core/framework/scene/mod.rs").read_text(encoding="utf-8")
    aliases = re.findall(r"^pub type EntityId = u64;$", alias_source, re.MULTILINE)
    if len(aliases) != 1:
        raise RuntimeError("production EntityId alias is not unique")
    render_root = ROOT / "zircon_runtime/src/core/framework/render"
    highlight = json.dumps((render_root / "highlight_set.rs").as_posix())
    store = json.dumps((render_root / "viewport_highlight_store.rs").as_posix())
    harness = output / "highlight_store_tests.rs"
    harness.write_text(
        "pub mod core { pub mod framework {\n"
        f"pub mod scene {{ {aliases[0]} }}\n"
        "pub mod render {\n"
        f"#[path = {highlight}] mod highlight_set;\n"
        "pub use highlight_set::*;\n"
        f"#[path = {store}] mod viewport_highlight_store;\n"
        "} } }\n", encoding="utf-8",
    )
    executable = output / "highlight_store_tests.exe"
    print(json.dumps({"rustcVersion": version, "rustcSha256": actual_rustc,
                      "sourceHashes": SOURCE_HASHES, "baseGitBlobHashes": BASE_GIT_BLOB_HASHES,
                      "managedTarget": str(output)}), flush=True)
    compiled = subprocess.run([
        str(rustc), "--edition=2021", "--test", str(harness), "-o", str(executable),
    ])
    if compiled.returncode:
        return compiled.returncode
    tested = subprocess.run([str(executable), "--test-threads=1", "--nocapture"],
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    print(tested.stdout, end="", flush=True)
    if tested.returncode:
        return tested.returncode
    if "test result: ok. 8 passed; 0 failed; 2 ignored;" not in tested.stdout:
        raise RuntimeError("expected production Rust behavior tests did not all execute")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
