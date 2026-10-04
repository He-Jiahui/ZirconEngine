"""Run one runtime descendant inside the atomically assigned keeper Job."""
import os
import sys
from pathlib import Path
import json
import subprocess
from contextlib import ExitStack


def _validated_runtime_assets(binding):
    if binding is None:
        return None
    from tools.jenkins.pilot.contracts import PilotError, sha256_identity
    from tools.jenkins.pilot.storage import require_managed_root
    if not isinstance(binding, dict) or binding.get("kind") not in {"controller", "agent"}:
        raise PilotError("runtime asset launch binding is invalid")
    fields = {"root", "kind", "proofSha256"}
    if binding["kind"] == "agent":
        fields.add("agentJarSha256")
    if set(binding) != fields:
        raise PilotError("runtime asset launch fields do not match the fixed binding")
    root = require_managed_root(Path(binding["root"]), allow_root=False)
    sha256_identity(binding["proofSha256"], "runtime asset proof sha256")
    if binding["kind"] == "agent":
        sha256_identity(binding["agentJarSha256"], "runtime agent JAR sha256")
    return dict(binding, root=str(root))


def _run_child(args, runtime_assets=None, *, start_file=None):
    """Keep verified executable handles from first Popen through actual exit."""
    binding = _validated_runtime_assets(runtime_assets)
    with ExitStack() as pins:
        if binding is not None:
            from tools.jenkins.pilot.assets import prepared_assets_context
            from tools.jenkins.pilot.storage import ManagedStorage
            root = Path(binding["root"])
            pins.enter_context(prepared_assets_context(root, expected_digest=binding["proofSha256"], retain_manifest=False))
            if binding["kind"] == "agent":
                backend = pins.enter_context(ManagedStorage(root).backend())
                pins.enter_context(backend._windows.open_relative("war/agent.jar", "rb"))
                backend.read_bytes("war/agent.jar", expected_sha256=binding["agentJarSha256"], max_bytes=32 * 1024 * 1024)
        process = subprocess.Popen(args, stdin=subprocess.DEVNULL, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            if start_file is not None:
                if binding is None:
                    raise ValueError("Java start marker requires fixed runtime assets")
                from tools.jenkins.pilot.storage import ManagedStorage, require_managed_root
                from tools.jenkins.pilot.contracts import canonical_json
                from tools.jenkins.pilot.native.process_identity import process_creation_time, popen_process_creation_time
                path = require_managed_root(Path(start_file), allow_root=False)
                marker = {"schemaVersion": 1, "hostPid": os.getpid(), "hostCreationTime": process_creation_time(os.getpid()),
                          "childPid": process.pid, "childCreationTime": popen_process_creation_time(process), "runtimeAssets": binding}
                storage = ManagedStorage(path.parent)
                with storage.backend() as backend:
                    if backend.exists(path.name):
                        raise ValueError("immutable Java start marker already exists")
                storage.atomic_write(path.name, canonical_json(marker))
            return process.wait()
        except BaseException:
            if process.poll() is None:
                process.kill()
            process.wait()
            raise


def main():
    # Redirect before importing the launcher: the short-lived submitting CLI
    # closes its inherited pipe readers after transferring the Job handle.
    from tools.jenkins.pilot.storage import ManagedStorage, require_managed_root
    path = require_managed_root(Path(os.environ.pop("JENKINS_PILOT_HOST_LOG")), allow_root=False)
    storage = ManagedStorage(path.parent)
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-assets")
    parser.add_argument("--runtime-start-file")
    options = parser.parse_args()
    runtime_assets = json.loads(options.runtime_assets) if options.runtime_assets is not None else None
    args = json.loads(os.environ.pop("JENKINS_PILOT_HOST_ARGS"))
    with storage.backend() as backend, backend.open_lock_file(path.name) as output:
        output.seek(0, 2)
        os.dup2(output.fileno(), 1)
        os.dup2(output.fileno(), 2)
        return _run_child(args, runtime_assets, start_file=options.runtime_start_file)


if __name__ == "__main__":
    sys.exit(main())
