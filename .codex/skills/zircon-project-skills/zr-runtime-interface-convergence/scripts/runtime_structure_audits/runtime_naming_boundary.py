from __future__ import annotations

import re
from dataclasses import asdict, dataclass
from pathlib import Path


EDITOR_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_]*editor[A-Za-z0-9_]*\b", re.I)
LEGACY_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_]*legacy[A-Za-z0-9_]*\b", re.I)
RAW_STRING_LITERAL_START = re.compile(r'(?:b|c)?r(?P<hashes>#{0,255})"')
SIMPLE_CHAR_ESCAPES = frozenset({"'", '"', "n", "r", "t", "\\", "0"})
HEX_DIGITS = frozenset("0123456789abcdefABCDEF")
ITEM_LEADING_QUALIFIERS = re.compile(
    r"^(?:(?:pub(?:\s*\([^)]*\))?|unsafe)\s+)*"
)
CONST_FN_ITEM = re.compile(
    r'^const(?:\s+(?:unsafe|async))?(?:\s+extern(?:\s+"[^"]*"))?\s+fn\b'
)

NAMING_CLASSIFICATION_ORDER = (
    "test-fixture",
    "runtime-profile-editor-host-target",
    "dynamic-api-editor-host-mode",
    "runtime-ui-component-catalog-editor-controls",
    "runtime-ui-template-editor-profile",
    "runtime-asset-editor-metadata",
    "framework-editor-facing-descriptor",
    "graphics-editor-facing-metadata",
    "platform-editor-target-diagnostic",
    "rhi-editor-surface-label",
    "scene-reflection-editor-visible-metadata",
    "script-editor-operation-contribution-descriptor",
    "runtime-text-editor-product-fixture",
    "curated-runtime-facade-editor-reference",
    "legacy-runtime-ui-input-debt",
    "legacy-runtime-ui-render-table-debt",
    "legacy-runtime-graphics-debt",
    "legacy-runtime-dds-container-policy",
    "legacy-runtime-ui-template-schema-debt",
    "legacy-runtime-ui-layout-debt",
    "legacy-runtime-input-event-debt",
    "legacy-runtime-asset-schema-debt",
    "legacy-dynamic-api-migration-debt",
    "legacy-scene-schema-render-debt",
    "curated-runtime-facade-legacy-reference",
    "unclassified-runtime-naming-reference",
)

SCRIPT_EDITOR_CONTRIBUTION_PATHS = frozenset(
    {
        "zircon_runtime/src/script/mod.rs",
        "zircon_runtime/src/script/vm/capability_set.rs",
        "zircon_runtime/src/script/vm/host_interface/descriptor.rs",
        "zircon_runtime/src/script/vm/host_interface/mod.rs",
        "zircon_runtime/src/script/vm/host_interface/registry.rs",
        "zircon_runtime/src/script/vm/mod.rs",
        "zircon_runtime/src/script/vm/plugin/state_migration.rs",
        "zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs",
    }
)

RUNTIME_TEXT_EDITOR_PRODUCT_FIXTURE_PATHS = frozenset(
    {
        "zircon_runtime/src/text/cache/shaped_cache.rs",
        "zircon_runtime/src/text/parallel/shape_pool.rs",
    }
)

NAMING_CLASSIFICATION_DECISIONS = {
    "test-fixture": {
        "target_owner": "runtime tests",
        "required_action": "test-only naming is allowed by file boundary",
    },
    "runtime-profile-editor-host-target": {
        "target_owner": "runtime profile and plugin target mode owner",
        "required_action": "keep as explicit editor-host target/profile vocabulary",
    },
    "dynamic-api-editor-host-mode": {
        "target_owner": "dynamic API runtime-session profile owner",
        "required_action": "keep as explicit editor-host session mode vocabulary",
    },
    "runtime-ui-component-catalog-editor-controls": {
        "target_owner": "runtime UI component catalog owner",
        "required_action": (
            "keep as component-catalog editor/control metadata; do not store scene "
            "authoring state here"
        ),
    },
    "runtime-ui-template-editor-profile": {
        "target_owner": "runtime UI compiled-asset profile owner",
        "required_action": "keep as UI asset compiler profile vocabulary",
    },
    "runtime-asset-editor-metadata": {
        "target_owner": "runtime asset metadata owner",
        "required_action": (
            "keep only asset authoring metadata or display labels; move behavior to "
            "editor-owned code"
        ),
    },
    "framework-editor-facing-descriptor": {
        "target_owner": "runtime framework descriptor and diagnostics owner",
        "required_action": "keep only neutral descriptors consumed by editor/tooling",
    },
    "graphics-editor-facing-metadata": {
        "target_owner": "graphics renderer data and diagnostics owner",
        "required_action": "keep as metadata/reporting until the graphics/RHI slice",
    },
    "platform-editor-target-diagnostic": {
        "target_owner": "platform capability matrix owner",
        "required_action": "keep as runtime target-mode diagnostic vocabulary",
    },
    "rhi-editor-surface-label": {
        "target_owner": "RHI UI surface resource-label owner",
        "required_action": "keep as resource label until the UI/RHI resource naming slice",
    },
    "scene-reflection-editor-visible-metadata": {
        "target_owner": "scene reflection and inspection metadata owner",
        "required_action": (
            "keep only neutral reflection metadata such as visibility hints; editor "
            "authoring state remains forbidden in serialization"
        ),
    },
    "script-editor-operation-contribution-descriptor": {
        "target_owner": "runtime script host-interface contribution descriptor owner",
        "required_action": (
            "keep only typed editor-operation capability, registration, and reflection "
            "metadata; editor command execution and authoring state remain editor-owned"
        ),
    },
    "runtime-text-editor-product-fixture": {
        "target_owner": "runtime text cache and parallel-shaping test owner",
        "required_action": (
            "keep editor-named sample strings inside cfg(test) product fixtures; "
            "production text behavior remains product-neutral"
        ),
    },
    "curated-runtime-facade-editor-reference": {
        "target_owner": "runtime facade or diagnostic owner",
        "required_action": "keep as curated facade/doc or diagnostic channel vocabulary",
    },
    "legacy-runtime-ui-input-debt": {
        "target_owner": "runtime UI input dispatch route owner",
        "required_action": "rename legacy input route variables during the runtime UI input slice",
    },
    "legacy-runtime-ui-render-table-debt": {
        "target_owner": "runtime UI collection row render owner",
        "required_action": "rename table text split fallback away from legacy terminology",
    },
    "legacy-runtime-graphics-debt": {
        "target_owner": "runtime graphics/render extraction owner",
        "required_action": "rename legacy render masks and feature fixtures during the graphics slice",
    },
    "legacy-runtime-dds-container-policy": {
        "target_owner": "texture importer DDS container owner",
        "required_action": "replace legacy DDS wording with explicit DDS capability policy",
    },
    "legacy-runtime-ui-template-schema-debt": {
        "target_owner": "runtime UI template schema owner",
        "required_action": "finish schema migration naming and remove legacy template wording",
    },
    "legacy-runtime-ui-layout-debt": {
        "target_owner": "runtime UI layout pass owner",
        "required_action": "rename the remaining legacy layout backend vocabulary",
    },
    "legacy-runtime-input-event-debt": {
        "target_owner": "runtime input event owner",
        "required_action": "rename legacy input delta helpers when the input route is cut over",
    },
    "legacy-runtime-asset-schema-debt": {
        "target_owner": "runtime asset schema owner",
        "required_action": "replace legacy asset defaults with explicit versioned migration names",
    },
    "legacy-dynamic-api-migration-debt": {
        "target_owner": "dynamic API input adapter owner",
        "required_action": "rename legacy input adapter helper after the current route stabilizes",
    },
    "legacy-scene-schema-render-debt": {
        "target_owner": "runtime scene schema/render owner",
        "required_action": "replace legacy scene schema/render names with explicit version names",
    },
    "curated-runtime-facade-legacy-reference": {
        "target_owner": "runtime facade/accessibility owner",
        "required_action": "keep temporarily as assistive/facade debt with owner classification",
    },
    "unclassified-runtime-naming-reference": {
        "target_owner": "unknown",
        "required_action": (
            "classify this editor/legacy reference with an owner or remove it before "
            "accepting the runtime naming boundary"
        ),
    },
}


@dataclass
class RuntimeNamingReferenceDecision:
    path: str
    line: int
    snippet: str
    term: str
    tokens: list[str]
    classification: str
    target_owner: str
    required_action: str


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _is_test_path(relative_path: str) -> bool:
    parts = relative_path.split("/")
    name = parts[-1]
    return "tests" in parts or name == "tests.rs" or name.endswith("_tests.rs")


def _runtime_source_files(root: Path) -> list[Path]:
    runtime_src = root / "zircon_runtime" / "src"
    if not runtime_src.exists():
        return []
    return sorted(runtime_src.rglob("*.rs"))


def _classify_editor_reference(
    relative_path: str,
    tokens: tuple[str, ...] = (),
    *,
    in_cfg_test_item: bool = False,
) -> str:
    if _is_test_path(relative_path):
        return "test-fixture"
    if relative_path == "zircon_runtime/src/core/runtime/lifecycle.rs":
        return "runtime-profile-editor-host-target"
    if relative_path.startswith("zircon_runtime/src/plugin/") or relative_path.startswith(
        "zircon_runtime/src/builtin/runtime_modules/"
    ):
        return "runtime-profile-editor-host-target"
    if relative_path.startswith("zircon_runtime/src/dynamic_api/"):
        return "dynamic-api-editor-host-mode"
    if relative_path.startswith(
        "zircon_runtime/src/ui/component/catalog/"
    ) or relative_path.startswith(
        "zircon_runtime/src/ui/component/state_reducer/"
    ) or relative_path.startswith("zircon_runtime/src/ui/v2/surface_tree/"):
        return "runtime-ui-component-catalog-editor-controls"
    if relative_path.startswith("zircon_runtime/src/ui/template/"):
        return "runtime-ui-template-editor-profile"
    if relative_path.startswith("zircon_runtime/src/asset/"):
        return "runtime-asset-editor-metadata"
    if relative_path.startswith("zircon_runtime/src/core/framework/") or relative_path.startswith(
        "zircon_runtime/src/core/runtime/diagnostics/"
    ):
        return "framework-editor-facing-descriptor"
    if relative_path.startswith("zircon_runtime/src/graphics/"):
        return "graphics-editor-facing-metadata"
    if relative_path.startswith("zircon_runtime/src/platform/"):
        return "platform-editor-target-diagnostic"
    if relative_path.startswith("zircon_runtime/src/rhi"):
        return "rhi-editor-surface-label"
    if relative_path.startswith(
        "zircon_runtime/src/scene/reflect/"
    ) or relative_path.startswith(
        "zircon_runtime/src/scene/inspection/"
    ):
        return "scene-reflection-editor-visible-metadata"
    if (
        relative_path.startswith("zircon_runtime/src/scene/components/scene/")
        and tokens
        and all(token.casefold() == "editor_hint" for token in tokens)
    ):
        return "scene-reflection-editor-visible-metadata"
    if relative_path in SCRIPT_EDITOR_CONTRIBUTION_PATHS:
        return "script-editor-operation-contribution-descriptor"
    if (
        relative_path in RUNTIME_TEXT_EDITOR_PRODUCT_FIXTURE_PATHS
        and in_cfg_test_item
    ):
        return "runtime-text-editor-product-fixture"
    if (
        relative_path == "zircon_runtime/src/diagnostic_log/level.rs"
        and in_cfg_test_item
    ):
        return "curated-runtime-facade-editor-reference"
    if relative_path in {
        "zircon_runtime/src/diagnostic_log/sink.rs",
        "zircon_runtime/src/prelude.rs",
    }:
        return "curated-runtime-facade-editor-reference"
    return "unclassified-runtime-naming-reference"


def _classify_legacy_reference(
    relative_path: str,
    *,
    in_cfg_test_item: bool = False,
) -> str:
    if _is_test_path(relative_path) or in_cfg_test_item:
        return "test-fixture"
    if (
        relative_path.startswith("zircon_runtime/src/ui/surface/input/")
        or relative_path == "zircon_runtime/src/ui/surface/property_mutation.rs"
        or relative_path == "zircon_runtime/src/ui/surface/surface/default_interactions.rs"
    ):
        return "legacy-runtime-ui-input-debt"
    if relative_path == "zircon_runtime/src/ui/surface/render/collection_rows/table.rs":
        return "legacy-runtime-ui-render-table-debt"
    if relative_path.startswith("zircon_runtime/src/graphics/") or relative_path.startswith(
        "zircon_runtime/src/core/framework/render/"
    ):
        return "legacy-runtime-graphics-debt"
    if relative_path == "zircon_runtime/src/asset/assets/texture/upload_support/dds.rs":
        return "legacy-runtime-dds-container-policy"
    if relative_path.startswith("zircon_runtime/src/ui/template/"):
        return "legacy-runtime-ui-template-schema-debt"
    if relative_path.startswith("zircon_runtime/src/ui/layout/"):
        return "legacy-runtime-ui-layout-debt"
    if relative_path.startswith("zircon_runtime/src/input/") or relative_path.startswith(
        "zircon_runtime/src/core/framework/input/"
    ):
        return "legacy-runtime-input-event-debt"
    if relative_path.startswith("zircon_runtime/src/asset/"):
        return "legacy-runtime-asset-schema-debt"
    if relative_path.startswith("zircon_runtime/src/dynamic_api/"):
        return "legacy-dynamic-api-migration-debt"
    if relative_path.startswith("zircon_runtime/src/scene/"):
        return "legacy-scene-schema-render-debt"
    if relative_path in {
        "zircon_runtime/src/prelude.rs",
        "zircon_runtime/src/ui/accessibility/extract.rs",
    }:
        return "curated-runtime-facade-legacy-reference"
    return "unclassified-runtime-naming-reference"


def _classification_counts(
    decisions: list[RuntimeNamingReferenceDecision],
) -> dict[str, int]:
    counts = {classification: 0 for classification in NAMING_CLASSIFICATION_ORDER}
    for decision in decisions:
        counts[decision.classification] = counts.get(decision.classification, 0) + 1
    return {key: value for key, value in counts.items() if value}


def _group_decisions(
    decisions: list[RuntimeNamingReferenceDecision],
) -> dict[str, list[str]]:
    grouped = {classification: [] for classification in NAMING_CLASSIFICATION_ORDER}
    for decision in decisions:
        grouped.setdefault(decision.classification, []).append(
            f"{decision.path}:{decision.line}"
        )
    return {key: value for key, value in grouped.items() if value}


def _classification_samples(
    decisions: list[RuntimeNamingReferenceDecision],
    max_samples: int = 5,
) -> dict[str, list[dict[str, object]]]:
    samples: dict[str, list[dict[str, object]]] = {}
    for decision in decisions:
        bucket = samples.setdefault(decision.classification, [])
        if len(bucket) < max_samples:
            bucket.append(asdict(decision))
    return samples


def _migration_debt(
    classification_counts: dict[str, int],
) -> list[str]:
    debt: list[str] = []
    for classification in NAMING_CLASSIFICATION_ORDER:
        count = classification_counts.get(classification, 0)
        if not count:
            continue
        if classification in {
            "test-fixture",
            "runtime-profile-editor-host-target",
            "dynamic-api-editor-host-mode",
            "runtime-ui-component-catalog-editor-controls",
            "runtime-ui-template-editor-profile",
            "runtime-asset-editor-metadata",
            "framework-editor-facing-descriptor",
            "graphics-editor-facing-metadata",
            "platform-editor-target-diagnostic",
            "rhi-editor-surface-label",
            "scene-reflection-editor-visible-metadata",
            "script-editor-operation-contribution-descriptor",
            "runtime-text-editor-product-fixture",
            "curated-runtime-facade-editor-reference",
            "curated-runtime-facade-legacy-reference",
        }:
            continue
        decision = NAMING_CLASSIFICATION_DECISIONS[classification]
        debt.append(
            f"{classification}: {decision['required_action']} ({count} location(s))"
        )
    return debt


def _rust_char_literal_end(source: str, start: int) -> int | None:
    """Return the end of a valid Rust character literal, preserving lifetimes."""
    cursor = start + 1
    if cursor >= len(source) or source[cursor] in {"'", "\r", "\n"}:
        return None

    if source[cursor] != "\\":
        cursor += 1
    else:
        cursor += 1
        if cursor >= len(source):
            return None
        escape = source[cursor]
        if escape in SIMPLE_CHAR_ESCAPES:
            cursor += 1
        elif escape == "x":
            digits = source[cursor + 1 : cursor + 3]
            if len(digits) != 2 or any(digit not in HEX_DIGITS for digit in digits):
                return None
            cursor += 3
        elif escape == "u" and source[cursor + 1 : cursor + 2] == "{":
            closing = source.find("}", cursor + 2)
            if closing == -1:
                return None
            digits = source[cursor + 2 : closing].replace("_", "")
            if not 1 <= len(digits) <= 6 or any(
                digit not in HEX_DIGITS for digit in digits
            ):
                return None
            cursor = closing + 1
        else:
            return None

    return cursor + 1 if source[cursor : cursor + 1] == "'" else None


def _rust_code_view(source: str) -> str:
    """Blank Rust comments and literals while preserving offsets and newlines."""
    rendered = list(source)
    index = 0
    block_comment_depth = 0
    source_length = len(source)

    def blank(start: int, end: int) -> None:
        for offset in range(start, end):
            if rendered[offset] not in {"\r", "\n"}:
                rendered[offset] = " "

    while index < source_length:
        character = source[index]
        if block_comment_depth:
            if character == "/" and source.startswith("/*", index):
                blank(index, index + 2)
                block_comment_depth += 1
                index += 2
            elif character == "*" and source.startswith("*/", index):
                blank(index, index + 2)
                block_comment_depth -= 1
                index += 2
            else:
                blank(index, index + 1)
                index += 1
            continue

        if character == "/" and source.startswith("//", index):
            line_end = source.find("\n", index)
            if line_end == -1:
                line_end = source_length
            blank(index, line_end)
            index = line_end
            continue
        if character == "/" and source.startswith("/*", index):
            blank(index, index + 2)
            block_comment_depth = 1
            index += 2
            continue

        raw_string_match = (
            RAW_STRING_LITERAL_START.match(source, index)
            if character == "r"
            or (
                character in {"b", "c"}
                and source[index + 1 : index + 2] == "r"
            )
            else None
        )
        if raw_string_match is not None:
            delimiter = '"' + raw_string_match.group("hashes")
            literal_end = source.find(delimiter, raw_string_match.end())
            if literal_end == -1:
                literal_end = source_length
            else:
                literal_end += len(delimiter)
            blank(index, literal_end)
            index = literal_end
            continue

        quote_index = index
        if character in {"b", "c"} and source[index + 1 : index + 2] == '"':
            quote_index += 1
        if source[quote_index : quote_index + 1] == '"':
            literal_end = quote_index + 1
            escaped = False
            while literal_end < source_length:
                literal_character = source[literal_end]
                literal_end += 1
                if escaped:
                    escaped = False
                elif literal_character == "\\":
                    escaped = True
                elif literal_character == '"':
                    break
            blank(index, literal_end)
            index = literal_end
            continue

        if character == "'":
            literal_end = _rust_char_literal_end(source, index)
            if literal_end is not None:
                blank(index, literal_end)
                index = literal_end
            else:
                index += 1
            continue

        index += 1

    return "".join(rendered)


def _item_signature_boundary(
    code_line: str,
    parenthesis_depth: int,
    bracket_depth: int,
    angle_depth: int,
    const_brace_depth: int,
    value_initializer_item: bool,
    in_initializer: bool,
) -> tuple[int, int, int, int, bool, int | None, bool]:
    """Scan one item-signature line for a top-level body or terminator."""
    for index, character in enumerate(code_line):
        if const_brace_depth:
            if character == "{":
                const_brace_depth += 1
            elif character == "}":
                const_brace_depth -= 1
            continue

        if character == "(":
            parenthesis_depth += 1
        elif character == ")":
            parenthesis_depth = max(0, parenthesis_depth - 1)
        elif character == "[":
            bracket_depth += 1
        elif character == "]":
            bracket_depth = max(0, bracket_depth - 1)
        elif (
            character == "="
            and value_initializer_item
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_depth == 0
        ):
            in_initializer = True
        elif (
            character == "<"
            and not in_initializer
            and parenthesis_depth == 0
            and bracket_depth == 0
        ):
            angle_depth += 1
        elif (
            character == ">"
            and not in_initializer
            and parenthesis_depth == 0
            and bracket_depth == 0
        ):
            angle_depth = max(0, angle_depth - 1)
        elif (
            character == "{"
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_depth > 0
        ):
            const_brace_depth = 1
        elif (
            character == "{"
            and in_initializer
            and parenthesis_depth == 0
            and bracket_depth == 0
        ):
            const_brace_depth = 1
        elif (
            character == "{"
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_depth == 0
        ):
            body = code_line[index:]
            return (
                parenthesis_depth,
                bracket_depth,
                angle_depth,
                const_brace_depth,
                in_initializer,
                body.count("{") - body.count("}"),
                False,
            )
        elif (
            character == ";"
            and parenthesis_depth == 0
            and bracket_depth == 0
            and angle_depth == 0
        ):
            return (
                parenthesis_depth,
                bracket_depth,
                angle_depth,
                const_brace_depth,
                in_initializer,
                None,
                True,
            )

    return (
        parenthesis_depth,
        bracket_depth,
        angle_depth,
        const_brace_depth,
        in_initializer,
        None,
        False,
    )


def _item_uses_value_initializer(signature_prefix: str) -> bool:
    tail = ITEM_LEADING_QUALIFIERS.sub("", signature_prefix.strip())
    if tail.startswith("static "):
        return True
    return tail.startswith("const ") and CONST_FN_ITEM.match(tail) is None


def _cfg_test_item_line_numbers(source: str) -> set[int]:
    """Return lines owned by items directly guarded with ``#[cfg(test)]``."""
    test_lines: set[int] = set()
    pending_test_cfg = False
    collecting_item_signature = False
    test_item_depth = 0
    signature_parenthesis_depth = 0
    signature_bracket_depth = 0
    signature_angle_depth = 0
    signature_const_brace_depth = 0
    signature_prefix = ""
    signature_value_initializer_item = False
    signature_in_initializer = False
    code_lines = _rust_code_view(source).splitlines()

    for line_no, code_line in enumerate(code_lines, start=1):
        stripped = code_line.strip()
        brace_delta = code_line.count("{") - code_line.count("}")

        if test_item_depth:
            test_lines.add(line_no)
            test_item_depth += brace_delta
            if test_item_depth <= 0:
                test_item_depth = 0
            continue

        if collecting_item_signature:
            test_lines.add(line_no)
            signature_prefix = f"{signature_prefix} {stripped}".strip()
            signature_value_initializer_item = _item_uses_value_initializer(
                signature_prefix
            )
            (
                signature_parenthesis_depth,
                signature_bracket_depth,
                signature_angle_depth,
                signature_const_brace_depth,
                signature_in_initializer,
                body_depth,
                terminated,
            ) = _item_signature_boundary(
                code_line,
                signature_parenthesis_depth,
                signature_bracket_depth,
                signature_angle_depth,
                signature_const_brace_depth,
                signature_value_initializer_item,
                signature_in_initializer,
            )
            if body_depth is not None:
                test_item_depth = max(0, body_depth)
                collecting_item_signature = False
            elif terminated:
                collecting_item_signature = False
            continue

        if stripped == "#[cfg(test)]":
            pending_test_cfg = True
            continue

        if pending_test_cfg:
            if not stripped or stripped.startswith("#["):
                continue
            test_lines.add(line_no)
            pending_test_cfg = False
            signature_prefix = stripped
            signature_value_initializer_item = _item_uses_value_initializer(
                signature_prefix
            )
            signature_in_initializer = False
            (
                signature_parenthesis_depth,
                signature_bracket_depth,
                signature_angle_depth,
                signature_const_brace_depth,
                signature_in_initializer,
                body_depth,
                terminated,
            ) = _item_signature_boundary(
                code_line,
                0,
                0,
                0,
                0,
                signature_value_initializer_item,
                signature_in_initializer,
            )
            if body_depth is not None:
                test_item_depth = max(0, body_depth)
            elif not terminated:
                collecting_item_signature = True

    return test_lines


def _decisions_for_term(
    root: Path,
    pattern: re.Pattern[str],
    term: str,
) -> list[RuntimeNamingReferenceDecision]:
    decisions: list[RuntimeNamingReferenceDecision] = []
    for path in _runtime_source_files(root):
        relative_path = _relative(root, path)
        source = _read_text(path)
        cfg_test_item_lines = _cfg_test_item_line_numbers(source)
        for line_no, line in enumerate(source.splitlines(), start=1):
            tokens = sorted({match.group(0) for match in pattern.finditer(line)})
            if not tokens:
                continue
            classification = (
                _classify_editor_reference(
                    relative_path,
                    tuple(tokens),
                    in_cfg_test_item=line_no in cfg_test_item_lines,
                )
                if term == "editor"
                else _classify_legacy_reference(
                    relative_path,
                    in_cfg_test_item=line_no in cfg_test_item_lines,
                )
            )
            decision = NAMING_CLASSIFICATION_DECISIONS[classification]
            decisions.append(
                RuntimeNamingReferenceDecision(
                    path=relative_path,
                    line=line_no,
                    snippet=line.strip(),
                    term=term,
                    tokens=tokens,
                    classification=classification,
                    target_owner=decision["target_owner"],
                    required_action=decision["required_action"],
                )
            )
    return decisions


def _term_report(decisions: list[RuntimeNamingReferenceDecision]) -> dict[str, object]:
    classification_counts = _classification_counts(decisions)
    unclassified = [
        decision
        for decision in decisions
        if decision.classification == "unclassified-runtime-naming-reference"
    ]
    return {
        "location_count": len(decisions),
        "file_count": len({decision.path for decision in decisions}),
        "classification_counts": classification_counts,
        "classification_groups": _group_decisions(decisions),
        "classification_samples": _classification_samples(decisions),
        "migration_debt": _migration_debt(classification_counts),
        "migration_debt_count": len(_migration_debt(classification_counts)),
        "unclassified_locations": [asdict(decision) for decision in unclassified],
        "unclassified_location_count": len(unclassified),
        "gate_status": "blocked" if unclassified else "classified",
    }


def runtime_naming_boundary_audit(root: Path) -> dict[str, object]:
    editor_decisions = _decisions_for_term(root, EDITOR_TOKEN_RE, "editor")
    legacy_decisions = _decisions_for_term(root, LEGACY_TOKEN_RE, "legacy")
    editor_report = _term_report(editor_decisions)
    legacy_report = _term_report(legacy_decisions)
    return {
        "editor": editor_report,
        "legacy": legacy_report,
        "gate_status": (
            "blocked"
            if editor_report["unclassified_location_count"]
            or legacy_report["unclassified_location_count"]
            else "classified"
        ),
    }
