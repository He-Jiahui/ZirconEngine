"""Strict joins for actual normal Editor hold, readback and desktop receipts.

This module certifies pixel association only. It does not admit business state,
source coverage, fonts, or whole managed Workbench acceptance.
"""
from __future__ import annotations

import hashlib
import base64
import json
import math
import os
import struct
import zlib
from pathlib import Path

MAX_BYTES = 24 * 1024 * 1024
CASE_MAX_BYTES = 8 * 1024 * 1024
SCENE_SCHEMA = "dev.zircon.editor.runtime-scene-media"
SCENE_SOURCE = "res://ui/editor/unreal/components/viewport.zui"


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
    full_case = any(key in bound_normal for key in
                    ("presentationCursor", "presentedFrameCount", "viewportToolbarPublication",
                     "designTokenInput", "arrangedInventoryPublication", "commands"))
    if bound is None and not full_case:
        return  # Static cases retain unknown business authority.
    expected_normal_fields = {"workbenchStatePublication"}
    if full_case:
        expected_normal_fields.update(("presentationCursor", "presentedFrameCount", "viewportToolbarPublication"))
        if "semanticNodes" in bound_normal:
            expected_normal_fields.add("semanticNodes")
        expected_normal_fields.update(
            key for key in ("designTokenInput", "arrangedInventoryPublication", "commands")
            if key in bound_normal)
    if "focusPublication" in bound_normal:
        expected_normal_fields.add("focusPublication")
    if "runtimeSceneMedia" in bound_normal:
        expected_normal_fields.add("runtimeSceneMedia")
    require(set(bound_normal) == expected_normal_fields,
            "The hashed case needs an exact legacy publication or a complete canonical seed publication")
    fields = {"authority", "observed", "presentationCursor",
              "workbenchPresentation", "managedSceneFingerprint"}
    bound_fields = fields | ({"presentedFrameCount"} if full_case else set())
    require(isinstance(bound, dict) and set(bound) == bound_fields and
            bound.get("authority") == "normal-retained-workbench-projection" and
            bound.get("observed") is True,
            "The hashed normal case must contain a complete actual business publication")
    counters = {"structure", "geometry", "interaction", "viewport",
                "hitTest", "theme", "diagnostics"}
    cursor = bound.get("presentationCursor")
    require(isinstance(cursor, dict) and set(cursor) == counters and
            all(uint(value) and value <= 2**53 - 1 for value in cursor.values()),
            "The hashed business publication needs all seven actual safe counters")
    if full_case:
        seed_toolbar = bound_normal["viewportToolbarPublication"]
        seed_count = bound["presentedFrameCount"]
        require(isinstance(seed_toolbar, dict) and
                json_identity_equal(bound_normal["presentationCursor"], cursor) and
                json_identity_equal(seed_toolbar.get("presentationCursor"), cursor) and
                all(uint(value) and 0 < value <= 2**53 - 1 for value in
                    (seed_count, bound_normal["presentedFrameCount"], seed_toolbar.get("presentedFrameCount"))) and
                seed_count == bound_normal["presentedFrameCount"] == seed_toolbar["presentedFrameCount"],
                "The canonical seed toolbar, business and normal publications must identify one successful frame")
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
            json_identity_equal({key: actual.get(key) for key in fields}, {key: bound[key] for key in fields}) and
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
    if full_case:
        actual_toolbar = normal.get("viewportToolbarPublication")
        require(isinstance(actual_toolbar, dict) and
                json_identity_equal(actual_toolbar.get("presentationCursor"), cursor) and
                uint(actual_toolbar.get("presentedFrameCount")) and
                actual_toolbar["presentedFrameCount"] == actual["presentedFrameCount"],
                "The held source toolbar does not belong to the held business and GPU frame")
        # The case SHA binds an immutable seed observation. A later held present
        # may advance only its frame count; every source/state/geometry field stays exact.
        # The canonical producer/receipt chain separately validates the full toolbar
        # schema and pins the observed seed profile, capture sequence and case SHA.
        require(json_identity_equal(
                    {key: value for key, value in seed_toolbar.items() if key != "presentedFrameCount"},
                    {key: value for key, value in actual_toolbar.items() if key != "presentedFrameCount"}),
                "The held toolbar source, state, bounds, desired size or cursor changed after case preparation")
        require(("semanticNodes" in bound_normal) == ("semanticNodes" in normal) and
                json_identity_equal(bound_normal.get("semanticNodes"), normal.get("semanticNodes")),
                "Actual semantic nodes changed after case preparation")
        if "commands" in bound_normal:
            observed_submitted_commands(bound_normal)
            observed_submitted_commands(normal)
            require(json_identity_equal(bound_normal["commands"], normal["commands"]),
                    "Actual submitted commands changed after case preparation")
        # Bind observations from the seed and the held normal present without
        # promoting a paired null or a pending inventory into readiness.
        paint_for_token_input = normal.get("paintEvidence")
        for publication_key, publication_owner in (
            ("designTokenInput", paint_for_token_input),
            ("arrangedInventoryPublication", normal),
        ):
            seed_has_publication = publication_key in bound_normal
            actual_has_publication = (
                isinstance(publication_owner, dict)
                and publication_key in publication_owner
            )
            require(seed_has_publication == actual_has_publication,
                    f"{publication_key} presence differs between seed and actual")
            if not seed_has_publication:
                continue
            seed_publication = bound_normal[publication_key]
            actual_publication = publication_owner[publication_key]
            if publication_key == "designTokenInput" or (
                seed_publication is None and actual_publication is None
            ):
                require(json_identity_equal(seed_publication, actual_publication),
                        f"{publication_key} differs between seed and actual")
                continue
            require(isinstance(seed_publication, dict)
                    and isinstance(actual_publication, dict),
                    "Arranged inventory must be an object or paired null")
            for inventory, outer in (
                (seed_publication, bound_normal),
                (actual_publication, normal),
            ):
                inventory_count = inventory.get("presentedFrameCount")
                outer_count = outer.get("presentedFrameCount")
                require(all(type(value) is int and 0 < value <= 2**53 - 1
                            for value in (inventory_count, outer_count)),
                        "Arranged inventory requires safe positive frame counts")
                require(inventory_count == outer_count,
                        "Arranged inventory frame count differs from its normal frame")
                require(json_identity_equal(inventory.get("presentationCursor"),
                                            outer.get("presentationCursor")),
                        "Arranged inventory cursor differs from its normal frame")
            require(json_identity_equal(
                {key: value for key, value in seed_publication.items()
                 if key != "presentedFrameCount"},
                {key: value for key, value in actual_publication.items()
                 if key != "presentedFrameCount"}),
                "Arranged inventory changed after case preparation")
    require(("focusPublication" in bound_normal) == ("focusPublication" in normal) and
            json_identity_equal(bound_normal.get("focusPublication"), normal.get("focusPublication")),
            "Actual focus changed after case preparation; unknown is not known unfocused")
    if "runtimeSceneMedia" in bound_normal:
        packet = runtime_scene_packet(review_case)
        scene = packet["bootstrapPixelAssociation"]["observed"]["normalReadbackReceipt"]["sceneImage"]
        validate_scene_command(scene, profile)


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


def decode_scene_png(data: bytes, size: tuple[int, int]) -> bytes:
    """Bounded, independent decoder for official noninterlaced RGBA8 scene PNGs.

    The image crate may choose any of the five row filters and split IDAT.
    This does not relax the separate desktop capture owner's fixed PNG format.
    """
    width, height = size
    require(all(type(x) is int and 0 < x <= 8192 for x in size) and width * height * 4 <= MAX_BYTES and
            len(data) <= CASE_MAX_BYTES and data.startswith(b"\x89PNG\r\n\x1a\n"), "Scene PNG extent or signature is invalid")
    offset, compressed, kinds, ended_idat = 8, bytearray(), [], False
    while offset < len(data):
        require(len(kinds) < 1024 and offset + 12 <= len(data), "Scene PNG chunk count or extent is invalid")
        length = struct.unpack_from(">I", data, offset)[0]
        kind, end = data[offset + 4:offset + 8], offset + length + 12
        require(end <= len(data) and all(65 <= x <= 90 or 97 <= x <= 122 for x in kind) and not kind[2] & 32,
                "Scene PNG chunk is invalid")
        body = data[offset + 8:end - 4]
        require(zlib.crc32(kind + body) & 0xffffffff == struct.unpack_from(">I", data, end - 4)[0], "Scene PNG CRC differs")
        if not kinds:
            require(kind == b"IHDR" and body == struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0), "Scene PNG format differs")
        elif kind == b"IHDR":
            raise ValueError("Scene PNG has duplicate IHDR")
        elif kind == b"IDAT":
            require(not ended_idat, "Scene PNG IDAT chunks are not contiguous")
            compressed.extend(body)
        elif kind == b"IEND":
            require(length == 0 and end == len(data) and b"IDAT" in kinds, "Scene PNG end or trailing bytes differ")
        else:
            require(kind[0] & 32, "Scene PNG has an unsupported critical chunk")
            if b"IDAT" in kinds:
                ended_idat = True
            # Do not reinterpret the explicitly sRGB encoder input via an ICC,
            # HDR or contradictory transfer declaration supplied by a file.
            require(kind not in (b"iCCP", b"cICP", b"mDCV", b"cLLI", b"tRNS", b"acTL", b"fcTL", b"fdAT"), "Scene PNG has an unsupported color/animation profile")
            if kind == b"gAMA":
                require(body == struct.pack(">I", 45455) and b"IDAT" not in kinds and kind not in kinds,
                        "Scene PNG transfer declaration differs")
            if kind == b"sRGB":
                require(len(body) == 1 and body[0] <= 3 and b"IDAT" not in kinds and kind not in kinds,
                        "Scene PNG sRGB declaration differs")
            if kind == b"cHRM":
                require(body == struct.pack(">8I", 31270,32900,64000,33000,30000,60000,15000,6000) and b"IDAT" not in kinds and kind not in kinds,
                        "Scene PNG primaries differ")
        kinds.append(kind); offset = end
    require(kinds and kinds[-1] == b"IEND", "Scene PNG is incomplete")
    stride, expected = width * 4, height * (width * 4 + 1)
    inflater = zlib.decompressobj()
    filtered = inflater.decompress(bytes(compressed), expected + 1)
    require(len(filtered) == expected and inflater.eof and not inflater.unconsumed_tail and not inflater.unused_data,
            "Scene PNG inflated extent differs")
    output = bytearray(width * height * 4)
    for row in range(height):
        method = filtered[row * (stride + 1)]
        require(method <= 4, "Scene PNG row filter is invalid")
        for column in range(stride):
            left = output[row * stride + column - 4] if column >= 4 else 0
            up = output[(row - 1) * stride + column] if row else 0
            diagonal = output[(row - 1) * stride + column - 4] if row and column >= 4 else 0
            if method == 0: predictor = 0
            elif method == 1: predictor = left
            elif method == 2: predictor = up
            elif method == 3: predictor = (left + up) // 2
            else:
                estimate = left + up - diagonal
                distances = (abs(estimate - left), abs(estimate - up), abs(estimate - diagonal))
                predictor = (left, up, diagonal)[distances.index(min(distances))]
            output[row * stride + column] = (filtered[row * (stride + 1) + 1 + column] + predictor) & 255
    return bytes(output)


def scene_source_alias(value: str) -> str:
    return "zircon_editor/assets/" + value[6:] if value.startswith("res://ui/editor/") else value


def scene_source_path(value: object) -> str:
    require(isinstance(value, str) and bool(value.strip()), "Scene source path is incomplete")
    path = scene_source_alias(value)
    require("\\" not in path and not any(part in (".", "..") for part in path.split("/")), "Invalid Scene source alias")
    return path


def source_tuple(value: object) -> tuple:
    require(isinstance(value, dict) and all(isinstance(value.get(key), str) and value[key].strip()
            for key in ("sourcePath", "sourceNodeId")), "Scene source owner is incomplete")
    path = value["instancePath"] if "instancePath" in value else None
    if isinstance(path, str):
        path = json.loads(path)
    require(isinstance(path, list) and len(path) <= 64 and all(
        isinstance(step, dict) and set(step) == {"sourcePath", "sourceNodeId"} and
        all(isinstance(step[key], str) and step[key].strip() for key in step) for step in path),
        "Scene invocation identity is incomplete")
    steps = [{"sourcePath": scene_source_path(step["sourcePath"]), "sourceNodeId": step["sourceNodeId"]} for step in path]
    return scene_source_path(value["sourcePath"]), value["sourceNodeId"], json.dumps(steps, sort_keys=True, separators=(",", ":"))


def observed_submitted_commands(normal: dict) -> list:
    commands = normal.get("commands")
    require(isinstance(commands, list), "Actual submitted normal commands are absent")
    indices = set()
    for command in commands:
        index = command.get("commandIndex") if isinstance(command, dict) else None
        require(uint(index) and index <= 2**53 - 1 and index not in indices,
                "Actual normal command indices are incomplete or duplicated")
        indices.add(index)
    return commands


def validate_scene_command(scene: dict, profile: dict, *, frame_local: bool = False) -> dict:
    require(isinstance(scene, dict) and scene.get("schema") == "dev.zircon.editor.normal-scene-image-readback" and
            type(scene.get("version")) is int and scene["version"] == 1 and scene.get("authority") == "normal-actual-submitted-runtime-image-readback" and
            scene.get("paneKind") == "Scene" and isinstance(scene.get("surfaceKey"), str) and bool(scene["surfaceKey"]) and
            "atlasUv" in scene and scene["atlasUv"] is None and type(scene.get("overlayPresent")) is bool and
            all(isinstance(scene.get(key), dict) and set(scene[key]) == {"sourcePath","sourceNodeId","instancePath"} and
                isinstance(scene[key].get("instancePath"), str) for key in ("owner","viewportRoot")),
            "Scene packet is not an exact submitted runtime image")
    reference = scene.get("sourceRef")
    require(isinstance(reference, dict) and set(reference) == {"authority", "primitiveIdentity", "treeId", "generation", "nodeId", "nodeCommandIndex", "fragmentIndex"} and
            reference.get("authority") == "surface-node-owner" and reference.get("primitiveIdentity") is False and
            isinstance(reference.get("treeId"), str) and bool(reference["treeId"]) and
            all(uint(reference.get(key)) for key in ("generation", "nodeId", "nodeCommandIndex", "fragmentIndex")) and
            uint(scene.get("commandIndex")) and all(isinstance(scene.get(key), dict) and set(scene[key]) == {"x", "y", "width", "height"} and
                all(type(value) in (int, float) and math.isfinite(value) for value in scene[key].values()) and
                scene[key]["width"] > 0 and scene[key]["height"] > 0 for key in ("commandBounds", "commandClip")),
            "Scene source reference or actual command rectangles are incomplete")
    owner, root = source_tuple(scene.get("owner")), source_tuple(scene.get("viewportRoot"))
    viewport = (owner[0], "viewport", owner[2])
    require(owner[:2] == (scene_source_path(SCENE_SOURCE), "scene_media") and
            root[:2] == (scene_source_path(SCENE_SOURCE), "root") and owner[2] == root[2],
            "Scene image has a different authored viewport owner")
    normal = profile.get("normal_window_evidence") or {}
    # Non-scene brushes may lack Source. A matching path/node must still provide
    # a complete strict invocation, including when another valid target exists.
    matches = [command for command in observed_submitted_commands(normal) if
               isinstance(command, dict) and (command.get("paint") or {}).get("kind") == "image" and
               isinstance(command.get("source"), dict) and command["source"].get("sourceNodeId") == owner[1] and
               isinstance(command["source"].get("sourcePath"), str) and scene_source_alias(command["source"]["sourcePath"]) == owner[0] and
               source_tuple(command["source"]) == owner]
    require(len(matches) == 1, "Scene needs one actual source-owned submitted image command")
    command, payload = matches[0], matches[0]["paint"]
    require(payload.get("residentAfterSuccessfulPresent") is True and payload.get("hasUploadBytes") is False and
            payload.get("resourceKey") == scene.get("resourceKey") and uint(scene.get("resourceGeneration")) and
            payload.get("generation") == scene["resourceGeneration"] and
            scene.get("physicalSize") == [payload.get("width"), payload.get("height")],
            "Scene submitted image resource identity, size or residency differs")
    source = command["source"]
    require(source.get("controlId") == "WorkbenchViewportSurface" and source_tuple({
            "sourcePath": source.get("parentSourcePath"), "sourceNodeId": source.get("parentSourceNodeId"),
            "instancePath": source.get("parentInstancePath")}) == viewport,
            "Scene submitted image has a different viewport parent")
    if frame_local:
        require(uint(scene.get("commandIndex")) and scene["commandIndex"] == command.get("commandIndex") and
                json_identity_equal(scene.get("sourceRef"), source.get("sourceRef")) and
                json_identity_equal(scene.get("commandBounds"), command.get("bounds")) and
                json_identity_equal(scene.get("commandClip"), command.get("clip")),
                "Scene receipt does not identify its actual frame-local command")
    inventory = normal.get("arrangedInventoryPublication") or {}
    require(inventory.get("schema") == "dev.zircon.editor.normal-arranged-inventory" and inventory.get("version") == 1 and
            inventory.get("authority") == "normal-retained-runtime-surfaces" and inventory.get("observed") is True and
            type(inventory.get("complete")) is bool and json_identity_equal(inventory.get("presentationCursor"), normal.get("presentationCursor")) and
            type(inventory.get("presentedFrameCount")) is int and inventory["presentedFrameCount"] == normal.get("presentedFrameCount"),
            "Scene owner inventory belongs to another normal frame")
    surfaces = [value for value in inventory.get("surfaces") or [] if isinstance(value, dict) and value.get("scope") == "workbench-window"]
    require(len(surfaces) == 1 and surfaces[0].get("documentId") == "res://ui/editor/unreal/level_editor_window.zui",
            "Scene has no unique actual Workbench owner surface")
    surface, by_id, surface_ids = surfaces[0], {}, set()
    require(surface.get("schema") == "dev.zircon.editor.normal-surface-arranged-inventory" and surface.get("version") == 1 and
            surface.get("coordinateSpace") == "surface-logical" and surface.get("frameAuthority") == "retained-host-projection" and
            uint(surface.get("sourceFrameGeneration")), "Scene surface schema or source frame is incomplete")
    for row in surface.get("nodes") or []:
        require(isinstance(row, dict) and isinstance(row.get("nodeId"), str) and row["nodeId"] not in by_id, "Scene inventory has duplicate runtime IDs")
        actual_id = row.get("surfaceNodeId")
        if actual_id is not None:
            require(uint(actual_id) and actual_id > 0 and actual_id not in surface_ids, "Scene inventory has invalid or duplicate actual source IDs")
            surface_ids.add(actual_id)
        by_id[row["nodeId"]] = row
    roots = [row for row in by_id.values() if row.get("controlId") == "WorkbenchWindowRoot"]
    def scene_rows(identity):
        return [row for row in by_id.values() if row.get("sourceNodeId") == identity[1] and isinstance(row.get("sourcePath"), str) and
                scene_source_path(row.get("sourcePath")) == identity[0] and source_tuple(row) == identity]
    matches, authored_roots, viewports = scene_rows(owner), scene_rows(root), scene_rows(viewport)
    reference = command["source"].get("sourceRef")
    require(len(roots) == len(matches) == len(authored_roots) == len(viewports) == 1 and
            matches[0].get("component") == "SceneViewViewport" and matches[0].get("controlId") == "WorkbenchViewportSurface" and
            authored_roots[0].get("component") == "VerticalGroup" and authored_roots[0].get("controlId") == "WorkbenchMainBandViewportPanel" and
            viewports[0].get("component") == "Overlay" and viewports[0].get("controlId") == "UnrealSceneViewViewport" and
            roots[0].get("parent") is None and roots[0].get("component") == "Overlay" and
            source_tuple(roots[0])[:2] == (scene_source_path("res://ui/editor/unreal/level_editor_window.zui"), "root") and
            uint(roots[0].get("surfaceNodeId")) and roots[0]["surfaceNodeId"] > 0 and surface.get("rootIds") == [roots[0]["surfaceNodeId"]] and
            isinstance(reference, dict) and set(reference) == {"authority", "primitiveIdentity", "treeId", "generation", "nodeId", "nodeCommandIndex", "fragmentIndex"} and
            reference.get("authority") == "surface-node-owner" and reference.get("primitiveIdentity") is False and
            reference.get("treeId") == "template.v2." + surface["documentId"] and
            all(uint(reference.get(key)) and reference[key] <= 2**53 - 1 for key in ("generation", "nodeId", "nodeCommandIndex", "fragmentIndex")) and
            reference["nodeId"] == matches[0].get("surfaceNodeId") and 0 < reference["generation"] <= surface["sourceFrameGeneration"] and
            isinstance(matches[0].get("parent"), dict) and
            source.get("parentNodeId") == matches[0]["parent"].get("nodeId"),
            "Scene command lacks its exact actual surface owner")
    row, visited = matches[0], set()
    root_steps = json.loads(source_tuple(roots[0])[2])
    require(json.loads(owner[2]) == root_steps + [{"sourcePath": scene_source_path(surface["documentId"]), "sourceNodeId": "viewport"}],
            "Scene invocation is not the actual Workbench viewport mount")
    require(source_tuple(row.get("parent")) == viewport and by_id.get(row["parent"].get("nodeId")) is viewports[0] and
            source_tuple(viewports[0].get("parent")) == root and by_id.get(viewports[0]["parent"].get("nodeId")) is authored_roots[0],
            "Actual scene root, viewport and media ancestry differs")
    while True:
        require(row["nodeId"] not in visited and uint(row.get("surfaceNodeId")) and row["surfaceNodeId"] > 0, "Scene owner ancestry is incomplete")
        visited.add(row["nodeId"])
        require(json.loads(source_tuple(row)[2])[:len(root_steps)] == root_steps, "Scene ancestry belongs to another full owner invocation")
        projected = row.get("projected") or {}
        require(projected.get("nodeId") == row["nodeId"] and projected.get("surfaceNodeId") == row["surfaceNodeId"] and
                projected.get("component") == row.get("component") and projected.get("controlId") == row.get("controlId") and
                source_tuple(projected) == source_tuple(row), "Scene actual source projection differs")
        if row is roots[0]:
            require(projected.get("parentId") is None, "Actual scene root was rebound")
            break
        parent = row.get("parent") or {}
        actual_parent = by_id.get(parent.get("nodeId"))
        require(isinstance(actual_parent, dict) and projected.get("parentId") == actual_parent["nodeId"] and
                source_tuple(parent) == source_tuple(actual_parent), "Actual scene owner parent differs")
        row = actual_parent
    return command


def scene_rgba(scene: dict, raw: bytes) -> bytes:
    size = scene.get("physicalSize")
    require(isinstance(size, list) and len(size) == 2 and all(type(x) is int and 0 < x <= 8192 for x in size) and
            len(raw) == size[0] * size[1] * 4 <= MAX_BYTES and scene.get("rawRowBytes") == size[0] * 4 and
            scene.get("rawByteLength") == len(raw) and digest(raw) == scene.get("rawSha256") and
            scene.get("readbackCompleted") is True and scene.get("failure") is None and scene.get("alphaMode") == "opaque" and
            scene.get("opaquePixels") is True and raw[3::4] == b"\xff" * (size[0] * size[1]) and
            scene.get("pngColorEncoding") == "srgb", "Scene raw bytes, alpha, completion or row layout differ")
    formats = {"Rgba8UnormSrgb": ("rgba8", "srgb"), "Bgra8UnormSrgb": ("bgra8", "srgb"),
               "Rgba8Unorm": ("rgba8", "linear"), "Bgra8Unorm": ("bgra8", "linear")}
    expected = formats.get(scene.get("sourceFormat"))
    require(expected is not None and expected == (scene.get("sourceByteLayout"), scene.get("colorEncoding")),
            "Scene GPU view format and transfer interpretation differ")
    rgba = bytearray(raw)
    if expected[0] == "bgra8":
        rgba[0::4], rgba[2::4] = raw[2::4], raw[0::4]
    algorithm = "identity-rgba8-srgb" if expected[0] == "rgba8" else "bgra8-srgb-to-rgba8-srgb"
    if expected[1] == "linear":
        lut = [math.floor(255 * (12.92 * (x / 255) if x / 255 <= 0.0031308
               else 1.055 * (x / 255) ** (1 / 2.4) - 0.055) + 0.5) for x in range(256)]
        for channel in range(3): rgba[channel::4] = bytes(lut[x] for x in rgba[channel::4])
        algorithm = "linear8-to-rgba8-srgb-iec61966-2-1"
    require(scene.get("conversion") == {"algorithm": algorithm, "inputSha256": digest(raw), "outputSha256": digest(rgba)} and
            scene.get("rgba8Sha256") == digest(rgba), "Scene conversion evidence differs from its actual GPU bytes")
    return bytes(rgba)


def validate_scene_image(readback: dict, profile: dict, directory: Path) -> dict:
    scene = readback.get("sceneImage")
    require(isinstance(scene, dict), "Actual scene image readback is unavailable")
    command = validate_scene_command(scene, profile, frame_local=True)
    nonce = readback.get("nonce")
    require(isinstance(nonce, str) and nonce and scene.get("rawFile") == f"normal-scene-{nonce}.color8" and
            scene.get("pngFile") == f"normal-scene-{nonce}.png", "Scene file names do not belong to this actual nonce")
    raw_path, png_path = (runtime_media_file(directory / scene[key]) for key in ("rawFile", "pngFile"))
    require(raw_path.parent == directory.resolve(strict=True) and png_path.parent == raw_path.parent and
            raw_path.stat().st_size <= MAX_BYTES and png_path.stat().st_size <= CASE_MAX_BYTES,
            "Scene files exceed their physical directory or byte budget")
    raw, png = raw_path.read_bytes(), png_path.read_bytes()
    rgba = scene_rgba(scene, raw)
    require(len(png) == scene.get("pngByteLength") and digest(png) == scene.get("pngSha256") and
            decode_scene_png(png, tuple(scene["physicalSize"])) == rgba, "Official scene PNG differs from the actual converted GPU image")
    receipt, ticket = scene.get("receipt"), readback.get("submission")
    width, height = scene["physicalSize"]
    padded = ((width * 4 + 255) // 256) * 256 * height
    frame = receipt.get("frame_key") if isinstance(receipt, dict) else None
    require(ticket_valid(ticket) and isinstance(frame, dict) and frame.get("submission") == ticket and
            frame.get("device_id") == ticket["device_id"] and frame.get("generation") == ticket["generation"] and
            uint(receipt.get("request")) and receipt.get("kind") == "Texture" and receipt.get("terminal") == "Succeeded" and
            uint(receipt.get("byte_len")) and padded <= receipt["byte_len"] < padded + 256 and
            isinstance(readback.get("receipt"), dict) and receipt["request"] != readback["receipt"].get("request") and
            uint(readback["receipt"].get("byte_len")) and receipt["byte_len"] + readback["receipt"]["byte_len"] <= MAX_BYTES,
            "Scene receipt is not a separate successful image in the same bounded native submission")
    require(raw_path.read_bytes() == raw and png_path.read_bytes() == png, "Scene bytes changed during verification")
    return {"sceneVerified": True, "sceneImage": scene, "command": command, "png": png,
            "rawPath": str(raw_path), "pngPath": str(png_path)}


def runtime_scene_packet(review_case: dict) -> dict:
    packet = ((review_case.get("data") or {}).get("normal_window_evidence") or {}).get("runtimeSceneMedia")
    require(isinstance(packet, dict) and set(packet) == {"schema", "version", "bootstrapCase", "bootstrapPixelAssociation", "image"} and
            packet.get("schema") == SCENE_SCHEMA and type(packet.get("version")) is int and packet["version"] == 1,
            "Runtime scene media requires the exact bootstrap case and original pixel association")
    bootstrap, proof, image = packet["bootstrapCase"], packet["bootstrapPixelAssociation"], packet["image"]
    require(isinstance(bootstrap, dict) and set(bootstrap) == {"path", "caseJsonSha256", "caseSha256"} and
            isinstance(proof, dict) and set(proof) == {"path", "sha256", "observed"} and isinstance(proof.get("observed"), dict) and
            isinstance(image, dict) and set(image) == {"mime", "byteLength", "pngSha256", "base64"} and image.get("mime") == "image/png" and
            type(image.get("byteLength")) is int and 0 < image["byteLength"] <= CASE_MAX_BYTES and
            isinstance(image.get("base64"), str) and len(image["base64"]) == 4 * ((image["byteLength"] + 2) // 3),
            "Runtime scene media metadata or encoded extent differs")
    for value in (bootstrap.get("caseJsonSha256"), bootstrap.get("caseSha256"), proof.get("sha256"), image.get("pngSha256")):
        require(isinstance(value, str) and len(value) == 64 and all(x in "0123456789abcdef" for x in value), "Runtime scene media hash is invalid")
    return packet


def canonical_case_sha(value: dict) -> str:
    def normalized(item):
        if isinstance(item, dict): return {key: normalized(child) for key, child in item.items()}
        if isinstance(item, list): return [normalized(child) for child in item]
        return int(item) if isinstance(item, float) and item.is_integer() else item
    return digest(json.dumps(normalized(value), sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode())


def validate_runtime_scene_media(review_case: dict, proof: dict, profile: dict) -> dict:
    packet = runtime_scene_packet(review_case)
    case_path, proof_path = (runtime_media_file(Path(packet[key]["path"])) for key in ("bootstrapCase", "bootstrapPixelAssociation"))
    case_bytes, proof_bytes = case_path.read_bytes(), proof_path.read_bytes()
    require(len(case_bytes) <= CASE_MAX_BYTES and len(proof_bytes) <= CASE_MAX_BYTES and
            digest(case_bytes) == packet["bootstrapCase"]["caseJsonSha256"] and
            digest(proof_bytes) == packet["bootstrapPixelAssociation"]["sha256"], "Bootstrap case or association bytes changed")
    bootstrap = json.loads(case_bytes)
    require(isinstance(bootstrap, dict) and "runtimeSceneMedia" not in ((bootstrap.get("data") or {}).get("normal_window_evidence") or {}) and
            canonical_case_sha(bootstrap) == packet["bootstrapCase"]["caseSha256"], "Bootstrap case hash or media lifecycle differs")
    observed = packet["bootstrapPixelAssociation"]["observed"]
    require(json_identity_equal(json.loads(proof_bytes), observed) and observed.get("caseJsonSha256") == digest(case_bytes) and
            observed.get("caseSha256") == packet["bootstrapCase"]["caseSha256"] and proof.get("nonce") != observed.get("nonce") and
            proof.get("caseSha256") != observed.get("caseSha256"), "Scene needs two distinct actual case/nonce captures")
    for key in ("officialCaseHashModuleSha256", "officialCaseHashInputAuditSha256"):
        require(observed.get(key) == proof.get(key) and isinstance(observed.get(key), str) and len(observed[key]) == 64,
                "Bootstrap and final cases need the same actual official hash contract")
    contract = observed.get("caseContract") or {}
    require(isinstance(contract.get("configurationPath"), str) and isinstance(contract.get("configurationSha256"), str),
            "Bootstrap official case contract binding is absent")
    current_binding = load_case_contract_configuration(Path(contract["configurationPath"]), contract["configurationSha256"], Path(__file__).resolve().parents[3])
    require(json_identity_equal(current_binding, contract) and contract.get("moduleSha256") == observed["officialCaseHashModuleSha256"] and
            contract.get("inputAuditSha256") == observed["officialCaseHashInputAuditSha256"] and
            json_identity_equal(observed.get("processImage"), proof.get("processImage")), "Bootstrap source contract or actual product binary differs")
    renderer = observed.get("processImage") or {}
    renderer = {"path": renderer.get("path"), "actual_sha256": renderer.get("sha256")}
    size, scale = tuple(observed.get("physicalSize") or ()), observed.get("actualWinitScaleFactor")
    require(len(size) == 2 and scale == review_case.get("dpi") and bootstrap.get("viewport") == review_case.get("viewport"),
            "Bootstrap and final scene cases differ in actual viewport or scale")
    _, _, _, bootstrap_profile = validate_association(proof_path, observed["caseSha256"], size, scale, renderer,
        ((bootstrap.get("data") or {}).get("normal_window_evidence") or {}).get("focusPublication"), bootstrap)
    first = validate_scene_image(observed["normalReadbackReceipt"], bootstrap_profile, Path(observed["normalReadbackReceiptPath"]).parent)
    current = validate_scene_image(proof["normalReadbackReceipt"], profile, Path(proof["normalReadbackReceiptPath"]).parent)
    stable = ("owner", "viewportRoot", "surfaceKey", "paneKind", "resourceKey", "resourceGeneration", "physicalSize", "sourceFormat",
              "sourceByteLayout", "colorEncoding", "alphaMode", "opaquePixels", "atlasUv", "overlayPresent", "rawRowBytes", "rawByteLength", "rawSha256",
              "rgba8Sha256", "pngByteLength", "pngSha256", "pngColorEncoding", "conversion")
    require(json_identity_equal({key: first["sceneImage"].get(key) for key in stable},
                                {key: current["sceneImage"].get(key) for key in stable}), "Final nonce changed the actual scene resource or content")
    encoded = packet["image"]
    inline = base64.b64decode(encoded["base64"], validate=True)
    require(base64.b64encode(inline).decode() == encoded["base64"] and inline == first["png"] == current["png"] and
            len(inline) == encoded["byteLength"] and digest(inline) == encoded["pngSha256"], "Inline scene PNG differs from both actual nonce images")
    require(case_path.read_bytes() == case_bytes and proof_path.read_bytes() == proof_bytes, "Bootstrap inputs changed during scene validation")
    return {**{key: value for key, value in current.items() if key not in ("png", "command")},
            "authority": "two-actual-normal-submission-scene-readbacks", "bootstrapCase": packet["bootstrapCase"],
            "bootstrapPixelAssociation": {"path": str(proof_path), "sha256": digest(proof_bytes)},
            "bootstrapNonce": observed["nonce"], "nonce": proof["nonce"], "commandIndex": current["command"]["commandIndex"],
            "sourceRef": current["sceneImage"]["sourceRef"]}


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
                         bound_case: dict | None = None,
                         scene_validation: dict | None = None) -> tuple[dict, Path, Path, dict]:
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
    if bound_case is not None and "runtimeSceneMedia" in ((bound_case.get("data") or {}).get("normal_window_evidence") or {}):
        scene_result = validate_runtime_scene_media(bound_case, proof, profile)
        if scene_validation is not None:
            scene_validation.update(scene_result)
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


def runtime_media_file(path: Path) -> Path:
    physical = physical_file(path)
    for drive in "DEF":
        base = Path(f"{drive}:/cargo-targets/zircon-local")
        if base.exists():
            try:
                if os.path.normcase(os.path.commonpath([physical, base.resolve(strict=True)])) == os.path.normcase(str(base.resolve(strict=True))):
                    return physical
            except ValueError:
                pass
    raise ValueError("Runtime scene receipt input is outside a physical approved artifact root")


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
    canonical = repo / "tools/penpot/apps/zircon-zui-plugin"
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
    support_files = {
        key(repo / "tools/analysis/visual" / name)
        for name in ("build-normal-workbench-contracts.mjs", "normal-workbench-case-producer.mjs")
    }
    toml_root = Path("D:/cargo-targets/zircon-local/penpot-public/zircon-plugin/workspace/tools/penpot/node_modules/.pnpm/smol-toml@1.8.0/node_modules/smol-toml/dist")
    dependency_files = {
        key(toml_root / (name + ".js"))
        for name in ("date", "error", "extract", "index", "parse", "primitive", "stringify", "struct", "util")
    }
    for row in rows:
        require(isinstance(row, dict) and isinstance(row.get("path"), str)
                and isinstance(row.get("sha256"), str) and len(row["sha256"]) == 64
                and all(c in "0123456789abcdef" for c in row["sha256"]),
                "Compiled contract input identity is malformed")
        path = key(row["path"])
        require(path not in paths, "Compiled contract input closure contains duplicate paths")
        try:
            canonical_input = os.path.commonpath([path, key(canonical)]) == key(canonical)
        except ValueError:
            canonical_input = False
        require(canonical_input or path in support_files or path in dependency_files,
                "Compiled contract input is outside the canonical app and explicit compiler dependencies")
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
