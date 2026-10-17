"""Run the ordinary management regression file under Jenkins-owned native evidence."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import stat
import sys


REPOSITORY = Path(r"E:\Git\ZirconEngine")
OUTPUT_BASE = Path(
    r"E:\cargo-targets\zircon-local\reports\jenkins-parallel-activation-20261011"
    r"\ordinary-management-regression"
)
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,79}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
SOURCE_NAMES = (
    "tools/jenkins-management/runner.py",
    "tools/jenkins-management/source_preparation.py",
    "tools/jenkins-management/tests/test_ordinary_flow.py",
    "tools/dev/local_cargo.py",
    "tools/dev/local-cargo.ps1",
    "tools/dev/dev-fast-build.ps1",
    "tools/dev/submit-jenkins-management.ps1",
    "tools/jenkins/frontend.py",
    "tools/jenkins-management/entry.py",
    "tools/jenkins-management/run_checks.py",
)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def encoded(value: object) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def physical(path: Path, *, must_exist: bool = True) -> Path:
    path = Path(os.path.abspath(path))
    for item in (path, *path.parents):
        try:
            info = item.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
            raise ValueError("physical_path_required")
    if must_exist and not path.exists():
        raise ValueError("required_path_missing")
    return path


def stable_hash(path: Path) -> str:
    path = physical(path)
    if not path.is_file():
        raise ValueError("required_file_missing")
    before = path.stat()
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            checksum.update(chunk)
    after = path.stat()
    fields = ("st_dev", "st_ino", "st_size", "st_mtime_ns", "st_ctime_ns")
    if any(getattr(before, field) != getattr(after, field) for field in fields):
        raise ValueError("source_changed_during_hash")
    return checksum.hexdigest()


def read_json(path: Path) -> dict:
    value = json.loads(physical(path).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("invalid_json_record")
    return value


def write_json(path: Path, value: object) -> None:
    data = encoded(value) + b"\n"
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def inventory_tree(root: Path, output: Path) -> list[dict]:
    root = physical(root)
    if root.parent != output or root.name not in {"basetemp", "temp", "cache", "fixtures"}:
        raise ValueError("cleanup_root_not_registered")
    result: list[dict] = []
    pending = [root]
    while pending:
        directory = pending.pop()
        for entry in os.scandir(directory):
            path = physical(Path(entry.path))
            if not path.is_relative_to(root):
                raise ValueError("cleanup_entry_escaped_root")
            info = path.lstat()
            if stat.S_ISDIR(info.st_mode):
                result.append({"path": str(path.relative_to(output)), "kind": "directory"})
                pending.append(path)
            elif stat.S_ISREG(info.st_mode):
                result.append({
                    "path": str(path.relative_to(output)), "kind": "file",
                    "sha256": stable_hash(path), "bytes": info.st_size,
                    "identity": [info.st_dev, info.st_ino, info.st_mtime_ns, info.st_ctime_ns],
                })
            else:
                raise ValueError("cleanup_entry_type_untrusted")
    result.sort(key=lambda item: item["path"])
    return result


def remove_registered_trees(output: Path, names: tuple[str, ...], manifest_bytes: bytes) -> dict:
    manifest_path = physical(output / "artifacts.json")
    if manifest_path.read_bytes() != manifest_bytes:
        raise ValueError("cleanup_manifest_changed")
    manifest = json.loads(manifest_bytes.decode("utf-8"))
    roots = [output / name for name in names]
    if manifest.get("outputDirectories") != [str(root) for root in roots]:
        raise ValueError("cleanup_roots_not_manifested")
    inventory = [inventory_tree(root, output) if root.exists() else [] for root in roots]
    inventory_digest = digest(encoded(inventory))
    write_json(output / "cleanup-plan.json", {
        "schemaVersion": 1, "status": "prepared", "manifestSha256": digest(manifest_bytes),
        "inventorySha256": inventory_digest, "directories": [str(root) for root in roots],
        "entries": inventory,
    })

    files = {entry["path"]: entry for group in inventory for entry in group if entry["kind"] == "file"}
    directories = [entry["path"] for group in inventory for entry in group if entry["kind"] == "directory"]
    removed_bytes = 0
    for relative, entry in sorted(files.items(), key=lambda item: len(item[0]), reverse=True):
        path = physical(output / relative)
        info = path.stat()
        identity = [info.st_dev, info.st_ino, info.st_mtime_ns, info.st_ctime_ns]
        if (not path.is_file() or info.st_size != entry["bytes"]
                or identity != entry["identity"] or stable_hash(path) != entry["sha256"]):
            raise ValueError("cleanup_file_changed_after_inventory")
        # Git creates read-only object files in these task-owned fixtures.
        # Clear only that DOS attribute after the exact plan and byte checks;
        # ACLs and every path outside the registered fixture roots are untouched.
        if os.name == "nt" and getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_READONLY:
            os.chmod(path, stat.S_IWRITE)
            after = physical(path).stat()
            if (not path.is_file() or after.st_size != entry["bytes"]
                    or [after.st_dev, after.st_ino, after.st_mtime_ns] != entry["identity"][:3]
                    or stable_hash(path) != entry["sha256"]):
                raise ValueError("cleanup_file_changed_after_readonly_clear")
        path.unlink()
        removed_bytes += entry["bytes"]
    for relative in sorted(directories, key=len, reverse=True):
        physical(output / relative).rmdir()
    for root in roots:
        if root.exists():
            physical(root).rmdir()
    receipt = {
        "schemaVersion": 1, "status": "cleaned", "manifestSha256": digest(manifest_bytes),
        "inventorySha256": inventory_digest, "removedDirectories": [str(root) for root in roots],
        "removedFileCount": len(files), "removedBytes": removed_bytes,
        "retainedEvidence": ["artifacts.json", "stdout.bin", "stderr.bin", "started.json",
                             "native-terminal.json", "source-hashes.json", "pytest.ini",
                             "pytest-junit.xml", "summary.json"],
    }
    write_json(output / "cleanup.json", receipt)
    return receipt


def verify_pins(repo: Path, expected_driver: str, expected_runtime: str,
                source_hashes: dict[str, str]) -> str:
    selected = read_json(repo / ".jenkins/state/deployment/driver.json")
    operation = read_json(repo / ".jenkins/state/deployment-operation.json")
    if (selected.get("driverDigest") != expected_driver
            or selected.get("runtimeOperationId") != expected_runtime
            or Path(selected.get("root", "")).absolute()
            != repo / ".jenkins/runtime/drivers" / expected_driver
            or operation.get("operationId") != expected_runtime
            or operation.get("state") != "running"):
        raise ValueError("selected_runtime_pin_mismatch")

    # Verify the selected sealed driver with the project's existing verifier.
    driver_root = repo / ".jenkins/runtime/drivers" / expected_driver
    sys.path.insert(0, str(driver_root))
    from tools.jenkins.deployment.driver import verify_driver

    verify_driver(driver_root, expected_digest=expected_driver)

    installed = read_json(repo / ".jenkins/state/management/installed.json")
    capsule_digest = installed.get("capsuleSha256")
    if not isinstance(capsule_digest, str) or not SHA256.fullmatch(capsule_digest):
        raise ValueError("management_capsule_pin_invalid")
    capsule = physical(repo / ".jenkins/runtime/management" / capsule_digest)
    declared_capsule = physical(Path(installed.get("capsule", "")))
    capsule_file = capsule / "capsule.json"
    capsule_bytes = physical(capsule_file).read_bytes()
    if (declared_capsule != capsule or capsule.name != capsule_digest
            or digest(capsule_bytes) != capsule_digest):
        raise ValueError("management_capsule_identity_mismatch")
    config = json.loads(capsule_bytes.decode("utf-8"))
    if (not isinstance(config, dict) or config.get("schemaVersion") != 1
            or config.get("repositoryRoot", "").casefold() != str(repo).casefold()
            or config.get("agentLabel") != "zircon-windows-agent"
            or config.get("driverDigest") != expected_driver
            or config.get("runtimeOperationId") != expected_runtime
            or installed.get("driverDigest") != expected_driver
            or installed.get("runtimeOperationId") != expected_runtime):
        raise ValueError("management_capsule_runtime_mismatch")

    files = config.get("files")
    allowed_capsule_files = {"runner.py", "source_preparation.py", "zircon-management.groovy", "checkout_guard.py", "checkout_product.py"}
    if (not isinstance(files, dict) or not {"runner.py", "zircon-management.groovy"}.issubset(files)
            or not set(files).issubset(allowed_capsule_files)):
        raise ValueError("management_capsule_files_invalid")
    product_capabilities = tuple(
        {"enabled": True, "kind": "zircon-development-editor-preview", "schemaVersion": version, "helper": "checkout_product.py"}
        for version in (1, 2)
    )
    if "checkout_product.py" in files:
        if set(files) != allowed_capsule_files or config.get("checkoutProduct") not in product_capabilities:
            raise ValueError("management_product_capsule_files_invalid")
    elif config.get("checkoutProduct") is not None:
        raise ValueError("management_product_helper_missing")
    for name, expected_hash in files.items():
        if not isinstance(expected_hash, str) or not SHA256.fullmatch(expected_hash):
            raise ValueError("management_capsule_file_pin_invalid")
        if stable_hash(capsule / name) != expected_hash:
            raise ValueError("management_capsule_file_mismatch")

    clients = config.get("clientSources")
    required_clients = {
        "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1", "tools/dev/dev-fast-build.ps1",
        "tools/dev/submit-jenkins-management.ps1", "tools/jenkins/frontend.py",
        "tools/jenkins-management/entry.py",
    }
    if (not isinstance(clients, dict) or set(clients) != required_clients
            or any(clients[name] != source_hashes[name] for name in required_clients)):
        raise ValueError("native_helper_capsule_pin_mismatch")
    return capsule_digest


def native_complete(native: object, expected_command: list[str], repo: Path) -> bool:
    if not isinstance(native, dict):
        return False
    tree = native.get("processTree")
    job = native.get("nativeJobEvidence")
    if not isinstance(tree, dict) or not isinstance(job, dict):
        return False
    records = job.get("records")
    if not isinstance(records, dict):
        return False
    before = records.get("beforeResume", {})
    terminal = records.get("afterTerminal", {})
    closed = records.get("afterClose", {})
    identity_ok = (
        type(native.get("pid")) is int and native["pid"] > 0
        and isinstance(native.get("creationTime"), str)
        and native["creationTime"].isascii() and native["creationTime"].isdigit()
        and int(native["creationTime"]) > 0
        and native.get("command") == expected_command
        and native.get("cwd") == str(repo)
        and type(before.get("activeProcesses")) is int
        and before["activeProcesses"] >= 1
    )
    return bool(
        identity_ok
        and tree.get("scope") == "retained-windows-job"
        and tree.get("terminal") is True
        and tree.get("nativeWaiterJoined") is True
        and tree.get("jobHandleRetainedUntilCliExit") is False
        and tree.get("readersFinished") is True
        and tree.get("pipeEOF") is True
        and tree.get("pipes", {}).get("stdout", {}).get("eof") is True
        and tree.get("pipes", {}).get("stderr", {}).get("eof") is True
        and type(native.get("rootExitCode")) is int
        and terminal.get("activeProcesses") == 0
        and closed.get("activeProcesses") == 0
        and not job.get("errors")
        and not native.get("terminalErrors")
    )


def compact_summary(run_id: str, repo: Path, output: Path, driver: str,
                    runtime: str, capsule: str | None, status: str,
                    native: dict | None, complete: bool, hashes_match: bool | None,
                    reason: str | None, cleanup_status: str = "pending",
                    junit_present: bool = False) -> dict:
    return {
        "schemaVersion": 1,
        "taskId": "jenkins-parallel-activation-20261011",
        "runId": run_id,
        "status": status,
        "formalAcceptance": False,
        "driverDigest": driver,
        "runtimeOperationId": runtime,
        "managementCapsuleSha256": capsule,
        "repositoryRoot": str(repo),
        "outputRoot": str(output),
        "nativeProofComplete": complete,
        "rootExitCode": native.get("rootExitCode") if native else None,
        "stdoutEof": bool(native and native.get("processTree", {}).get("pipes", {}).get("stdout", {}).get("eof") is True),
        "stderrEof": bool(native and native.get("processTree", {}).get("pipes", {}).get("stderr", {}).get("eof") is True),
        "sourceHashesMatch": hashes_match,
        "junitReceiptPresent": junit_present,
        "temporaryTreeCleanup": cleanup_status,
        "reasonCode": reason,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expected-driver", required=True)
    parser.add_argument("--expected-runtime", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--suite", choices=("ordinary", "workflow-routing"), default="ordinary")
    args = parser.parse_args(argv)

    output: Path | None = None
    native: dict | None = None
    capsule_digest: str | None = None
    hashes_match: bool | None = None
    source_before: dict[str, str] = {}
    stage = "preflight_rejected"
    try:
        if os.name != "nt":
            raise ValueError("windows_agent_required")
        if not SHA256.fullmatch(args.expected_driver):
            raise ValueError("driver_pin_invalid")
        if not ID.fullmatch(args.expected_runtime) or not ID.fullmatch(args.run_id):
            raise ValueError("runtime_or_run_id_invalid")
        repo = physical(Path(args.repo_root))
        if str(repo).casefold() != str(REPOSITORY).casefold():
            raise ValueError("repository_root_fixed")

        driver_manifest_name = f".jenkins/runtime/drivers/{args.expected_driver}/driver-manifest.json"
        routing_sources = ("tools/jenkins-management/test_job.py",
                           "tools/jenkins-management/tests/test_test_job.py",
                           "tools/jenkins-management/zircon-management-tests.groovy") if args.suite == "workflow-routing" else ()
        source_names = (*SOURCE_NAMES, *routing_sources, driver_manifest_name)
        source_before = {name: stable_hash(repo / Path(name)) for name in source_names}
        capsule_digest = verify_pins(repo, args.expected_driver, args.expected_runtime, source_before)

        from tools.jenkins.deployment.driver import driver_environment

        local_cargo_spec = importlib.util.spec_from_file_location(
            "zircon_jenkins_local_cargo", repo / "tools/dev/local_cargo.py",
        )
        if local_cargo_spec is None or local_cargo_spec.loader is None:
            raise ValueError("native_helper_load_failed")
        local_cargo = importlib.util.module_from_spec(local_cargo_spec)
        sys.modules[local_cargo_spec.name] = local_cargo
        local_cargo_spec.loader.exec_module(local_cargo)
        _run_owned_command = local_cargo._run_owned_command
        validate_output_path = local_cargo.validate_output_path

        output = OUTPUT_BASE / args.run_id
        output = validate_output_path(output)
        physical(output, must_exist=False)
        if output.exists():
            raise ValueError("run_id_already_registered")
        output.mkdir(parents=True, exist_ok=False)
        test_file = repo / "tools/jenkins-management/tests/test_ordinary_flow.py"
        test_files = [str(test_file)]
        if args.suite == "workflow-routing":
            test_files = [str(repo / "tools/jenkins-management/tests/test_test_job.py")]
        python = repo / ".jenkins/runtime/python/python.exe"
        basetemp = output / "basetemp"
        temp = output / "temp"
        cache = output / "cache"
        pytest_config = output / "pytest.ini"
        junit = output / "pytest-junit.xml"
        native_receipt = output / "native-terminal.json"
        source_receipt = output / "source-hashes.json"
        summary_path = output / "summary.json"

        # Register every owned output and input before creating temp trees or starting pytest.
        registry = {
            "schemaVersion": 1,
            "taskId": "jenkins-parallel-activation-20261011",
            "owner": "jenkins-management-ordinary-flow-regression",
            "runId": args.run_id,
            "testSuite": args.suite,
            "repositoryRoot": str(repo),
            "outputRoot": str(output),
            "sourceInputs": {
                name: {"path": str(repo / Path(name)), "sha256Before": source_before[name]}
                for name in source_names
            },
            "dependencies": [str(python), "pytest from the pinned repository Python runtime", "retained Windows Job helper"],
            "outputs": [str(output / name) for name in (
                "artifacts.json", "stdout.bin", "stderr.bin", "started.json",
                "native-terminal.json", "source-hashes.json", "pytest.ini",
                "pytest-junit.xml", "cleanup-plan.json", "cleanup.json", "summary.json",
            )],
            "outputDirectories": [str(basetemp), str(temp), str(cache)],
            "retention": "retain native and test receipts; remove only registered temp trees after native EOF proof",
            "formalAcceptance": False,
        }
        manifest_bytes = encoded(registry) + b"\n"
        with (output / "artifacts.json").open("xb") as stream:
            stream.write(manifest_bytes)
            stream.flush()
            os.fsync(stream.fileno())

        temp.mkdir(exist_ok=False)
        pytest_config.write_text("[pytest]\naddopts =\n", encoding="utf-8", newline="\n")
        driver_root = repo / ".jenkins/runtime/drivers" / args.expected_driver
        command = [
            str(python), "-B", "-Xutf8", "-P", "-m", "pytest",
            "-c", str(pytest_config), *test_files, "-q", "--noconftest",
            "--confcutdir", str(test_file.parent), "--basetemp", str(basetemp),
            "--junitxml", str(junit), "-o", "addopts=",
            "-o", "cache_dir=" + str(cache),
        ]
        environment = {
            key: os.environ[key]
            for key in ("PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "PATHEXT", "SYSTEMDRIVE")
            if key in os.environ
        }
        environment.update(driver_environment(
            {"root": str(driver_root), "driverDigest": args.expected_driver}, python, repo,
        ))
        environment.update({
            "TEMP": str(temp), "TMP": str(temp), "TMPDIR": str(temp),
            "PYTHONPYCACHEPREFIX": str(cache / "pycache"),
            "PYTHONDONTWRITEBYTECODE": "1", "PYTHONNOUSERSITE": "1",
            "PYTHONSAFEPATH": "1", "PYTHONPATH": os.pathsep.join((str(driver_root), str(repo / ".jenkins/runtime/test-deps"))),
            "PYTEST_ADDOPTS": "", "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
            "GIT_TERMINAL_PROMPT": "0", "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": "NUL",
            "GIT_AUTHOR_NAME": "Zircon Jenkins Regression",
            "GIT_AUTHOR_EMAIL": "jenkins-regression@invalid",
            "GIT_COMMITTER_NAME": "Zircon Jenkins Regression",
            "GIT_COMMITTER_EMAIL": "jenkins-regression@invalid",
            "ZIRCON_REPO_ROOT": str(repo), "ZIRCON_DRIVER_DIGEST": args.expected_driver,
            "ZIRCON_RUNTIME_OPERATION_ID": args.expected_runtime,
            "JENKINS_PYTHON": str(python),
        })
        stage = "pytest_native_run"
        native = _run_owned_command(command, repo, environment, output, timeout=900)
        complete = native_complete(native, command, repo)
        write_json(native_receipt, native)
        source_after = {name: stable_hash(repo / Path(name)) for name in source_names}
        hashes_match = source_before == source_after
        write_json(source_receipt, {
            "schemaVersion": 1,
            "files": {
                name: {"before": source_before[name], "after": source_after[name],
                       "matches": source_before[name] == source_after[name]}
                for name in source_names
            },
        })
        junit_present = junit.is_file() and junit.stat().st_size > 0

        cleanup_status = "pending"
        if complete and hashes_match:
            try:
                remove_registered_trees(output, ("basetemp", "temp", "cache"), manifest_bytes)
                cleanup_status = "cleaned"
            except Exception:
                cleanup_status = "pending"

        exit_code = native.get("rootExitCode")
        if complete and type(exit_code) is int and exit_code == 0 and hashes_match and junit_present:
            status, reason = "passed", None
        elif complete:
            status = "failed"
            reason = ("source_hash_mismatch" if not hashes_match else
                      "junit_receipt_missing" if not junit_present else "pytest_exit_nonzero")
        else:
            status, reason = "unknown", "native_terminal_or_eof_unproven"
        write_json(summary_path, compact_summary(
            args.run_id, repo, output, args.expected_driver, args.expected_runtime,
            capsule_digest, status, native, complete, hashes_match, reason, cleanup_status,
            junit_present,
        ))
        print(json.dumps({"runId": args.run_id, "status": status, "reasonCode": reason,
                          "summary": str(summary_path)}, separators=(",", ":")))
        return 0 if status == "passed" else (1 if status == "failed" else 2)
    except Exception as error:
        # The exception text may contain host details; only a fixed reason code is emitted.
        fixed_reasons = {
            "selected_runtime_pin_mismatch", "management_capsule_pin_invalid",
            "management_capsule_identity_mismatch", "management_capsule_runtime_mismatch",
            "management_capsule_files_invalid", "management_product_capsule_files_invalid",
            "management_product_helper_missing", "management_capsule_file_pin_invalid",
            "management_capsule_file_mismatch", "native_helper_capsule_pin_mismatch",
            "run_id_already_registered", "repository_root_fixed", "windows_agent_required",
            "driver_pin_invalid", "runtime_or_run_id_invalid",
        }
        reason = str(error) if isinstance(error, ValueError) and str(error) in fixed_reasons else stage
        if output is not None and output.exists() and not (output / "summary.json").exists():
            try:
                repo = REPOSITORY
                write_json(output / "summary.json", compact_summary(
                    args.run_id, repo, output, args.expected_driver, args.expected_runtime,
                    capsule_digest, "unknown", native, False, hashes_match, reason,
                ))
            except Exception:
                pass
        print(json.dumps({"runId": args.run_id, "status": "unknown", "reasonCode": reason},
                         separators=(",", ":")))
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
