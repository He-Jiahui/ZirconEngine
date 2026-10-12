"""Strict joins for actual normal Editor hold, readback and desktop receipts.

This module certifies pixel association only. It does not admit business state,
source coverage, fonts, or whole managed Workbench acceptance.
"""
from __future__ import annotations

import hashlib
import json
import math
import os
import struct
import zlib
from pathlib import Path

MAX_BYTES = 24 * 1024 * 1024


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise ValueError(reason)


def uint(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and 0 <= value < 2**64


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def read_json(path: Path) -> dict:
    require(path.stat().st_size <= 8 * 1024 * 1024, "Metadata exceeds bounded size")
    value = json.loads(path.read_text(encoding="utf-8"))
    require(isinstance(value, dict), "Metadata must be an object")
    return value


def ticket_valid(value: object) -> bool:
    return (isinstance(value, dict) and
            all(uint(value.get(key)) for key in ("device_id", "generation", "sequence")) and
            value.get("queue_class") in ("Graphics", "Compute", "Copy"))


def validate_live_request(request: dict, expected: dict, at_ms: int) -> None:
    require(request == expected and request.get("release") is False and request.get("gpuReadback") is True and
            uint(request.get("requestedAtUnixMs")) and uint(request.get("deadlineMs")) and
            50 <= request["deadlineMs"] <= 10000 and
            0 <= at_ms - request["requestedAtUnixMs"] <= request["deadlineMs"],
            "Actual live capture request changed, released, expired or has invalid timing")


def validate_hold(hold: dict, nonce: str, pid: int, case_sha: str,
                  size: tuple[int, int], at_ms: int) -> None:
    require(hold.get("schema") == "dev.zircon.editor.normal-capture-hold" and hold.get("version") == 1 and
            hold.get("authority") == "normal-successful-gpu-present" and hold.get("pixelFrameVerified") is False and
            hold.get("status") == "held" and hold.get("nonce") == nonce and
            hold.get("pid") == pid and uint(hold.get("pid")) and
            hold.get("caseSha256") == case_sha and hold.get("physicalSize") == list(size) and
            isinstance(hold.get("presentationCursor"), str) and bool(hold["presentationCursor"]) and
            uint(hold.get("presentedFrameCount")) and hold["presentedFrameCount"] > 0 and
            uint(hold.get("acquiredAtUnixMs")) and hold["acquiredAtUnixMs"] <= at_ms and
            uint(hold.get("expiresAtUnixMs")) and hold["expiresAtUnixMs"] > at_ms,
            "Actual normal hold is missing, changed, untyped or expired")


def validate_readback(value: dict, hold: dict, hwnd: int, size: tuple[int, int]) -> None:
    require(value.get("schema") == "dev.zircon.editor.normal-frame-readback" and
            value.get("version") == 1 and
            value.get("authority") == "normal-actual-gpu-packet-and-readback-receipt" and
            value.get("gpuReadbackCompleted") is True and value.get("failure") is None and
            value.get("pixelFrameVerified") is False and
            uint(value.get("hwnd")) and value["hwnd"] == hwnd and hwnd > 0 and
            all(value.get(key) == hold.get(key) for key in
                ("nonce", "pid", "caseSha256", "physicalSize", "presentationCursor",
                 "presentedFrameCount", "expiresAtUnixMs")) and
            uint(value.get("surfaceEpoch")) and uint(value.get("nativePresentIndex")) and
            value.get("colorEncoding") == "srgb" and value.get("opaquePixels") is True and
            value.get("rawByteLength") == size[0] * size[1] * 4 and uint(value.get("rawByteLength")),
            "Actual GPU readback does not identify the successful held normal frame")
    row_staging_bytes = ((size[0] * 4 + 255) // 256) * 256 * size[1]
    require(0 < row_staging_bytes <= MAX_BYTES, "Actual row-padded GPU staging exceeds capture budget")
    receipt = value.get("receipt")
    require(isinstance(receipt, dict), "Actual readback receipt is absent")
    frame = receipt.get("frame_key")
    ticket = value.get("submission")
    require(ticket_valid(ticket) and isinstance(frame, dict) and
            ticket_valid(frame.get("submission")) and frame["submission"] == ticket and
            uint(frame.get("device_id")) and uint(frame.get("generation")) and
            frame["device_id"] == ticket["device_id"] and frame["generation"] == ticket["generation"] and
            uint(receipt.get("request")) and receipt.get("kind") == "Texture" and
            uint(receipt.get("byte_len")) and
            row_staging_bytes <= receipt["byte_len"] <= MAX_BYTES and
            0 <= receipt["byte_len"] - row_staging_bytes < 256 and
            receipt.get("terminal") == "Succeeded",
            "Actual successful texture receipt lacks a complete matching native submission ticket")



def json_identity_equal(left: object, right: object) -> bool:
    # Preserve JSON bool/number distinction while treating 1 and 1.0 as the
    # same JSON number, as the official TypeScript producer does.
    if isinstance(left, bool) or isinstance(right, bool):
        return type(left) is bool and type(right) is bool and left == right
    if isinstance(left, (int, float)) and isinstance(right, (int, float)):
        return math.isfinite(left) and math.isfinite(right) and left == right
    if isinstance(left, dict) and isinstance(right, dict):
        return left.keys() == right.keys() and all(
            json_identity_equal(left[key], right[key]) for key in left)
    if isinstance(left, list) and isinstance(right, list):
        return len(left) == len(right) and all(
            json_identity_equal(a, b) for a, b in zip(left, right))
    return type(left) is type(right) and left == right


def validate_case_publication(profile: dict, review_case: dict) -> None:
    data = review_case.get("data") or {}
    bound_normal = data.get("normal_window_evidence") or {}
    bound = bound_normal.get("workbenchStatePublication")
    if bound is None:
        return  # Static cases retain unknown business authority.
    expected_normal_fields = {"workbenchStatePublication"}
    if "focusPublication" in bound_normal:
        expected_normal_fields.add("focusPublication")
    require(set(bound_normal) == expected_normal_fields,
            "The hashed case must exclude per-frame capture, font, media and command evidence")
    fields = {"authority", "observed", "presentationCursor",
              "workbenchPresentation", "managedSceneFingerprint"}
    require(isinstance(bound, dict) and set(bound) == fields and
            bound.get("authority") == "normal-retained-workbench-projection" and
            bound.get("observed") is True,
            "The hashed normal case must contain a stable actual business publication")
    counters = {"structure", "geometry", "interaction", "viewport",
                "hitTest", "theme", "diagnostics"}
    cursor = bound.get("presentationCursor")
    require(isinstance(cursor, dict) and set(cursor) == counters and
            all(uint(value) and value <= 2**53 - 1 for value in cursor.values()),
            "The hashed business publication needs all seven actual safe counters")
    dto = bound.get("workbenchPresentation")
    require(isinstance(dto, dict) and isinstance(dto.get("stateFingerprint"), str) and
            len(dto["stateFingerprint"]) == 64 and
            all(char in "0123456789abcdef" for char in dto["stateFingerprint"]) and
            json_identity_equal(data.get("workbenchPresentation"), dto) and
            json_identity_equal(data.get("managedSceneFingerprint"), bound.get("managedSceneFingerprint")),
            "The hashed case business DTO and managed identities disagree")
    normal = profile.get("normal_window_evidence") or {}
    submitted = profile.get("submitted_gpu_text") or {}
    actual = normal.get("workbenchStatePublication")
    require(isinstance(actual, dict) and
            json_identity_equal({key: actual.get(key) for key in fields}, bound) and
            actual.get("authority") == "normal-retained-workbench-projection" and
            actual.get("observed") is True and
            json_identity_equal(normal.get("presentationCursor"), cursor) and
            uint(normal.get("captureSequence")) and
            normal["captureSequence"] == profile.get("capture_sequence") and
            uint(actual.get("presentedFrameCount")) and
            actual["presentedFrameCount"] > 0 and
            actual["presentedFrameCount"] == normal.get("presentedFrameCount") and
            actual["presentedFrameCount"] == submitted.get("presentedFrameCount"),
            "The held GPU frame changed the actual business DTO, source, managed scene or seven-counter cursor")
    require(("focusPublication" in bound_normal) == ("focusPublication" in normal) and
            json_identity_equal(bound_normal.get("focusPublication"), normal.get("focusPublication")),
            "Actual focus changed after case preparation; unknown is not known unfocused")


def validate_case_inputs(review_case: dict, repo_root: Path,
                         actual_project_root: Path | None) -> None:
    data = review_case.get("data") or {}
    publication = (data.get("normal_window_evidence") or {}).get("workbenchStatePublication")
    if publication is None:
        return
    require(isinstance(publication, dict) and actual_project_root is not None,
            "An actual normal business case requires its original official project launch")
    dto = publication.get("workbenchPresentation") or {}
    fingerprint = dto.get("sourceFingerprint") or {}
    owning_source = "zircon_editor/assets/ui/editor/unreal/level_editor_window.zui"
    require(set(fingerprint) == {"sourcePath", "sha256"} and
            fingerprint.get("sourcePath") == owning_source and
            review_case.get("sourcePath") == owning_source and
            digest((repo_root.resolve(strict=True) / owning_source).read_bytes()) == fingerprint.get("sha256"),
            "Current authored Unreal Level Editor owner differs from the actual case publication")
    managed = publication.get("managedSceneFingerprint") or {}
    require(set(managed) == {"projectRoot", "sources"} and
            isinstance(managed.get("projectRoot"), str),
            "Actual managed scene identity is incomplete")
    project = Path(managed["projectRoot"]).resolve(strict=True)
    # Rust canonicalize emits an extended-length Win32 spelling. Compare the
    # physical directory identity with the actual launch path.
    require(project.samefile(actual_project_root.resolve(strict=True)),
            "Actual case managed project differs from this official Editor instance")
    expected = ["zircon-project.toml", "assets/scenes/main.scene.toml",
                "assets/scenes/workbench-review-empty.scene.toml"]
    sources = managed.get("sources")
    require(isinstance(sources, list) and len(sources) == 3 and
            all(isinstance(item, dict) and set(item) == {"sourcePath", "sha256"} for item in sources) and
            [item["sourcePath"] for item in sources] == expected,
            "Actual managed case needs the exact ordered three owning inputs")
    for item in sources:
        source = project / item["sourcePath"]
        require(source.resolve(strict=True).is_relative_to(project) and
                digest(source.read_bytes()) == item["sha256"],
                "An actual managed project input changed after case preparation")

def validate_profile(profile: dict, hold: dict, size: tuple[int, int], scale: float,
                     bound_focus: object = None) -> None:
    normal = profile.get("normal_window_evidence") or {}
    submitted = profile.get("submitted_gpu_text") or {}
    require(profile.get("presenter_backend") == "gpu" and
            profile.get("winit_scale_factor") == scale and
            profile.get("window_client_size") == {"width": size[0], "height": size[1]} and
            uint(profile.get("capture_sequence")) and profile["capture_sequence"] > 0 and
            normal.get("captureHold") == hold and
            uint(normal.get("presentedFrameCount")) and
            normal.get("presentedFrameCount") == hold["presentedFrameCount"] and
            normal.get("commandStreamFullRebuild") is True and
            submitted.get("commandStreamFullRebuild") is True and
            submitted.get("coverage") == "submitted-command-stream" and
            uint(submitted.get("presentedFrameCount")) and
            submitted.get("presentedFrameCount") == hold["presentedFrameCount"],
            "Actual normal GPU/source profile does not match the complete held presentation")
    if bound_focus is not None:
        require(normal.get("focusPublication") == bound_focus,
                "Actual focus publication differs from the immutable hashed case")


def raw_rgba(value: dict, raw: bytes, size: tuple[int, int]) -> bytes:
    require(len(raw) == size[0] * size[1] * 4 <= MAX_BYTES and
            digest(raw) == value.get("rawSha256") and raw[3::4] == bytes([255]) * (size[0] * size[1]),
            "Actual raw GPU color8 bytes, extent, SHA or alpha differ")
    if value.get("sourceFormat") == "Rgba8UnormSrgb" and value.get("sourceByteLayout") == "rgba8":
        return raw
    require(value.get("sourceFormat") == "Bgra8UnormSrgb" and value.get("sourceByteLayout") == "bgra8",
            "Actual GPU format is unsupported for lossless desktop comparison")
    output = bytearray(raw)
    output[0::4] = raw[2::4]
    output[2::4] = raw[0::4]
    return bytes(output)


def decode_own_png(data: bytes, size: tuple[int, int]) -> bytes:
    """Decode the fixed RGBA8/filter-zero PNG emitted by the capture owner."""
    require(data.startswith(b"\x89PNG\r\n\x1a\n"), "Client PNG signature is invalid")
    position, payload, kinds = 8, bytearray(), []
    while position < len(data):
        require(position + 12 <= len(data), "Client PNG chunk is truncated")
        length = struct.unpack_from(">I", data, position)[0]
        kind = data[position + 4:position + 8]
        end = position + 12 + length
        require(end <= len(data), "Client PNG chunk extent is invalid")
        body = data[position + 8:position + 8 + length]
        require(zlib.crc32(kind + body) & 0xffffffff == struct.unpack_from(">I", data, end - 4)[0],
                "Client PNG CRC differs")
        kinds.append(kind)
        if kind == b"IHDR":
            require(body == struct.pack(">IIBBBBB", *size, 8, 6, 0, 0, 0), "Client PNG format differs")
        elif kind == b"IDAT":
            payload.extend(body)
        elif kind == b"IEND":
            require(length == 0 and end == len(data), "Client PNG trailing data differs")
        else:
            raise ValueError("Client PNG has an unexpected chunk")
        position = end
    require(kinds == [b"IHDR", b"IDAT", b"IEND"], "Client PNG chunk order differs")
    expected = size[1] * (1 + size[0] * 4)
    require(expected <= MAX_BYTES + size[1], "Client PNG extent exceeds capture budget")
    inflater = zlib.decompressobj()
    rows = inflater.decompress(bytes(payload), expected + 1)
    require(len(rows) == expected and inflater.eof and not inflater.unconsumed_tail and not inflater.unused_data,
            "Client PNG decoded byte extent differs")
    stride = size[0] * 4 + 1
    require(all(rows[index] == 0 for index in range(0, len(rows), stride)), "Client PNG filter differs")
    return b"".join(rows[index + 1:index + stride] for index in range(0, len(rows), stride))


def validate_capture_instance(proof: dict, capture: dict, project: dict, path: Path) -> None:
    association = capture.get("normal_pixel_association") or {}
    require(uint(capture.get("process_id")) and capture["process_id"] > 0 and
            uint(capture.get("window_handle")) and capture["window_handle"] > 0 and
            proof.get("pid") == capture["process_id"] and proof.get("hwnd") == capture["window_handle"] and
            association.get("case_sha256") == proof.get("caseSha256") and
            association.get("sha256") == digest(path.read_bytes()) and
            Path(association.get("path", "")).resolve(strict=True) == path.resolve(strict=True),
            "Pixel receipt is not bound to this actual capture process/window and case")
    case_path = Path(association["case_json_path"]).resolve(strict=True)
    require(digest(case_path.read_bytes()) == association.get("case_json_sha256") == proof.get("caseJsonSha256"),
            "Actual immutable bound case input changed")
    require(isinstance(capture.get("input_project_root"), str) and
            Path(capture["input_project_root"]).resolve(strict=True) == Path(project["project_root"]).resolve(strict=True) and
            capture.get("input_project_default_scene_uri") == project.get("default_scene_uri"),
            "This normal capture instance is not bound to the unchanged official project launch")


def validate_capture_method(value: object) -> None:
    require(value in ("win32-desktop-bitblt-gdiflush-own-client",
                      "win32-printwindow-client-renderfullcontent-gdiflush"),
            "Unsupported actual Win32 client capture method")


def validate_association(path: Path, case_sha: str, size: tuple[int, int], scale: float,
                         renderer: dict, bound_focus: object = None,
                         bound_case: dict | None = None) -> tuple[dict, Path, Path, dict]:
    proof = read_json(path)
    validate_capture_method(proof.get("captureMethod"))
    require(proof.get("schema") == "dev.zircon.editor.normal-frame-pixel-association" and
            proof.get("version") == 1 and proof.get("pixelFrameVerified") is True and
            proof.get("wholeUiAccepted") is False and
            proof.get("caseSha256") == case_sha and proof.get("physicalSize") == list(size) and
            proof.get("actualWindowDpi") == round(scale * 96) and
            proof.get("actualWinitScaleFactor") == scale,
            "Pixel receipt does not match the requested actual normal case")
    image = proof.get("processImage") or {}
    require(image.get("sha256") == renderer["actual_sha256"] and
            Path(image.get("path", "")).resolve(strict=True) == Path(renderer["path"]).resolve(strict=True) and
            digest(Path(image["path"]).read_bytes()) == image["sha256"],
            "Pixel receipt process image differs from the actual product capture")
    hold = proof.get("holdBefore") or {}
    require(hold == proof.get("holdAfter") and uint(proof.get("capturedAtUnixMs")) and
            proof.get("presentationCursor") == hold.get("presentationCursor") and
            proof.get("presentedFrameCount") == hold.get("presentedFrameCount"),
            "Pixel receipt did not observe one unchanged actual hold")
    validate_hold(hold, proof.get("nonce"), proof.get("pid"), case_sha, size, proof["capturedAtUnixMs"])
    require(uint(proof.get("hwnd")) and proof["hwnd"] > 0, "Pixel receipt HWND is missing")
    readback_path = Path(proof["normalReadbackReceiptPath"]).resolve(strict=True)
    require(readback_path.name == f"normal-gpu-{proof['nonce']}.json" and
            digest(readback_path.read_bytes()) == proof.get("normalReadbackReceiptSha256"),
            "Actual normal readback receipt changed")
    readback = read_json(readback_path)
    require(readback == proof.get("normalReadbackReceipt"), "Actual readback receipt differs from the observed copy")
    validate_readback(readback, hold, proof["hwnd"], size)
    require(readback.get("rawFile") == f"normal-gpu-{proof['nonce']}.color8", "Raw GPU path differs")
    raw_path = readback_path.parent / readback["rawFile"]
    require(raw_path.stat().st_size == size[0] * size[1] * 4 <= MAX_BYTES, "Actual GPU file exceeds byte extent")
    raw = raw_path.read_bytes()
    gpu = raw_rgba(readback, raw, size)
    profile_path = Path(proof["normalSourceProfile"]).resolve(strict=True)
    require(digest(profile_path.read_bytes()) == proof.get("normalSourceProfileSha256"),
            "Actual normal source profile changed")
    profile = read_json(profile_path)
    validate_profile(profile, hold, size, scale, bound_focus)
    if bound_case is not None:
        validate_case_publication(profile, bound_case)
    require(proof.get("focusPublication") == (profile.get("normal_window_evidence") or {}).get("focusPublication"),
            "Pixel receipt focus publication differs from the actual engine profile")
    png_path = Path(proof["clientPng"]).resolve(strict=True)
    require(png_path.stat().st_size <= MAX_BYTES + size[1] + 65536, "Actual client PNG exceeds byte budget")
    png = png_path.read_bytes()
    require(digest(png) == proof.get("clientPngSha256") and digest(raw) == proof.get("rawGpuSha256"),
            "Actual pixel files changed")
    desktop = decode_own_png(png, size)
    require(desktop == gpu and digest(desktop) == proof.get("decodedRgba8Sha256"),
            "Actual desktop PNG differs from its same-frame GPU bytes")
    return proof, png_path, profile_path, profile

def physical_file(path: Path) -> Path:
    absolute = Path(os.path.abspath(path))
    for ancestor in (absolute, *absolute.parents):
        info = ancestor.lstat()
        if ancestor.is_symlink() or getattr(info, "st_file_attributes", 0) & 1024:
            raise ValueError(f"Contract input contains a reparse alias: {ancestor}")
    physical = absolute.resolve(strict=True)
    if os.path.normcase(str(physical)) != os.path.normcase(str(absolute)) or not physical.is_file():
        raise ValueError(f"Contract input is not its physical regular file: {path}")
    return physical


def file_sha(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def validate_case_contract_documents(configuration: dict, audit: dict, repo: Path) -> None:
    require(configuration.get("schema") == "dev.zircon.editor.normal-frame-case-producer"
            and configuration.get("version") == 1,
            "Unsupported actual case-producer configuration")
    for key in ("node", "producer", "contract", "contractInputs"):
        item = configuration.get(key)
        require(isinstance(item, dict) and isinstance(item.get("path"), str)
                and isinstance(item.get("sha256"), str) and len(item["sha256"]) == 64
                and all(c in "0123456789abcdef" for c in item["sha256"]),
                f"Missing immutable case contract identity: {key}")
    def key(path):
        return os.path.normcase(os.path.abspath(path))
    canonical = repo / "dev/penpot/plugins/apps/zircon-zui-plugin"
    require(audit.get("schema") == "dev.zircon.editor.normal-workbench-contract-inputs"
            and audit.get("version") == 1
            and key(audit.get("repoRoot", "")) == key(repo)
            and key(audit.get("canonicalAppRoot", "")) == key(canonical)
            and audit.get("sourceCopiesCreated") is False
            and key(audit.get("outputPath", "")) == key(configuration["contract"]["path"])
            and audit.get("outputSha256") == configuration["contract"]["sha256"],
            "Compiled contract does not belong to current canonical Main sources")
    rows = audit.get("inputs")
    require(isinstance(rows, list) and bool(rows), "Compiled contract input closure is empty")
    paths = set()
    plugin_root = repo / "dev/penpot/plugins"
    for row in rows:
        require(isinstance(row, dict) and isinstance(row.get("path"), str)
                and isinstance(row.get("sha256"), str) and len(row["sha256"]) == 64
                and all(c in "0123456789abcdef" for c in row["sha256"]),
                "Compiled contract input identity is malformed")
        path = key(row["path"])
        require(path not in paths, "Compiled contract input closure contains duplicate paths")
        require(os.path.commonpath([path, key(plugin_root)]) == key(plugin_root),
                "Compiled contract input is outside the canonical plugin tree")
        paths.add(path)
    for required in ("tools/zui-layout-review-contract.ts",
                     "tools/zui-layout-workbench-presentation.ts",
                     "tools/zui-layout-workbench-managed-inputs.ts",
                     "tools/zui-layout-component-hosts.ts"):
        require(key(canonical / required) in paths, f"Required canonical contract owner is missing: {required}")


def load_case_contract_configuration(path: Path, expected_sha: str, repo: Path) -> dict:
    def artifact_file(value: Path) -> Path:
        physical = physical_file(value)
        for drive in ("D", "E", "F"):
            base = Path(f"{drive}:/cargo-targets/zircon-local")
            if base.exists():
                try:
                    if os.path.normcase(os.path.commonpath([physical, base.resolve(strict=True)])) == os.path.normcase(str(base.resolve(strict=True))):
                        return physical
                except ValueError:
                    continue
        raise ValueError(f"Case contract is outside a physical approved artifact root: {physical}")
    path = artifact_file(path)
    require(file_sha(path) == expected_sha, "Case-producer configuration changed")
    configuration = read_json(path)
    pinned = {}
    for label in ("node", "producer", "contract", "contractInputs"):
        item = configuration.get(label)
        require(isinstance(item, dict) and isinstance(item.get("path"), str), f"Missing case contract: {label}")
        physical = artifact_file(Path(item["path"]))
        require(file_sha(physical) == item.get("sha256"), f"Actual case contract changed: {label}")
        pinned[label] = physical
    require(pinned["producer"].parent == path.parent == pinned["contract"].parent == pinned["contractInputs"].parent,
            "Case contract artifacts must share their reviewed producer directory")
    audit = read_json(pinned["contractInputs"])
    validate_case_contract_documents(configuration, audit, repo)
    for row in audit["inputs"]:
        source = physical_file(Path(row["path"]))
        require(file_sha(source) == row["sha256"], f"Compiled contract no longer matches current Main source: {source}")
    return {"configurationPath": str(path), "configurationSha256": expected_sha,
            "nodePath": str(pinned["node"]), "nodeSha256": configuration["node"]["sha256"],
            "modulePath": str(pinned["contract"]), "moduleSha256": configuration["contract"]["sha256"],
            "inputAuditPath": str(pinned["contractInputs"]), "inputAuditSha256": configuration["contractInputs"]["sha256"],
            "inputCount": len(audit["inputs"])}
