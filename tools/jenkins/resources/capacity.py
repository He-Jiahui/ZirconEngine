"""Atomic in-process capacity ledger used by the state adapter."""
from __future__ import annotations
from dataclasses import dataclass
from threading import Lock
from uuid import uuid4
from ..contracts import JenkinsError

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

    def queue(self, owner: str, *, priority: int = 0, request_id: str | None = None, amount: Capacity | None = None):
        import time
        key=f"{owner}:{request_id or uuid4().hex}"
        with self.state.transaction() as conn:
            old=self.state.get("resource_queue", key, connection=conn)
            if old and old["payload"].get("amount") != (amount.__dict__ if amount else None):
                raise JenkinsError("request_payload_mismatch", "Queued resource recipe changed")
            return self.state.put("resource_queue", key, {"owner":owner,"priority":priority,"queuedAt":old["payload"].get("queuedAt",time.time()) if old else time.time(),"status":old["payload"].get("status","waiting") if old else "waiting", "amount": amount.__dict__ if amount else None}, expected_version=old["version"] if old else None, connection=conn)

    def fair_queue(self, *, connection=None):
        import time
        rows=[r for r in self.state.list("resource_queue", connection=connection) if r["payload"].get("status")=="waiting"]
        return sorted(rows, key=lambda r: (-(r["payload"].get("priority",0)+(time.time()-r["payload"].get("queuedAt",time.time()))/60), r["payload"].get("queuedAt",0)))

    def publish_inventory(self, generation: str, capacity: Capacity, *, complete: bool = True, observed_at: float | None = None, build_root: str = ""):
        if not generation or not complete: raise JenkinsError("storage_snapshot_stale", "Inventory must be complete")
        with self.state.transaction() as conn:
            import time
            return self.state.put("capacity_inventory", generation, {"generation":generation,"cpu":capacity.cpu,"memoryBytes":capacity.memory_bytes,"diskBytes":capacity.disk_bytes,"complete":True,"observedAt":observed_at or time.time(),"buildRoot":build_root}, connection=conn)

    def scan_inventory(self, build_root: str):
        """Capture a fresh local snapshot; callers may apply policy limits later."""
        import os, time, multiprocessing
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
        return self.publish_inventory(generation, capacity, build_root=str(build_root)), generation

    def admit(self, owner: str, amount: Capacity, *, inventory_generation: str = "", writer_key: str | None = None, max_age_seconds: float = 30.0, priority: int = 0, request_id: str | None = None, build_root: str = "", heavy_writer: bool = False):
        if min(amount.cpu, amount.memory_bytes, amount.disk_bytes) < 0:
            raise JenkinsError("invalid_reservation", "Reservation cannot be negative")
        if heavy_writer and min(amount.cpu, amount.memory_bytes, amount.disk_bytes) <= 0:
            raise JenkinsError("invalid_reservation", "Heavy reservation requires positive estimates")
        key = f"{owner}:{request_id or uuid4().hex}"
        with self.state.transaction() as conn:
            inv = self.state.get("capacity_inventory", inventory_generation, connection=conn) if inventory_generation else None
            import time
            if not inv or not inv["payload"].get("complete") or time.time()-inv["payload"].get("observedAt",0) > max_age_seconds:
                raise JenkinsError("storage_snapshot_stale", "Complete capacity inventory is required", retryable=True)
            policy_record = self.state.get("resource_policy", "default", connection=conn)
            requested_root = str(policy_record["payload"].get("buildRoot", "")) if policy_record else ""
            allowed = policy_record['payload'].get('allowedBuildRoots') if policy_record else None
            if allowed and str(build_root).replace('/', '\\').casefold() not in {str(root).replace('/', '\\').casefold() for root in allowed}:
                raise JenkinsError('build_root_not_current', 'New reservations must use the current configured build root')
            if build_root and inv["payload"].get("buildRoot") != build_root:
                raise JenkinsError("inventory_root_mismatch", "Inventory belongs to a different build root", retryable=True)
            existing = self.state.get("resource_reservation", key, connection=conn)
            if existing:
                if any(existing["payload"].get(k) != v for k,v in {"owner":owner,"cpu":amount.cpu,"memoryBytes":amount.memory_bytes,"diskBytes":amount.disk_bytes,"priority":priority,"buildRoot":build_root,"heavyWriter":heavy_writer}.items()): raise JenkinsError("request_payload_mismatch", "Reservation request changed")
                if existing["payload"].get("status") == "active": return existing
            waiting=self.fair_queue(connection=conn)
            if waiting and waiting[0]["key"] != key:
                return None
            if writer_key:
                hold = self.state.get("pool_writer_hold", writer_key, connection=conn)
                if hold and hold["payload"].get("status") == "active":
                    raise JenkinsError("pool_writer_active", "Pool writer is already held", retryable=True)
            rows = self.state.list("resource_reservation", connection=conn)
            used = Capacity(0, 0, 0)
            for row in rows:
                p = row["payload"]
                if p.get("status") == "active":
                    used = Capacity(used.cpu+p["cpu"], used.memory_bytes+p["memoryBytes"], used.disk_bytes+p["diskBytes"])
            candidate = Capacity(used.cpu+amount.cpu, used.memory_bytes+amount.memory_bytes, used.disk_bytes+amount.disk_bytes)
            available = inv["payload"]
            floor=int(policy_record["payload"].get("diskReserveBytes",35*1024**3)) if policy_record else 35*1024**3
            limits = Capacity(min(self.total.cpu, int(available["cpu"])), min(self.total.memory_bytes, int(available["memoryBytes"])), min(self.total.disk_bytes, max(0, int(available["diskBytes"]) - floor)))
            if heavy_writer:
                held=sum(1 for r in self.state.list("resource_reservation", connection=conn) if r["payload"].get("status")=="active" and r["payload"].get("heavyWriter"))
                max_heavy=int(policy_record["payload"].get("maxHeavyWriters",1)) if policy_record else 1
                if held >= max_heavy: return None
            if candidate.cpu > limits.cpu or candidate.memory_bytes > limits.memory_bytes or candidate.disk_bytes > limits.disk_bytes:
                return None
            if writer_key:
                self.state.put("pool_writer_hold", writer_key, {"owner":owner,"status":"active","generation":inventory_generation,"reservationKey":key}, connection=conn)
            result=self.state.put("resource_reservation", key, {"owner":owner,"cpu":amount.cpu,"memoryBytes":amount.memory_bytes,"diskBytes":amount.disk_bytes,"inventoryGeneration":inventory_generation,"status":"active","priority":priority,"buildRoot":build_root,"heavyWriter":heavy_writer}, connection=conn)
            queued=self.state.get("resource_queue", key, connection=conn)
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
