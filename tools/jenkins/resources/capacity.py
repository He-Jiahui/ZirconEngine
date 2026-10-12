"""Atomic in-process capacity ledger used by the state adapter."""
from __future__ import annotations
from dataclasses import dataclass
import math
import ntpath
import time
from threading import Lock
from uuid import uuid4
from ..contracts import JenkinsError, digest


def _priority(value):
    if type(value) not in (int, float) or not math.isfinite(value) or not -1000 <= value <= 1000:
        raise JenkinsError("invalid_priority", "Priority must be a finite number between -1000 and 1000")
    return value


def _capacity(value, *, positive=False):
    if (not isinstance(value, Capacity)
            or any(type(item) is not int or item < (1 if positive else 0)
                   for item in (value.cpu, value.memory_bytes, value.disk_bytes))):
        raise JenkinsError("invalid_reservation", "Resource estimates must be bounded nonnegative integers")


def _stored_root(value):
    """Recognize exact historical roots without following or repairing aliases."""
    from .paths import APPROVED_NAMES
    if not isinstance(value, str):
        return ""
    raw = value.replace("/", "\\")
    if ntpath.normpath(raw) != raw:
        return ""
    return next((root for root in APPROVED_NAMES if raw.casefold() == root.casefold()), "")


def _requested_root(value):
    if not value:
        return "", ""
    from .paths import canonical_build_root
    root = canonical_build_root(value)
    return str(root.path), root.volume_identity


def _fresh_inventory(value, maximum_age):
    observed = value.get("observedAt")
    if (value.get("complete") is not True or type(observed) not in (int, float)
            or not math.isfinite(observed) or not 0 <= time.time() - observed <= maximum_age
            or any(type(value.get(field)) is not int or value[field] <= 0
                   for field in ("cpu", "memoryBytes", "diskBytes"))):
        raise JenkinsError("storage_snapshot_stale", "Fresh positive complete capacity inventory is required", retryable=True)

@dataclass(frozen=True)
class Capacity:
    cpu: int
    memory_bytes: int
    disk_bytes: int

@dataclass(frozen=True)
class Reservation:
    id: str
    owner: str
    amount: Capacity
    priority: int = 0

class ReservationBook:
    def __init__(self, total: Capacity):
        if min(total.cpu, total.memory_bytes, total.disk_bytes) < 0:
            raise JenkinsError("invalid_capacity", "Capacity cannot be negative")
        self.total = total
        self._used = Capacity(0, 0, 0)
        self._items: dict[str, Reservation] = {}
        self._lock = Lock()

    @property
    def used(self) -> Capacity:
        with self._lock: return self._used

    def reserve(self, owner: str, amount: Capacity, *, priority: int = 0) -> Reservation | None:
        if min(amount.cpu, amount.memory_bytes, amount.disk_bytes) < 0:
            raise JenkinsError("invalid_reservation", "Reservation cannot be negative")
        with self._lock:
            candidate = Capacity(self._used.cpu + amount.cpu, self._used.memory_bytes + amount.memory_bytes, self._used.disk_bytes + amount.disk_bytes)
            if candidate.cpu > self.total.cpu or candidate.memory_bytes > self.total.memory_bytes or candidate.disk_bytes > self.total.disk_bytes:
                return None
            r = Reservation(uuid4().hex, owner, amount, priority)
            self._items[r.id] = r; self._used = candidate
            return r

    def release(self, reservation_id: str) -> bool:
        with self._lock:
            r = self._items.pop(reservation_id, None)
            if r is None: return False
            self._used = Capacity(self._used.cpu-r.amount.cpu, self._used.memory_bytes-r.amount.memory_bytes, self._used.disk_bytes-r.amount.disk_bytes)
            return True

class ResourceManager:
    """SQLite-backed reservations; the State transaction is the authority."""
    def __init__(self, state, total: Capacity):
        self.state, self.total = state, total

    def queue(self, owner: str, *, priority: int = 0, request_id: str | None = None, amount: Capacity | None = None,
              build_root: str = "", heavy_writer: bool = False, writer_key: str | None = None):
        _priority(priority)
        if amount is not None:
            _capacity(amount, positive=heavy_writer)
        root, _ = _requested_root(build_root)
        key=f"{owner}:{request_id or uuid4().hex}"
        with self.state.transaction() as conn:
            old=self.state.get("resource_queue", key, connection=conn)
            desired = {"owner": owner, "priority": priority, "amount": amount.__dict__ if amount else None,
                       "buildRoot": root, "heavyWriter": heavy_writer, "writerKey": writer_key}
            if old:
                # Old rootless entries retain their identity and age. New
                # entries can never move drives or alter priority on retry.
                compared = ["owner", "priority", "amount", *[field for field in
                            ("buildRoot", "heavyWriter", "writerKey") if field in old["payload"]]]
                if any(old["payload"].get(field) != desired[field] for field in compared):
                    raise JenkinsError("request_payload_mismatch", "Queued resource identity changed")
                return old
            return self.state.put("resource_queue", key, {**desired, "queuedAt": time.time(), "status": "waiting"},
                                  expected_version=0, connection=conn)

    def fair_queue(self, *, connection=None):
        rows=[r for r in self.state.list("resource_queue", connection=connection) if r["payload"].get("status")=="waiting"]
        policy = self.state.get("resource_policy", "default", connection=connection)
        aging = policy["payload"].get("agingSeconds", 60) if policy else 60
        if type(aging) not in (int, float) or not math.isfinite(aging) or aging <= 0:
            raise JenkinsError("resource_policy_invalid", "Queue aging must be a finite positive interval")
        now = time.time()
        for row in rows:
            value = row["payload"]
            _priority(value.get("priority", 0))
            queued = value.get("queuedAt")
            if type(queued) not in (int, float) or not math.isfinite(queued) or queued > now:
                raise JenkinsError("resource_queue_invalid", "Queue age is unknown")
        return sorted(rows, key=lambda r: (-(r["payload"].get("priority", 0)
                      + (now - r["payload"]["queuedAt"]) / aging), r["payload"]["queuedAt"], r["key"]))

    def publish_inventory(self, generation: str, capacity: Capacity, *, complete: bool = True, observed_at: float | None = None, build_root: str = ""):
        root, volume = _requested_root(build_root)
        value = {"generation": generation, "cpu": capacity.cpu, "memoryBytes": capacity.memory_bytes,
                 "diskBytes": capacity.disk_bytes, "complete": complete,
                 "observedAt": time.time() if observed_at is None else observed_at, "buildRoot": root,
                 "volumeIdentity": volume}
        if not isinstance(generation, str) or not generation:
            raise JenkinsError("storage_snapshot_stale", "Inventory generation must be known")
        _fresh_inventory(value, 30)
        with self.state.transaction() as conn:
            old = self.state.get("capacity_inventory", generation, connection=conn)
            if old:
                if old["payload"] != value:
                    raise JenkinsError("inventory_generation_conflict", "Inventory generations are immutable")
                return old
            return self.state.put("capacity_inventory", generation, value, expected_version=0, connection=conn)

    def capture_inventory(self, build_root: str):
        """Capture a fresh local snapshot; callers may apply policy limits later."""
        import os, time, multiprocessing
        build_root, volume = _requested_root(build_root)
        disk = 0
        if os.name == "nt":
            import ctypes
            free = ctypes.c_ulonglong()
            if not ctypes.windll.kernel32.GetDiskFreeSpaceExW(str(build_root), ctypes.byref(free), None, None): raise JenkinsError("inventory_unknown", "Disk inventory unavailable", retryable=True)
            disk = int(free.value)
            # GlobalMemoryStatusEx supplies physical availability. CPU count is
            # a conservative admission ceiling, not a claim of live usage.
            class M(ctypes.Structure): _fields_=[('length',ctypes.c_ulong),('memoryLoad',ctypes.c_ulong),('total',ctypes.c_ulonglong),('avail',ctypes.c_ulonglong),('pageTotal',ctypes.c_ulonglong),('pageAvail',ctypes.c_ulonglong),('virtTotal',ctypes.c_ulonglong),('virtAvail',ctypes.c_ulonglong),('extAvail',ctypes.c_ulonglong)]
            mem=M(); mem.length=ctypes.sizeof(M)
            if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem)): raise JenkinsError("inventory_unknown", "Memory inventory unavailable", retryable=True)
            memory=int(mem.avail)
        else:
            disk = int(os.statvfs(build_root).f_bavail * os.statvfs(build_root).f_frsize); memory=int(os.sysconf('SC_PAGE_SIZE')*os.sysconf('SC_AVPHYS_PAGES'))
        generation = f"inventory-{time.time_ns()}"
        capacity = Capacity(max(1,multiprocessing.cpu_count()), memory, disk)
        return {"generation": generation, "cpu": capacity.cpu, "memoryBytes": capacity.memory_bytes,
                "diskBytes": capacity.disk_bytes, "complete": True, "observedAt": time.time(), "buildRoot": str(build_root),
                "volumeIdentity": volume}

    def scan_inventory(self, build_root: str):
        value = self.capture_inventory(build_root)
        return self.publish_inventory(value["generation"], Capacity(value["cpu"], value["memoryBytes"], value["diskBytes"]),
                                      observed_at=value["observedAt"], build_root=value["buildRoot"]), value["generation"]

    def _root_disk_only_blocked(self, queued, active, policy, *, connection):
        """A fair head can yield only for proven pressure on its own volume."""
        payload = queued["payload"]
        if not {"buildRoot", "amount", "heavyWriter", "writerKey"} <= set(payload):
            return None
        root = _stored_root(payload.get("buildRoot"))
        if not root or not isinstance(payload.get("amount"), dict):
            return None
        try:
            root, volume = _requested_root(root)
            amount = Capacity(**payload["amount"])
            _capacity(amount, positive=payload["heavyWriter"])
        except (JenkinsError, TypeError, ValueError, OSError):
            return None
        inventories = sorted(self.state.list("capacity_inventory", connection=connection),
                             key=lambda row: row["updatedAt"], reverse=True)
        selected = None
        for row in inventories:
            value = row["payload"]
            if _stored_root(value.get("buildRoot")) != _stored_root(root):
                continue
            try:
                _fresh_inventory(value, min(30, policy.get("inventoryMaxAgeSeconds", 30)))
            except (JenkinsError, TypeError, ValueError):
                continue
            if value.get("volumeIdentity") == volume and volume:
                selected = row
                break
        if selected is None:
            return None
        inventory = selected["payload"]
        used = Capacity(sum(row["payload"]["cpu"] for row in active),
                        sum(row["payload"]["memoryBytes"] for row in active),
                        sum(row["payload"]["diskBytes"] for row in active))
        if (used.cpu + amount.cpu > min(self.total.cpu, inventory["cpu"])
                or used.memory_bytes + amount.memory_bytes > min(self.total.memory_bytes, inventory["memoryBytes"])
                or used.disk_bytes + amount.disk_bytes > self.total.disk_bytes):
            return None
        if payload["heavyWriter"]:
            heavy = [row for row in active if row["payload"].get("heavyWriter")]
            if len(heavy) >= policy.get("maxHeavyWriters", 1):
                return None
            if "maxNewHeavyWriters" in policy:
                protected = (policy.get("quarantineReserve") or {}).get("executionIds", [])
                if sum(row["payload"].get("owner") not in protected for row in heavy) >= policy["maxNewHeavyWriters"]:
                    return None
        if payload.get("writerKey"):
            for domain in ("pool_writer_hold", "cache_maintenance_hold"):
                hold = self.state.get(domain, payload["writerKey"], connection=connection)
                if hold and hold["payload"].get("status") == "active":
                    return None
        growth = sum(row["payload"]["diskBytes"] for row in active
                     if not _stored_root(row["payload"].get("buildRoot"))
                     or _stored_root(row["payload"].get("buildRoot")) == _stored_root(root))
        free = inventory["diskBytes"] - policy.get("diskReserveBytes", 35 * 1024**3)
        if growth + amount.disk_bytes <= max(0, free):
            return None
        return {"queueKey": queued["key"], "buildRoot": root, "reasonCode": "root_disk_capacity",
                "queueVersion": queued["version"], "inventoryGeneration": selected["key"],
                "inventoryDigest": digest(inventory),
                "freeDiskBytes": inventory["diskBytes"], "outstandingRootGrowthBytes": growth,
                "requestedDiskBytes": amount.disk_bytes, "diskReserveBytes": policy.get("diskReserveBytes", 35 * 1024**3)}

    def admit(self, owner: str, amount: Capacity, *, inventory_generation: str = "", writer_key: str | None = None, max_age_seconds: float = 30.0, priority: int = 0, request_id: str | None = None, build_root: str = "", heavy_writer: bool = False):
        _capacity(amount, positive=heavy_writer)
        _priority(priority)
        if type(max_age_seconds) not in (int, float) or not math.isfinite(max_age_seconds) or max_age_seconds <= 0:
            raise JenkinsError("resource_policy_invalid", "Inventory freshness interval must be positive")
        build_root, volume_identity = _requested_root(build_root)
        key = f"{owner}:{request_id or uuid4().hex}"
        with self.state.transaction() as conn:
            inv = self.state.get("capacity_inventory", inventory_generation, connection=conn) if inventory_generation else None
            if not inv:
                raise JenkinsError("storage_snapshot_stale", "Complete capacity inventory is required", retryable=True)
            policy_record = self.state.get("resource_policy", "default", connection=conn)
            policy = policy_record["payload"] if policy_record else {}
            maximum_age = policy.get("inventoryMaxAgeSeconds", 30)
            if type(maximum_age) not in (int, float) or not math.isfinite(maximum_age) or maximum_age <= 0:
                raise JenkinsError("resource_policy_invalid", "Inventory freshness policy is invalid")
            _fresh_inventory(inv["payload"], min(30, max_age_seconds, maximum_age))
            allowed = policy.get('allowedBuildRoots')
            if allowed and build_root.casefold() not in {_stored_root(root).casefold() for root in allowed}:
                raise JenkinsError('build_root_not_current', 'New reservations must use the current configured build root')
            if build_root and _stored_root(inv["payload"].get("buildRoot")) != _stored_root(build_root):
                raise JenkinsError("inventory_root_mismatch", "Inventory belongs to a different build root", retryable=True)
            if inv["payload"].get("volumeIdentity") and inv["payload"]["volumeIdentity"] != volume_identity:
                raise JenkinsError("inventory_root_mismatch", "Inventory belongs to a different physical volume", retryable=True)
            existing = self.state.get("resource_reservation", key, connection=conn)
            if existing:
                if any(existing["payload"].get(k) != v for k,v in {"owner":owner,"cpu":amount.cpu,"memoryBytes":amount.memory_bytes,"diskBytes":amount.disk_bytes,"priority":priority,"buildRoot":build_root,"heavyWriter":heavy_writer}.items()): raise JenkinsError("request_payload_mismatch", "Reservation request changed")
                if existing["payload"].get("status") == "active": return existing
            waiting=self.fair_queue(connection=conn)
            queued = self.state.get("resource_queue", key, connection=conn)
            expected_queue = {"owner": owner, "priority": priority, "amount": amount.__dict__,
                              "buildRoot": build_root, "heavyWriter": heavy_writer, "writerKey": writer_key}
            if queued and any(queued["payload"].get(field) != value for field, value in expected_queue.items()
                              if field in queued["payload"]):
                raise JenkinsError("request_payload_mismatch", "Admission differs from its queued identity")
            if waiting and not any(row["key"] == key for row in waiting):
                return None
            if writer_key:
                maintenance = self.state.get("cache_maintenance_hold", writer_key, connection=conn)
                if maintenance and maintenance["payload"].get("status") == "active":
                    raise JenkinsError("pool_cache_maintenance_active", "Pool cache maintenance is still held", retryable=True)
                hold = self.state.get("pool_writer_hold", writer_key, connection=conn)
                if hold and hold["payload"].get("status") == "active":
                    raise JenkinsError("pool_writer_active", "Pool writer is already held", retryable=True)
            rows = self.state.list("resource_reservation", connection=conn)
            used = Capacity(0, 0, 0)
            root_growth = 0
            for row in rows:
                p = row["payload"]
                if p.get("status") == "active":
                    _capacity(Capacity(p.get("cpu"), p.get("memoryBytes"), p.get("diskBytes")))
                    used = Capacity(used.cpu+p["cpu"], used.memory_bytes+p["memoryBytes"], used.disk_bytes+p["diskBytes"])
                    # Existing physical bytes are already reflected by the
                    # scan. Charge only outstanding growth on this volume;
                    # unrecognized historical roots remain conservative.
                    historic_root = _stored_root(p.get("buildRoot"))
                    if not historic_root or not build_root or historic_root == _stored_root(build_root):
                        root_growth += p["diskBytes"]
            candidate = Capacity(used.cpu+amount.cpu, used.memory_bytes+amount.memory_bytes, used.disk_bytes+amount.disk_bytes)
            available = inv["payload"]
            floor = policy.get("diskReserveBytes", 35 * 1024**3)
            if type(floor) is not int or floor < 0:
                raise JenkinsError("resource_policy_invalid", "Disk reserve must be a nonnegative byte count")
            limits = Capacity(min(self.total.cpu, available["cpu"]), min(self.total.memory_bytes, available["memoryBytes"]), self.total.disk_bytes)
            if heavy_writer:
                active_heavy = [r for r in rows if r["payload"].get("status") == "active" and r["payload"].get("heavyWriter")]
                held = len(active_heavy)
                max_heavy = policy.get("maxHeavyWriters", 1)
                if type(max_heavy) is not int or max_heavy <= 0:
                    raise JenkinsError("resource_policy_invalid", "Heavy writer limit must be positive")
                if held >= max_heavy: return None
                if "maxNewHeavyWriters" in policy:
                    max_new = policy["maxNewHeavyWriters"]
                    quarantine = policy.get("quarantineReserve") or {}
                    protected = quarantine.get("executionIds", [])
                    if (type(max_new) is not int or not 0 < max_new <= max_heavy
                            or not isinstance(protected, list)
                            or any(not isinstance(eid, str) or not eid for eid in protected)
                            or (protected and quarantine.get("countedInBudget") is not True)):
                        raise JenkinsError("resource_policy_invalid", "New writer limit requires an exact budgeted historical set")
                    new_held = sum(r["payload"].get("owner") not in protected for r in active_heavy)
                    if new_held >= max_new: return None
            if candidate.cpu > limits.cpu or candidate.memory_bytes > limits.memory_bytes or candidate.disk_bytes > limits.disk_bytes:
                return None
            if root_growth + amount.disk_bytes > max(0, available["diskBytes"] - floor):
                return None
            skipped = []
            for queued_head in waiting:
                if queued_head["key"] == key:
                    break
                if policy.get("skipProvenRootDiskBlockedHeads") is not True:
                    return None
                blocked = self._root_disk_only_blocked(queued_head,
                    [row for row in rows if row["payload"].get("status") == "active"], policy, connection=conn)
                if blocked is None:
                    return None
                skipped.append(blocked)
            if writer_key:
                self.state.put("pool_writer_hold", writer_key, {"owner":owner,"status":"active","generation":inventory_generation,"reservationKey":key}, connection=conn)
            result=self.state.put("resource_reservation", key, {"owner":owner,"cpu":amount.cpu,"memoryBytes":amount.memory_bytes,"diskBytes":amount.disk_bytes,"inventoryGeneration":inventory_generation,"status":"active","priority":priority,"buildRoot":build_root,"heavyWriter":heavy_writer}, connection=conn)
            if skipped:
                self.state.put("resource_fair_decision", f"{key}:{inventory_generation}",
                               {"reservationKey": key, "buildRoot": build_root, "inventoryGeneration": inventory_generation,
                                "policyVersion": policy_record["version"], "skippedHeads": skipped,
                                "selectionPolicy": "first_aged_candidate_with_only_proven_root_disk_heads_skipped"}, connection=conn)
            if queued and queued["payload"].get("status")=="waiting": self.state.put("resource_queue", key, {**queued["payload"],"status":"admitted"}, expected_version=queued["version"], connection=conn)
            return result

    def release(self, key: str, *, writer_key: str | None = None, native_proof_ref: str | None = None, execution_id: str | None = None, native_terminal: bool = False) -> bool:
        with self.state.transaction() as conn:
            old = self.state.get("resource_reservation", key, connection=conn)
            if not old or old["payload"].get("status") != "active": return False
            proof = self.state.get("native_job", native_proof_ref, connection=conn) if native_proof_ref else None
            native_proof = proof["payload"].get("completeProof") if proof else None
            boot_proof = None
            if execution_id and not native_proof_ref:
                from ..processes.boot import boot_abort_evidence
                boot_proof = boot_abort_evidence(self.state, execution_id)
            if not boot_proof and (not proof or proof["payload"].get("status") != "terminal" or not isinstance(native_proof, dict) or native_proof.get("complete") is not True):
                raise JenkinsError("native_termination_unproven", "Cannot release reservation before trusted native proof", retryable=True)
            if (proof["payload"].get("owner") if proof else execution_id) != old["payload"].get("owner"):
                raise JenkinsError("native_owner_mismatch", "Native proof owner does not match reservation")
            if execution_id and proof and proof["payload"].get("executionId") != execution_id:
                raise JenkinsError("native_execution_mismatch", "Native proof execution does not match reservation")
            if proof and proof["payload"].get("role") == "execution_host_guard":
                from ..processes.guards import guard_phases_complete
                if not guard_phases_complete(self.state, proof, connection=conn):
                    raise JenkinsError("native_termination_unproven", "Nested phase termination is unproven", retryable=True)
            self.state.put("resource_reservation", key, {**old["payload"], "status":"released"}, expected_version=old["version"], connection=conn)
            if writer_key:
                hold = self.state.get("pool_writer_hold", writer_key, connection=conn)
                if hold and (hold["payload"].get("status") != "active" or hold["payload"].get("owner") != old["payload"].get("owner") or hold["payload"].get("reservationKey") != key):
                    raise JenkinsError("writer_owner_mismatch", "Writer hold is not owned by reservation")
                if hold and hold["payload"].get("status") == "active":
                    self.state.put("pool_writer_hold", writer_key, {**hold["payload"], "status":"released"}, expected_version=hold["version"], connection=conn)
            return True


    def query(self, key: str):
        return self.state.get("resource_reservation", key)

    def release_unlaunched(self, key: str, *, writer_key: str, execution_id: str) -> bool:
        """Release only when the durable broker intent proves nothing launched."""
        with self.state.transaction() as conn:
            reservation = self.state.get("resource_reservation", key, connection=conn)
            if not reservation or reservation["payload"].get("status") != "active":
                return False
            execution = self.state.get("execution", execution_id, connection=conn)
            launch = self.state.get("execution_launch", execution_id, connection=conn)
            host = self.state.get("execution_host", execution_id, connection=conn)
            native = [r for r in self.state.list("native_job", connection=conn)
                      if r["payload"].get("executionId") == execution_id]
            if (not execution or execution["payload"].get("status") not in {"cancelled", "failed"}
                    or reservation["payload"].get("owner") != execution_id or host or native
                    or (launch and not (launch["payload"].get("status") == "cancelled"
                                        and launch["payload"].get("cancelledBeforeClaim") is True))):
                raise JenkinsError("native_termination_unproven", "An absent or atomically cancelled launch is required")
            hold = self.state.get("pool_writer_hold", writer_key, connection=conn)
            if not hold or any(hold["payload"].get(k) != v for k, v in
                               {"owner": execution_id, "status": "active", "reservationKey": key}.items()):
                raise JenkinsError("writer_owner_mismatch", "The execution must own its preparation reservation")
            self.state.put("resource_reservation", key, {**reservation["payload"], "status": "released",
                           "releaseEvidence": "durable_unlaunched"}, expected_version=reservation["version"], connection=conn)
            self.state.put("pool_writer_hold", writer_key, {**hold["payload"], "status": "released"},
                           expected_version=hold["version"], connection=conn)
            return True

    def handle(self, action: str, payload: dict):
        if action == "scan": return self.scan_inventory(payload["buildRoot"])[0]
        if action == "admit": return self.admit(payload["owner"], Capacity(payload["cpu"],payload["memoryBytes"],payload["diskBytes"]), inventory_generation=payload["inventoryGeneration"], writer_key=payload.get("writerKey"), request_id=payload.get("requestId"), build_root=payload.get("buildRoot", ""), heavy_writer=payload.get("heavyWriter", False))
        if action == "release": return self.release(payload["key"], writer_key=payload.get("writerKey"), native_proof_ref=payload.get("nativeJobId"), execution_id=payload.get("executionId"))
        if action == "query": return self.query(payload["key"])
        raise JenkinsError("unsupported_resource_action", action)
