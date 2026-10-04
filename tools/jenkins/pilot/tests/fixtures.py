"""All pilot fixtures stay below the approved physical storage root."""
from pathlib import Path
import hashlib
import io
import tempfile
import zipfile
import os

from tools.jenkins.pilot.contracts import canonical_json, digest
from tools.jenkins.pilot.storage import DEFAULT_PILOT_ROOT, ManagedStorage


def temporary_directory(prefix="fixture-"):
    storage = ManagedStorage(Path(os.environ.get("JENKINS_PILOT_ROOT", str(DEFAULT_PILOT_ROOT))))
    storage.ensure_layout()
    return tempfile.TemporaryDirectory(prefix=prefix, dir=storage.tmp_dir)


def sealed_bundle(root: Path):
    data = b"value = 1\n"
    value_hash = hashlib.sha256(data).hexdigest()
    manifest = {"schemaVersion": 1, "baseCommit": "a" * 40, "paths": ["fixture.py"], "entries": [{
        "path": "fixture.py", "status": "new", "mode": "100644", "baseBlob": None,
        "size": len(data), "sha256": value_hash, "payload": value_hash,
    }]}
    manifest["inputHash"] = digest(manifest)
    output = io.BytesIO()
    with zipfile.ZipFile(output, "w") as archive:
        archive.writestr("manifest.json", canonical_json(manifest))
        archive.writestr(f"payloads/{value_hash}", data)
    path = root / "fixture.zip"
    path.write_bytes(output.getvalue())
    return path, manifest["inputHash"]
