"""Record a real desktop Editor capture in the existing native evidence format.

This adapter preserves successful GPU present observations and checks their
actual resource bytes. Full catalog acceptance also requires case admission.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil

from zircon_editor_ui_visual_oracle import _validate_capture_provenance
from zircon_editor_normal_frame_evidence import validate_association, validate_capture_instance


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_sha(value: object) -> str:
    # JSON.stringify, used by the existing TS case hash, emits integral floats
    # without a decimal suffix. Review cases contain finite JSON numbers.
    def normalize(item: object) -> object:
        if isinstance(item, dict):
            return {key: normalize(child) for key, child in item.items()}
        if isinstance(item, list):
            return [normalize(child) for child in item]
        if isinstance(item, float) and item.is_integer():
            return int(item)
        return item

    encoded = json.dumps(normalize(value), sort_keys=True, ensure_ascii=False,
                         separators=(",", ":"), allow_nan=False).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def verified(path: str, expected: str, base: Path) -> Path:
    result = Path(path)
    if not result.is_absolute():
        result = base / result
    result = result.resolve(strict=True)
    if sha(result) != expected:
        raise ValueError(f"Evidence fingerprint changed: {result}")
    return result


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2,
                               allow_nan=False) + "\n", encoding="utf-8")


def source_identity(value: dict) -> tuple | None:
    result = tuple(value.get(key) for key in ("sourcePath", "sourceNodeId", "instancePath"))
    return result if all(isinstance(part, str) and part for part in result) else None


def paint_identity(value: dict) -> tuple | None:
    """Keep authored provenance and qualify real property rows by owner/item.

    A physical slot or generated control name cannot identify the model item
    after scrolling. Missing repeat metadata stays unknown rather than joining
    the last style painted for the same prefab.
    """
    authored = source_identity(value)
    if authored is None:
        return None
    item, field = value.get("itemKey"), value.get("propertyFieldId")
    if item in (None, "") and field in (None, ""):
        return authored
    if (not isinstance(item, str) or not item.isascii() or not item.isdecimal() or
            str(int(item)) != item or int(item) >= 1 << 128 or
            not isinstance(field, str) or not field.strip()):
        return None
    owner = tuple(value.get(key) for key in
                  ("parentSourcePath", "parentSourceNodeId", "parentInstancePath"))
    if not all(isinstance(part, str) and part for part in owner):
        return None
    return (*authored, (*owner, item))


def same_paint_identity(first: dict, second: dict) -> bool:
    identity = paint_identity(first)
    return (identity is not None and identity == paint_identity(second) and
            (first.get("itemKey") in (None, "") or
             first.get("propertyFieldId") == second.get("propertyFieldId")))


def native_row_identity_valid(value: object) -> bool:
    return (isinstance(value, dict) and isinstance(value.get("nodeId"), str) and bool(value["nodeId"]) and
            isinstance(value.get("index"), int) and not isinstance(value["index"], bool) and value["index"] >= 0)


def logical_rect(value: dict | None, scale: float) -> dict | None:
    if not isinstance(value, dict):
        return None
    return {axis: value[axis] / scale for axis in ("x", "y", "width", "height")}


def shaped_line_text(line: dict) -> str:
    text = line.get("original_line_text") or ""
    byte_range = line.get("byte_range")
    if isinstance(byte_range, list) and len(byte_range) == 2:
        return text.encode("utf-8")[byte_range[0]:byte_range[1]].decode("utf-8")
    return text


def present_observations(profile: dict, repo: Path, scale: float, scene_media_proof: dict | None = None) -> dict:
    """Join actual submitted commands, source owners, faces and resolver receipts.

    A successful GPU image residency receipt and a verified resolver receipt
    are both necessary. A font name or the existence of an asset never suffices.
    """
    normal = profile.get("normal_window_evidence") or {}
    submitted = profile.get("submitted_gpu_text") or {}
    paint = normal.get("paintEvidence") or {}
    issues = list(paint.get("issues") or [])
    complete = (normal.get("commandStreamFullRebuild") is True and
                submitted.get("commandStreamFullRebuild") is True and
                normal.get("presentedFrameCount", 0) > 0 and
                normal.get("presentedFrameCount") == submitted.get("presentedFrameCount"))
    if not complete:
        issues.append("No complete source-bound successful GPU present snapshot")
    styles = {}
    for style in paint.get("styles") or []:
        identity = paint_identity(style)
        if identity in styles:
            issues.append(f"Duplicate resolved style owner: {identity}")
        if identity:
            styles[identity] = style
    semantic = []
    for node in normal.get("semanticNodes") or []:
        identity = paint_identity(node)
        if not identity:
            issues.append("A painted semantic node has no immutable source identity")
            continue
        style = styles.get(identity) or {}
        if not same_paint_identity(node, style):
            issues.append(f"No unique resolved style for the current painted owner: {identity}")
            style = {}
        semantic.append({**node, "bounds": logical_rect(node.get("bounds"), scale),
                         "clip": logical_rect(node.get("clip"), scale),
                         "effectiveStyle": style.get("effectiveStyle"),
                         "styleInventory": style.get("styleInventory")})
    commands = {command["commandIndex"]: command for command in normal.get("commands") or []}
    native_rows, native_issues = [], []
    for draw in paint.get("nativeHierarchyDraws") or []:
        owner, origin = draw.get("sourceOwner") or {}, draw.get("originComponent") or {}
        data = draw.get("dataIdentity") or {}
        indices = draw.get("submittedCommandIndices") or []
        ready = (draw.get("sourceOwnerMappingComplete") is True and
                 source_identity(owner) is not None and source_identity(origin) is not None and
                 isinstance(data.get("nodeId"), str) and bool(data["nodeId"]) and bool(indices))
        for index in indices:
            source = (commands.get(index) or {}).get("source") or {}
            if (source.get("nativeIdentityAuthority") != "actual-native-row-draw-and-submitted-command" or
                    source.get("nativeRowIdentity") != data or source.get("sourceOwner") != owner or
                    source.get("originComponent") != origin):
                ready = False
        if not ready:
            native_issues.append(f"Native row lacks unique admitted origin and submitted command links: {data}")
        native_rows.append({**draw, "bounds": logical_rect(draw.get("bounds"), scale),
                            "clip": logical_rect(draw.get("clip"), scale), "drawSourceClosureComplete": ready})
    visible_model_rows = [row for pane in profile.get("native_hierarchy") or []
                          for row in pane.get("rows") or [] if row.get("frame")]
    for row in visible_model_rows:
        matches = [draw for draw in native_rows if
                   (draw.get("dataIdentity") or {}).get("nodeId") == row.get("node_id") and
                   (draw.get("dataIdentity") or {}).get("index") == row.get("index") and
                   draw.get("modelName") == row.get("name") and draw.get("selected") == row.get("selected")]
        if len(matches) != 1:
            native_issues.append(f"Visible native snapshot row lacks one actual draw: {row.get('node_id')}")
    nodes, fingerprints, faces, font_issues = [], {}, {}, []
    for run in submitted.get("runs") or []:
        source = run.get("source") or {}
        identity = paint_identity(source)
        command = commands.get(run.get("commandIndex")) or {}
        native_identity = None
        command_source = command.get("source") or {}
        if any(key in source or key in command_source for key in ("nativeIdentityAuthority", "nativeRowIdentity")):
            owner = source.get("sourceOwner") or {}
            native_identity = source.get("nativeRowIdentity")
            if (source_identity(owner) and source_identity(source.get("originComponent") or {}) and
                    native_row_identity_valid(native_identity) and native_row_identity_valid(command_source.get("nativeRowIdentity")) and
                    source.get("nativeIdentityAuthority") == "actual-native-row-draw-and-submitted-command" and
                    command_source.get("nativeIdentityAuthority") == "actual-native-row-draw-and-submitted-command" and
                    command_source.get("sourceOwner") == owner and
                    command_source.get("nativeRowIdentity") == native_identity and
                    command_source.get("originComponent") == source.get("originComponent")):
                identity = source_identity(owner)
            else:
                font_issues.append("A submitted native row text run lacks its actual draw and source owner closure")
                continue
        if not identity or (native_identity is None and not same_paint_identity(source, command_source)):
            font_issues.append("A submitted text run has no unique matching source command")
            continue
        observed_faces = []
        for face in run.get("faces") or []:
            observed = dict(face)
            path, digest = face.get("resource_path"), face.get("resource_sha256")
            try:
                if not path or not digest:
                    raise ValueError("No actual admitted font resource receipt")
                actual = verified(path, digest, repo)
                observed["currentFileMatches"] = True
                fingerprints[str(actual)] = digest
            except (OSError, ValueError) as error:
                observed["currentFileMatches"] = False
                font_issues.append(f"Font {face.get('font_id')}: {error}")
            if not face.get("sha256") or not isinstance(face.get("face_index"), int):
                font_issues.append("Actual font face bytes SHA or collection index missing")
            faces[face.get("font_id")] = observed
            observed_faces.append(observed)
        observed_ids = {face.get("font_id") for face in observed_faces}
        for line in run.get("lines") or []:
            if any(font_id not in observed_ids for font_id in line.get("font_ids") or []):
                font_issues.append("A shaped line uses a face without resident byte/identity evidence")
        lines = [{**line, "frame": logical_rect(line.get("frame"), scale),
                  "baseline_y": line["baseline_y"] / scale}
                 for line in run.get("lines") or []]
        if run.get("text", "").strip() and (not lines or not observed_faces):
            font_issues.append("Nonempty submitted text lacks actual shaped lines or faces")
        fonts = []
        for face in observed_faces:
            families, count = face.get("families") or [], face.get("glyph_count")
            if not families or not isinstance(count, int) or count <= 0:
                font_issues.append("Submitted face lacks actual family metadata or glyph usage count")
                continue
            fonts.append({"familyName": families[0], "glyphCount": count,
                          "postScriptName": face.get("post_script_name"),
                          "resourcePath": face.get("resource_path"), "sha256": face.get("sha256"),
                          "faceIndex": face.get("face_index")})
        request = command.get("paint") or {}
        if not request.get("fontFamily") or not request.get("fontWeight"):
            font_issues.append("Submitted text command lacks its resolved family/weight request")
        nodes.append({**source, "nodeId": json.dumps({"kind": "native-hierarchy-row", "sourceOwner": identity, "dataIdentity": native_identity} if native_identity is not None else identity, ensure_ascii=False, separators=(",", ":")),
                      "nativeDataIdentity": native_identity,
                      "commandIndex": run["commandIndex"], "text": run.get("text"),
                      "bounds": logical_rect(command.get("bounds"), scale),
                      "clip": logical_rect(run.get("clip"), scale),
                      "lines": lines, "faces": observed_faces, "fonts": fonts,
                      "layout": {"font_size": request.get("fontSize", 0) / scale,
                                 "font_family": request.get("fontFamily"),
                                 "font_weight": request.get("fontWeight"),
                                 "line_height": request.get("lineHeight", 0) / scale,
                                 "lines": [{"text": shaped_line_text(line), "frame": line["frame"]}
                                           for line in lines]},
                      "submittedPaint": command.get("paint")})
    resources = []
    image_commands = [command for command in commands.values()
                      if (command.get("paint") or {}).get("kind") == "image"]
    receipts = (paint.get("assetAudit") or {}).get("resources") or []
    media_issues = []
    for command in image_commands:
        payload = command["paint"]
        command_source = command.get("source") or {}
        identity = paint_identity(command_source)
        if scene_media_proof and scene_media_proof.get("sceneVerified") is True and command.get("commandIndex") == scene_media_proof.get("commandIndex"):
            scene = scene_media_proof["sceneImage"]
            if (scene.get("resourceKey") != payload.get("resourceKey") or scene.get("resourceGeneration") != payload.get("generation") or
                    scene.get("sourceRef") != command_source.get("sourceRef") or source_identity(scene.get("owner") or {}) != source_identity(command_source) or
                    payload.get("residentAfterSuccessfulPresent") is not True):
                raise ValueError("Verified runtime scene does not own this actual submitted image command")
            resources.append({"kind": "runtime-product-scene", "authority": scene_media_proof["authority"],
                              "sceneVerified": True, "resourceKey": payload["resourceKey"], "gpuGeneration": payload["generation"],
                              "residentAfterSuccessfulPresent": True, "owner": scene["owner"], "viewportRoot": scene["viewportRoot"],
                              "sourceRef": scene["sourceRef"], "rawSha256": scene["rawSha256"], "pngSha256": scene["pngSha256"],
                              "rgba8Sha256": scene["rgba8Sha256"], "conversion": scene["conversion"],
                              "nonce": scene_media_proof["nonce"], "bootstrapNonce": scene_media_proof["bootstrapNonce"]})
            continue
        matches = [receipt for receipt in receipts
                   if receipt.get("resourceKey") == payload.get("resourceKey") and
                   same_paint_identity(receipt, command_source) and identity is not None]
        if len(matches) != 1:
            media_issues.append(f"No unique actual resolver receipt for GPU image {payload.get('resourceKey')}")
            continue
        resource = {**matches[0], "gpuGeneration": payload.get("generation"),
                    "residentAfterSuccessfulPresent": payload.get("residentAfterSuccessfulPresent") is True}
        try:
            actual = verified(resource["path"], resource["sha256"], repo)
            resource["currentFileMatches"] = True
            fingerprints[str(actual)] = resource["sha256"]
        except (KeyError, OSError, ValueError) as error:
            resource["currentFileMatches"] = False
            media_issues.append(str(error))
        if resource.get("loaded") is not True or not resource["residentAfterSuccessfulPresent"]:
            media_issues.append(f"Image is not resolver-loaded and GPU-resident: {payload.get('resourceKey')}")
        resources.append(resource)
    if (paint.get("assetAudit") or {}).get("complete") is not True:
        media_issues.append("Actual resolver asset audit is incomplete")
    return {"semanticNodes": semantic, "nodes": nodes, "faces": list(faces.values()),
            "runtimeAssetFingerprints": sorted(fingerprints.items()),
            "fontAudit": {"loaded": complete and bool(nodes) and not font_issues,
                          "issues": font_issues, "faces": list(faces.values())},
            "mediaReadiness": {"complete": complete and not media_issues,
                               "resources": resources, "issues": media_issues,
                               "sceneVerified": bool(scene_media_proof and scene_media_proof.get("sceneVerified") is True)},
            "semanticAudit": {"complete": complete and bool(semantic) and not issues,
                              "coverage": "painted-template-models", "issues": issues},
            "focusPublication": normal.get("focusPublication"), "normal": normal,
            "nativeHierarchyDraws": native_rows,
            "nativeHierarchySourceAudit": {"complete": complete and not native_issues,
                                            "issues": native_issues, "coverage": "actual-visible-native-draws"}}


def record(args: argparse.Namespace) -> Path:
    manifest_path = args.capture_manifest.resolve(strict=True)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8-sig"))
    _validate_capture_provenance(manifest, manifest_path.parent)
    project = manifest.get("input_project")
    if not isinstance(project, dict) or project.get("unchanged_before_after") is not True:
        raise ValueError("Normal Workbench evidence requires the actual unchanged product project")
    catalog = json.loads(args.catalog.read_text(encoding="utf-8-sig"))
    repo = Path(catalog["repoRoot"]).resolve(strict=True)
    entries = [entry for entry in catalog["entries"] if entry["sourcePath"] == args.source]
    if len(entries) != 1:
        raise ValueError("Source must resolve to one current catalog entry")
    entry = entries[0]
    cases = [case for case in entry["cases"] if case["id"] == args.case]
    captures = [capture for capture in manifest["captures"] if capture["case_id"] == args.capture_case]
    if len(cases) != 1 or len(captures) != 1:
        raise ValueError("Case and physical capture must each resolve exactly once")
    case, capture = cases[0], captures[0]
    verified(entry["sourcePath"], entry["sourceSha256"], repo)
    for dependency in entry["dependencyFingerprints"]:
        verified(dependency["sourcePath"], dependency["sha256"], repo)
    if capture["presenter_backend"] != "gpu":
        raise ValueError("A real GPU Editor presenter is required")
    scale = float(case["dpi"])
    viewport = case["viewport"]
    if (capture["winit_scale_factor"] != scale or
            capture["windows_window_dpi"] != round(96 * scale) or
            capture["windows_window_dpi_before_capture"] != round(96 * scale) or
            capture["windows_window_dpi_after_interactions"] != round(96 * scale) or
            capture["requested_client_extent_logical"] != viewport):
        raise ValueError("Catalog case differs from the actual Windows DPI or logical client extent")
    if not capture.get("capture_monitor_device_name"):
        raise ValueError("Capture must identify its actual selected display")
    screenshot = verified(capture["screenshot"]["path"],
                          capture["screenshot"]["sha256"], manifest_path.parent)
    profile_path = verified(capture["profile_geometry_path"],
                            capture["profile_geometry_sha256"], manifest_path.parent)
    profile = json.loads(profile_path.read_text(encoding="utf-8"))
    if profile.get("presenter_backend") != "gpu" or profile.get("capture_sequence", 0) <= 0:
        raise ValueError("A successful normal GPU present sequence is required")
    if profile.get("winit_scale_factor") != scale:
        raise ValueError("GPU present scale differs from the catalog case")
    expected_physical = {axis: round(viewport[axis] * scale) for axis in ("width", "height")}
    if (profile["window_client_size"] != expected_physical or
            {axis: capture["screenshot"][axis] for axis in expected_physical} != expected_physical):
        raise ValueError("GPU present and desktop PNG must match the physical case extent")
    pixel_proof = None
    scene_media_proof = {}
    pixel_proof_path = getattr(args, "pixel_association", None)
    if pixel_proof_path is not None:
        pixel_proof_path = pixel_proof_path.resolve(strict=True)
        bound_focus = ((case.get("data") or {}).get("normal_window_evidence") or {}).get("focusPublication")
        pixel_proof, screenshot, profile_path, profile = validate_association(
            pixel_proof_path, canonical_sha(case),
            (expected_physical["width"], expected_physical["height"]), scale,
            manifest["binaries"]["editor"], bound_focus, case, scene_media_proof)
        validate_capture_instance(pixel_proof, capture, project, pixel_proof_path)
        if (pixel_proof.get("actualMonitor") or {}).get("deviceName") != capture["capture_monitor_device_name"]:
            raise ValueError("Matched normal pixel frame moved to a different actual display")
    output = args.output_directory.resolve()
    allowed = any(output.is_relative_to(Path(f"{drive}:/cargo-targets/zircon-local").resolve())
                  for drive in "DEF")
    if not allowed:
        raise ValueError("Evidence output must remain under D/E/F:/cargo-targets/zircon-local")
    catalog_root = args.catalog.resolve(strict=True).parent
    if not output.is_relative_to(catalog_root):
        raise ValueError("Use a D/E/F managed catalog and place evidence beneath its catalog directory")
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise ValueError("Use an empty evidence directory; existing receipts are preserved")
    image_path = output / "editor-normal-window.png"
    shutil.copyfile(screenshot, image_path)
    submitted = profile.get("submitted_gpu_text") or {}
    if "runtimeSceneMedia" in ((case.get("data") or {}).get("normal_window_evidence") or {}) and not scene_media_proof.get("sceneVerified"):
        raise ValueError("Runtime scene case requires two actual source-bound nonce readbacks")
    observations = present_observations(profile, repo, scale, scene_media_proof)
    semantic = observations["semanticNodes"]
    case_hash = canonical_sha(case)
    reasons = [
        "Normal-window observations do not yet certify the full catalog business-state selector",
        "Normal GPU painted-template coverage does not yet certify catalog semantic source closure",
    ]
    if pixel_proof is None:
        reasons.append("Desktop PNG and GPU profile are observed separately; exact present-to-PNG identity is not certified")
    if not submitted:
        reasons.append("The GPU presenter did not emit submitted text observations")
    if submitted.get("coverage") != "submitted-command-stream" or submitted.get("commandStreamFullRebuild") is not True:
        reasons.append("GPU text observation covers a retained or damaged stream, not a complete rebuild")
    for audit in ("semanticAudit", "fontAudit", "mediaReadiness", "nativeHierarchySourceAudit"):
        reasons.extend(observations[audit].get("issues") or [])
    envelope = {"case": case, "caseSha256": case_hash, "caseId": case["id"],
                "coordinateSpace": "logical", "captureSequence": profile["capture_sequence"],
                "rawProfilePath": str(profile_path), "rawProfileSha256": sha(profile_path),
                "pixelFrameVerified": pixel_proof is not None,
                "sceneVerified": scene_media_proof.get("sceneVerified") is True,
                "pixelAssociation": pixel_proof,
                "pixelAssociationReceiptPath": str(pixel_proof_path) if pixel_proof_path else None,
                "pixelAssociationReceiptSha256": sha(pixel_proof_path) if pixel_proof_path else None}
    geometry = {**envelope, "schema": "dev.zircon.zui.native-geometry-evidence", "version": 1,
                "semanticAudit": {**observations["semanticAudit"], "complete": False},
                "paintedTemplateAudit": observations["semanticAudit"],
                "layout": {"semanticNodes": semantic},
                "focusPublication": observations["focusPublication"],
                "nativeHierarchyDraws": observations["nativeHierarchyDraws"],
                "nativeHierarchySourceAudit": observations["nativeHierarchySourceAudit"],
                "captureHold": observations["normal"].get("captureHold"),
                "rawNormalWindowProfile": profile, "nativeReadiness": {"status": "pending", "reasons": reasons}}
    text = {**envelope, "schema": "dev.zircon.zui.native-text-evidence", "version": 1,
            "fontAudit": observations["fontAudit"], "nodes": observations["nodes"],
            "runtimeAssetFingerprints": observations["runtimeAssetFingerprints"],
            "rawSubmittedGpuText": submitted,
            "mediaReadiness": observations["mediaReadiness"]}
    geometry_path, text_path = output / "geometry.json", output / "text.json"
    write_json(geometry_path, geometry)
    write_json(text_path, text)
    renderer = manifest["binaries"]["editor"]
    relative_output = output.relative_to(catalog_root)
    evidence = {"sourcePath": entry["sourcePath"], "caseId": case["id"], "status": "pending",
                "pendingReason": "; ".join(reasons), "rendererKind": "zircon-editor-retained-host",
                "rendererPath": renderer["path"], "rendererSha256": renderer["actual_sha256"],
                "sourceSha256": entry["sourceSha256"], "inputSha256": entry["sourceSha256"],
                "dependencySha256": entry["dependencySha256"], "caseSha256": case_hash,
                "screenshotPath": (relative_output / image_path.name).as_posix(), "screenshotSha256": sha(image_path),
                "geometryPath": (relative_output / geometry_path.name).as_posix(), "geometrySha256": sha(geometry_path),
                "textPath": (relative_output / text_path.name).as_posix(), "textSha256": sha(text_path),
                "normalWindowCaptureManifest": str(manifest_path), "normalWindowCaptureManifestSha256": sha(manifest_path)}
    if pixel_proof is not None:
        evidence["pixelFrameVerified"] = True
        evidence["pixelAssociationReceiptPath"] = str(pixel_proof_path)
        evidence["pixelAssociationReceiptSha256"] = sha(pixel_proof_path)
    if case.get("reviewHost"):
        evidence["reviewHostSha256"] = case["reviewHost"]["sha256"]
    report_path = output / "editor-native-evidence.json"
    write_json(report_path, {"schema": "dev.zircon.zui.native-evidence", "version": 1,
                             "rendererSha256": renderer["actual_sha256"], "evidence": [evidence]})
    return report_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture-manifest", type=Path, required=True)
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--case", required=True)
    parser.add_argument("--capture-case", required=True)
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument("--pixel-association", type=Path, help="Actual normal Win32/GPU handshake receipt; independently revalidated")
    args = parser.parse_args()
    print(record(args))


if __name__ == "__main__":
    main()
