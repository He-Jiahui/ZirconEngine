from __future__ import annotations

import json
import os
import secrets
import subprocess
import threading
import ctypes
from dataclasses import asdict, dataclass
from datetime import date
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from .config import CoordinatorConfig
from .baselines import BaselineService
from .database import Database
from .leases import LeaseService, PathPolicy
from .migrations import migrate
from .models import CoordinatorError, SessionStatus
from .sessions import SessionService
from .patches import PatchService, PatchStatus
from .failures import FailureGraphService, FailureResolution
from .plans import PlanRepository
from .snapshots import ObjectStore, SnapshotService
from .watch import WorkspaceWatcher


def _atomic_json_write(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    os.replace(temporary, path)


def _pid_is_alive(pid: int) -> bool:
    if pid <= 0:
        return False
    if os.name == "nt":
        process_query_limited_information = 0x1000
        kernel32 = ctypes.windll.kernel32
        kernel32.OpenProcess.argtypes = [ctypes.c_uint32, ctypes.c_bool, ctypes.c_uint32]
        kernel32.OpenProcess.restype = ctypes.c_void_p
        kernel32.CloseHandle.argtypes = [ctypes.c_void_p]
        kernel32.CloseHandle.restype = ctypes.c_bool
        handle = kernel32.OpenProcess(
            process_query_limited_information, False, pid
        )
        if not handle:
            return False
        kernel32.CloseHandle(handle)
        return True
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


class CoordinatorApplication:
    def __init__(self, config: CoordinatorConfig):
        self.config = config
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, config.repo_root)
        self.baselines = BaselineService(self.database, config.repo_root)
        self.object_store = ObjectStore(self.database, config.object_root)
        self.snapshots = SnapshotService(
            self.database, config.repo_root, self.object_store
        )
        self.leases = LeaseService(
            self.database,
            PathPolicy(config.repo_root),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.patches = PatchService(
            self.database,
            config.repo_root,
            self.object_store,
            self.snapshots,
            self.leases,
            self.sessions,
        )
        self.watcher = WorkspaceWatcher(self.baselines)
        self.plans = PlanRepository(config.repo_root)
        self.failures = FailureGraphService(self.database, config.repo_root)
        self.branch = self._branch()

    @property
    def read_only(self) -> bool:
        return self.branch != "main"

    def health(self) -> dict[str, Any]:
        try:
            baseline_health = self.baselines.current().health.value
        except CoordinatorError as error:
            if error.code != "baseline_missing":
                raise
            baseline_health = "uninitialized"
        return {
            "status": "ok",
            "branch": self.branch,
            "mode": "read_only" if self.read_only else "read_write",
            "repo_root": str(self.config.repo_root),
            "pid": os.getpid(),
            "baseline": baseline_health,
        }

    def command(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        read_only_commands = {
            "session.list",
            "session.show",
            "baseline.status",
            "baseline.diff",
            "lease.list",
            "patch.status",
            "patch.list",
            "plan.audit",
            "plan.owner",
            "failure.audit",
            "failure.open",
        }
        if self.read_only and name not in read_only_commands:
            raise CoordinatorError(
                "not_on_main",
                f"Coordinator mutations require main; current branch is {self.branch}",
            )
        if name == "session.register":
            session = self.sessions.register(
                session_id=str(arguments["session_id"]),
                display_name=arguments.get("display_name"),
                plan_path=arguments.get("plan_path"),
                write_scope=arguments.get("write_scope") or [],
            )
            if session.plan_path:
                self.failures.import_repository()
                open_failures = self.failures.open_for_plan(session.plan_path)
            else:
                open_failures = []
            if open_failures:
                session = self.sessions.set_status(
                    session.session_id,
                    SessionStatus.RESOLVING_FAILURE,
                    reason=f"{len(open_failures)} open failure handoff(s) require priority",
                )
            return {
                "session": session.to_dict(),
                "open_failures": [asdict(item) for item in open_failures],
            }
        if name == "session.list":
            sessions = self.sessions.list(include_archived=bool(arguments.get("include_archived")))
            return {"sessions": [session.to_dict() for session in sessions]}
        if name == "session.show":
            return {"session": self.sessions.get(str(arguments["session_id"])).to_dict()}
        if name == "session.heartbeat":
            return {"session": self.sessions.heartbeat(str(arguments["session_id"])).to_dict()}
        if name == "session.set_status":
            status = SessionStatus(str(arguments["status"]))
            session = self.sessions.set_status(
                str(arguments["session_id"]), status, reason=arguments.get("reason")
            )
            return {"session": session.to_dict()}
        if name == "baseline.init":
            return {"baseline": self._baseline_dict(self.baselines.initialize())}
        if name == "baseline.status":
            return {"baseline": self._baseline_dict(self.baselines.current())}
        if name in {"baseline.diff", "baseline.scan"}:
            changes = self.baselines.scan() if name == "baseline.scan" else self.baselines.diff()
            return {"changes": [asdict(change) for change in changes]}
        if name == "baseline.attribute":
            self.baselines.attribute(str(arguments["session_id"]), arguments.get("paths") or [])
            return {"status": "attributed"}
        if name == "baseline.accept":
            baseline = self.baselines.accept(reason=str(arguments["reason"]))
            return {"baseline": self._baseline_dict(baseline)}
        if name == "lease.claim":
            result = self.leases.acquire(str(arguments["session_id"]), arguments.get("paths") or [])
            return {"lease": asdict(result)}
        if name == "lease.release":
            released = self.leases.release(
                str(arguments["session_id"]), arguments.get("paths")
            )
            processed = self.patches.process_queue()
            return {
                "released": released,
                "processed_patches": [self._patch_dict(patch) for patch in processed],
            }
        if name == "lease.heartbeat":
            return {"renewed": self.leases.heartbeat(str(arguments["session_id"]))}
        if name == "lease.list":
            return {"leases": self.leases.list()}
        if name == "snapshot.create":
            snapshot = self.snapshots.create(
                session_id=str(arguments["session_id"]),
                paths=arguments.get("paths") or [],
                baseline_epoch=arguments.get("baseline_epoch"),
                purpose=str(arguments["purpose"]),
            )
            return {"snapshot": asdict(snapshot)}
        if name == "snapshot.preview":
            preview = self.snapshots.restore_preview(int(arguments["snapshot_id"]))
            return {"preview": [asdict(item) for item in preview]}
        if name == "patch.enqueue":
            patch = self.patches.submit(
                str(arguments["session_id"]),
                str(arguments["patch_text"]),
                arguments.get("targets") or [],
            )
            return {"patch": self._patch_dict(patch)}
        if name == "patch.status":
            return {"patch": self._patch_dict(self.patches.get(int(arguments["patch_id"])))}
        if name == "patch.list":
            requested_status = arguments.get("status")
            status = PatchStatus(str(requested_status)) if requested_status else None
            return {"patches": [self._patch_dict(item) for item in self.patches.list(status=status)]}
        if name == "patch.process":
            return {"patches": [self._patch_dict(item) for item in self.patches.process_queue()]}
        if name == "watch.scan":
            return {"changes": [asdict(item) for item in self.watcher.scan_once()]}
        if name == "plan.audit":
            inventory = self.plans.scan()
            return {
                "formal_plans": [asdict(item) for item in inventory.formal_plans],
                "legacy_documents": list(inventory.legacy_documents),
            }
        if name == "plan.owner":
            return {"owner": asdict(self.plans.resolve_owner(str(arguments["plan_path"])))}
        if name == "plan.authorize":
            session = self.sessions.get(str(arguments["session_id"]))
            if not session.plan_path:
                raise CoordinatorError(
                    "session_plan_missing", "Session must register a numbered plan before plan writes"
                )
            decision = self.plans.authorize_write(
                session.plan_path,
                str(arguments["target_path"]),
                maintenance=bool(arguments.get("maintenance")),
            )
            return {"decision": asdict(decision)}
        if name == "failure.import":
            return {"audit": self._failure_audit_dict(self.failures.import_repository())}
        if name == "failure.audit":
            return {"audit": self._failure_audit_dict(self.failures.audit())}
        if name == "failure.open":
            nodes = self.failures.open_for_plan(str(arguments["fixing_plan"]))
            return {"failures": [asdict(item) for item in nodes]}
        if name == "failure.return":
            destination = self.failures.return_fixed(
                str(arguments["lifecycle_key"]),
                FailureResolution(
                    root_cause=str(arguments["root_cause"]),
                    architecture_fix=str(arguments["architecture_fix"]),
                    validation=str(arguments["validation"]),
                    return_summary=str(arguments["return_summary"]),
                ),
                resolved_at=date.fromisoformat(str(arguments["resolved_at"])),
            )
            return {"fixed_artifact": destination.relative_to(self.config.repo_root).as_posix()}
        raise CoordinatorError("unknown_command", f"Unknown coordinator command {name}")

    @staticmethod
    def _baseline_dict(baseline) -> dict[str, Any]:
        return {
            "epoch_id": baseline.epoch_id,
            "head_commit": baseline.head_commit,
            "index_tree": baseline.index_tree,
            "health": baseline.health.value,
            "manifest_count": len(baseline.manifest),
            "degraded_reason": baseline.degraded_reason,
        }

    @staticmethod
    def _patch_dict(patch) -> dict[str, Any]:
        result = asdict(patch)
        result["status"] = patch.status.value
        return result

    @staticmethod
    def _failure_audit_dict(audit) -> dict[str, Any]:
        return {
            "node_count": audit.node_count,
            "nodes": [asdict(item) for item in audit.nodes],
            "diagnostics": [asdict(item) for item in audit.diagnostics],
        }

    def _branch(self) -> str:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=self.config.repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()


class _CoordinatorHttpServer(ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, address, handler, *, application: CoordinatorApplication, token: str):
        super().__init__(address, handler)
        self.application = application
        self.token = token


class CoordinatorRequestHandler(BaseHTTPRequestHandler):
    server: _CoordinatorHttpServer

    def do_GET(self) -> None:
        if not self._authorized():
            return
        if self.path == "/health":
            self._write_json(HTTPStatus.OK, self.server.application.health())
            return
        self._write_error(HTTPStatus.NOT_FOUND, "not_found", "Unknown endpoint")

    def do_POST(self) -> None:
        if not self._authorized():
            return
        if self.path == "/shutdown":
            self._write_json(HTTPStatus.OK, {"status": "stopping"})
            threading.Thread(target=self.server.shutdown, daemon=True).start()
            return
        if self.path != "/command":
            self._write_error(HTTPStatus.NOT_FOUND, "not_found", "Unknown endpoint")
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            command = str(payload["command"])
            arguments = payload.get("arguments") or {}
            if not isinstance(arguments, dict):
                raise ValueError("arguments must be an object")
            result = self.server.application.command(command, arguments)
            self._write_json(HTTPStatus.OK, result)
        except CoordinatorError as error:
            self._write_json(HTTPStatus.CONFLICT, {"error": error.to_dict()})
        except (KeyError, TypeError, ValueError, json.JSONDecodeError) as error:
            self._write_error(HTTPStatus.BAD_REQUEST, "invalid_request", str(error))
        except Exception as error:  # pragma: no cover - defensive service boundary
            self._write_error(HTTPStatus.INTERNAL_SERVER_ERROR, "internal_error", str(error))

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def _authorized(self) -> bool:
        if self.headers.get("Authorization") == f"Bearer {self.server.token}":
            return True
        self._write_error(HTTPStatus.UNAUTHORIZED, "unauthorized", "Invalid coordinator token")
        return False

    def _write_error(self, status: HTTPStatus, code: str, message: str) -> None:
        self._write_json(status, {"error": {"code": code, "message": message, "details": {}}})

    def _write_json(self, status: HTTPStatus, payload: dict[str, Any]) -> None:
        encoded = json.dumps(payload, sort_keys=True).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)


@dataclass(slots=True)
class RunningCoordinator:
    config: CoordinatorConfig
    httpd: _CoordinatorHttpServer
    thread: threading.Thread
    maintenance_thread: threading.Thread
    maintenance_stop: threading.Event
    token: str

    @classmethod
    def start(cls, config: CoordinatorConfig) -> "RunningCoordinator":
        config.state_root.mkdir(parents=True, exist_ok=True)
        cls._acquire_lock(config)
        token = secrets.token_urlsafe(32)
        try:
            application = CoordinatorApplication(config)
            httpd = _CoordinatorHttpServer(
                (config.host, config.port),
                CoordinatorRequestHandler,
                application=application,
                token=token,
            )
            thread = threading.Thread(target=httpd.serve_forever, name="zircon-session-coordinator", daemon=True)
            thread.start()
            maintenance_stop = threading.Event()
            maintenance_thread = threading.Thread(
                target=cls._maintenance_loop,
                args=(application, config.watch_interval_seconds, maintenance_stop),
                name="zircon-session-coordinator-watch",
                daemon=True,
            )
            maintenance_thread.start()
            host, port = httpd.server_address[:2]
            _atomic_json_write(
                config.runtime_path,
                {"host": host, "port": port, "token": token, "pid": os.getpid(), "repo_root": str(config.repo_root)},
            )
            return cls(
                config=config,
                httpd=httpd,
                thread=thread,
                maintenance_thread=maintenance_thread,
                maintenance_stop=maintenance_stop,
                token=token,
            )
        except BaseException:
            cls._remove_owned_file(config.lock_path, os.getpid())
            raise

    @property
    def base_url(self) -> str:
        host, port = self.httpd.server_address[:2]
        return f"http://{host}:{port}"

    def stop(self) -> None:
        self.maintenance_stop.set()
        self.httpd.shutdown()
        self.httpd.server_close()
        self.thread.join(timeout=5)
        self.maintenance_thread.join(timeout=5)
        self._remove_owned_file(self.config.runtime_path, os.getpid())
        self._remove_owned_file(self.config.lock_path, os.getpid())

    def __enter__(self) -> "RunningCoordinator":
        return self

    def __exit__(self, _exc_type, _exc_value, _traceback) -> None:
        self.stop()

    @staticmethod
    def _acquire_lock(config: CoordinatorConfig) -> None:
        if config.lock_path.exists():
            try:
                existing = json.loads(config.lock_path.read_text(encoding="utf-8"))
                if _pid_is_alive(int(existing.get("pid", 0))):
                    raise CoordinatorError("already_running", "Coordinator is already running")
            except (OSError, ValueError, TypeError, json.JSONDecodeError):
                pass
            config.lock_path.unlink(missing_ok=True)
        descriptor = json.dumps({"pid": os.getpid()})
        descriptor_fd = os.open(config.lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
        with os.fdopen(descriptor_fd, "w", encoding="utf-8") as stream:
            stream.write(descriptor)

    @staticmethod
    def _remove_owned_file(path: Path, pid: int) -> None:
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
            if int(payload.get("pid", -1)) == pid:
                path.unlink(missing_ok=True)
        except (OSError, ValueError, TypeError, json.JSONDecodeError):
            return

    @staticmethod
    def _maintenance_loop(
        application: CoordinatorApplication,
        interval_seconds: float,
        stop_event: threading.Event,
    ) -> None:
        while not stop_event.wait(max(interval_seconds, 0.05)):
            try:
                application.watcher.scan_once()
            except Exception as error:  # pragma: no cover - defensive long-lived boundary
                with application.database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, datetime('now'))",
                        (
                            "watch.scan_failed",
                            json.dumps({"error": str(error)}, sort_keys=True),
                        ),
                    )


def run_forever(config: CoordinatorConfig) -> None:
    running = RunningCoordinator.start(config)
    try:
        running.thread.join()
    except KeyboardInterrupt:
        pass
    finally:
        running.stop()
