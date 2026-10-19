"""In-memory contract tests for the public installer client pin gate."""
from __future__ import annotations

import base64
import copy
import os
import re
import sys
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[3]
INSTALLER = ROOT / "tools/dev/install-jenkins-management.ps1"
RESULT_MARKER = "ZR_CLIENT_PIN_RESULTS:"

CLIENTS = {
    "entry": (
        "tools/jenkins-management/entry.py",
        "ExpectedEntrySourceSha256",
        "Reviewed public entry pin differs",
    ),
    "local_cargo": (
        "tools/dev/local_cargo.py",
        "ExpectedLocalCargoSourceSha256",
        "Reviewed native helper pin differs",
    ),
    "frontend": (
        "tools/jenkins/frontend.py",
        "ExpectedFrontendSourceSha256",
        "Reviewed frontend pin differs",
    ),
    "submit": (
        "tools/dev/submit-jenkins-management.ps1",
        "ExpectedSubmitSourceSha256",
        "ExpectedSubmitSourceSha256 does not match current raw bytes",
    ),
}

OLD_BYTES = {name: f"old {name} client\r\n".encode("ascii") for name in CLIENTS}
REVIEWED_BYTES = {name: f"reviewed {name} client\r\n".encode("ascii") for name in CLIENTS}
_MISSING = object()


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _scenario(
    identifier: str,
    client_name: str,
    client_bytes: bytes,
    *,
    original: object = _MISSING,
    pins: dict[str, object] | None = None,
) -> dict[str, object]:
    row: dict[str, object] = {
        "Id": identifier,
        "ClientName": client_name,
        "BytesBase64": base64.b64encode(client_bytes).decode("ascii"),
    }
    if original is not _MISSING:
        row["OriginalSha256"] = original
    row.update(pins or {})
    return row


def _scenarios() -> tuple[list[dict[str, object]], dict[str, tuple[str, str | None]]]:
    cases: list[dict[str, object]] = []
    expectations: dict[str, tuple[str, str | None]] = {}

    def accept(identifier: str, case: dict[str, object], expected_hash: str) -> None:
        cases.append(case)
        expectations[identifier] = ("accepted", expected_hash)

    def reject(identifier: str, case: dict[str, object], message: str) -> None:
        cases.append(case)
        expectations[identifier] = ("rejected", message)

    for name, (path, _pin_name, _error) in CLIENTS.items():
        content = OLD_BYTES[name]
        identifier = f"first_install_{name}"
        accept(identifier, _scenario(identifier, path, content), _sha256(content))

        identifier = f"unchanged_raw_{name}"
        accept(
            identifier,
            _scenario(identifier, path, content, original=_sha256(content)),
            _sha256(content),
        )

        reviewed = REVIEWED_BYTES[name]
        pin_name = CLIENTS[name][1]
        identifier = f"reviewed_change_{name}"
        accept(
            identifier,
            _scenario(
                identifier,
                path,
                reviewed,
                original=_sha256(content),
                pins={pin_name: _sha256(reviewed)},
            ),
            _sha256(reviewed),
        )

        wrong_pin = _sha256(f"wrong pin for {name}".encode("ascii"))
        identifier = f"wrong_pin_old_raw_{name}"
        reject(
            identifier,
            _scenario(
                identifier,
                path,
                content,
                original=_sha256(content),
                pins={pin_name: wrong_pin},
            ),
            _error,
        )

        for suffix, provided in (("empty", ""), ("null", None)):
            identifier = f"{suffix}_pin_old_raw_{name}"
            reject(
                identifier,
                _scenario(
                    identifier,
                    path,
                    content,
                    original=_sha256(content),
                    pins={pin_name: provided},
                ),
                _error,
            )

    submit_path, submit_pin, submit_error = CLIENTS["submit"]
    submit_old = OLD_BYTES["submit"]
    submit_reviewed = REVIEWED_BYTES["submit"]
    identifier = "changed_submit_without_review"
    reject(
        identifier,
        _scenario(identifier, submit_path, submit_reviewed, original=_sha256(submit_old)),
        "Unreviewed public entry change",
    )

    identifier = "submit_entry_pin_does_not_authorize_submit_change"
    reject(
        identifier,
        _scenario(
            identifier,
            submit_path,
            submit_reviewed,
            original=_sha256(submit_old),
            pins={CLIENTS["entry"][1]: _sha256(submit_reviewed)},
        ),
        "Unreviewed public entry change",
    )

    identifier = "submit_changed_after_review"
    later_submit = b"submit client changed after review\r\n"
    reject(
        identifier,
        _scenario(
            identifier,
            submit_path,
            later_submit,
            original=_sha256(submit_old),
            pins={submit_pin: _sha256(submit_reviewed)},
        ),
        submit_error,
    )

    identifier = "submit_review_pin_rejects_bom_byte_drift"
    reject(
        identifier,
        _scenario(
            identifier,
            submit_path,
            b"\xef\xbb\xbf" + submit_reviewed,
            original=_sha256(submit_old),
            pins={submit_pin: _sha256(submit_reviewed)},
        ),
        submit_error,
    )

    identifier = "submit_first_install_rejects_wrong_provided_pin"
    reject(
        identifier,
        _scenario(
            identifier,
            submit_path,
            submit_reviewed,
            pins={submit_pin: _sha256(b"wrong submit first-install pin")},
        ),
        submit_error,
    )

    identifier = "submit_pin_is_not_a_frontend_pin"
    frontend_reviewed = REVIEWED_BYTES["frontend"]
    reject(
        identifier,
        _scenario(
            identifier,
            CLIENTS["frontend"][0],
            frontend_reviewed,
            original=_sha256(OLD_BYTES["frontend"]),
            pins={submit_pin: _sha256(frontend_reviewed)},
        ),
        "Unreviewed public entry change",
    )

    unsupported_path = "tools/jenkins/experimental-client.py"
    unsupported_old = b"old unsupported client\n"
    identifier = "unsupported_unchanged_raw"
    accept(
        identifier,
        _scenario(identifier, unsupported_path, unsupported_old, original=_sha256(unsupported_old)),
        _sha256(unsupported_old),
    )

    unsupported_new = b"changed unsupported client\n"
    identifier = "unsupported_changed_cannot_borrow_submit_pin"
    reject(
        identifier,
        _scenario(
            identifier,
            unsupported_path,
            unsupported_new,
            original=_sha256(unsupported_old),
            pins={submit_pin: _sha256(unsupported_new)},
        ),
        "Unreviewed public entry change",
    )

    lf_entry = b"entry line one\nentry line two\n"
    crlf_entry = b"entry line one\r\nentry line two\r\n"
    identifier = "entry_legacy_lf_normalization"
    accept(
        identifier,
        _scenario(identifier, CLIENTS["entry"][0], crlf_entry, original=_sha256(lf_entry)),
        _sha256(crlf_entry),
    )

    identifier = "entry_explicit_lf_pin_cannot_use_legacy_normalization"
    reject(
        identifier,
        _scenario(
            identifier,
            CLIENTS["entry"][0],
            crlf_entry,
            original=_sha256(lf_entry),
            pins={CLIENTS["entry"][1]: _sha256(lf_entry)},
        ),
        CLIENTS["entry"][2],
    )

    for name in ("local_cargo", "frontend", "submit"):
        identifier = f"{name}_does_not_inherit_entry_lf_normalization"
        reject(
            identifier,
            _scenario(
                identifier,
                CLIENTS[name][0],
                crlf_entry,
                original=_sha256(lf_entry),
            ),
            "Unreviewed public entry change",
        )

    return cases, expectations


def _powershell_command(cases: list[dict[str, object]]) -> str:
    encoded_cases = base64.b64encode(
        json.dumps(cases, separators=(",", ":"), ensure_ascii=True).encode("ascii")
    ).decode("ascii")
    encoded_installer = base64.b64encode(str(INSTALLER).encode("utf-8")).decode("ascii")
    return f"""
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSEdition -ne 'Desktop') {{ throw 'Windows PowerShell Desktop is required' }}
$installerPath = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded_installer}'))
$tokens = $null
$parseErrors = $null
$installerAst = [System.Management.Automation.Language.Parser]::ParseFile(
    $installerPath, [ref]$tokens, [ref]$parseErrors)
if ($null -eq $installerAst) {{ throw 'Canonical installer did not produce an AST' }}
if ($parseErrors.Count -ne 0) {{ throw ('Canonical installer parse failed: ' + ($parseErrors -join '; ')) }}
$hashFunctions = @($installerAst.FindAll({{
    param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
        $node.Name -ceq 'Hash-Bytes'
}}, $true))
$gateFunctions = @($installerAst.FindAll({{
    param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
        $node.Name -ceq 'Get-ValidatedManagementClientSha256'
}}, $true))
if ($hashFunctions.Count -ne 1) {{ throw ('Expected one Hash-Bytes definition; found ' + $hashFunctions.Count) }}
if ($gateFunctions.Count -ne 1) {{ throw ('Expected one Get-ValidatedManagementClientSha256 definition; found ' + $gateFunctions.Count) }}
$clientSourceFunctions = @($installerAst.FindAll({{
    param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
        $node.Name -ceq 'Get-ManagementClientSource'
}}, $true))
if ($clientSourceFunctions.Count -ne 1) {{ throw 'Expected one Get-ManagementClientSource definition' }}
$definitions = $hashFunctions[0].Extent.Text + [Environment]::NewLine + $gateFunctions[0].Extent.Text +
    [Environment]::NewLine + $clientSourceFunctions[0].Extent.Text
$isolatedFunctions = [ScriptBlock]::Create($definitions)
. $isolatedFunctions

$caseJson = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded_cases}'))
$cases = ConvertFrom-Json -InputObject $caseJson
$results = New-Object 'System.Collections.Generic.List[object]'
$optionalNames = @(
    'OriginalSha256',
    'ExpectedEntrySourceSha256',
    'ExpectedLocalCargoSourceSha256',
    'ExpectedFrontendSourceSha256',
    'ExpectedSubmitSourceSha256'
)
foreach ($case in $cases) {{
    $parameters = @{{
        ClientName = [string]$case.ClientName
        ClientBytes = [byte[]][Convert]::FromBase64String([string]$case.BytesBase64)
    }}
    foreach ($name in $optionalNames) {{
        $property = $case.PSObject.Properties[$name]
        if ($null -ne $property) {{ $parameters[$name] = $property.Value }}
    }}
    try {{
        $values = @(Get-ValidatedManagementClientSha256 @parameters)
        if ($values.Count -ne 1) {{ throw ('Expected one gate result; found ' + $values.Count) }}
        $results.Add([ordered]@{{ id=[string]$case.Id; status='accepted'; value=[string]$values[0]; error=$null }}) | Out-Null
    }} catch {{
        $results.Add([ordered]@{{ id=[string]$case.Id; status='rejected'; value=$null; error=$_.Exception.Message }}) | Out-Null
    }}
}}
# Exercise the installed/candidate locator from the parsed production function.
function Assert-Physical([string]$Path) {{ return $Path }}
$repo = 'E:/client-source-memory'
$taskOldSourceRoot = "$repo/.jenkins/runtime/management/old"
$taskCandidateSources = $null
$AnchorManagementClients = $false
$taskAnchoredClients = $false
$name = 'tools/jenkins-management/entry.py'
$clientPaths = [ordered]@{{ legacy=(Get-ManagementClientSource $name) }}
$taskAnchoredClients = $true
$clientPaths.anchored = Get-ManagementClientSource $name
$AnchorManagementClients = $true
$taskCandidateSources = [pscustomobject]@{{ clientSources=[pscustomobject]@{{}} }}
$taskCandidateSources.clientSources | Add-Member -NotePropertyName $name -NotePropertyValue "$repo/reviewed/$name"
$clientPaths.candidate = Get-ManagementClientSource $name
$taskCandidateSources = $null
try {{ Get-ManagementClientSource $name | Out-Null; $clientPaths.missing='accepted' }}
catch {{ $clientPaths.missing=$_.Exception.Message }}
$AnchorManagementClients = $false
$MigrateCanonicalManagementClients = $true
$taskCandidateSources = [pscustomobject]@{{ clientSources=[pscustomobject]@{{}} }}
$taskCandidateSources.clientSources | Add-Member -NotePropertyName $name -NotePropertyValue "$repo/canonical/$name"
$clientPaths.canonical = Get-ManagementClientSource $name
$taskCandidateSources = $null
try {{ Get-ManagementClientSource $name | Out-Null; $clientPaths.canonicalMissing='accepted' }}
catch {{ $clientPaths.canonicalMissing=$_.Exception.Message }}
$payload = [ordered]@{{
    clientPaths=$clientPaths
    count=$results.Count
    hashVector=(Hash-Bytes ([byte[]](97, 98, 99)))
    results=@($results.ToArray())
}}
$json = ConvertTo-Json -InputObject $payload -Depth 8 -Compress
[Console]::Out.WriteLine('ZR_CLIENT_PIN_RESULTS:' + $json)
"""


class InstallerClientPinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        if powershell is None:
            raise RuntimeError("Windows PowerShell Desktop is required; this is not a passing skip")

        cls._cases, cls._expectations = _scenarios()
        completed = subprocess.run(
            [powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", _powershell_command(cls._cases)],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=60,
            check=False,
        )
        if completed.returncode != 0:
            raise RuntimeError(
                f"PowerShell AST/gate runner exited {completed.returncode}: {completed.stderr.strip()}"
            )
        output_lines = completed.stdout.splitlines()
        marked = [line for line in output_lines if line.startswith(RESULT_MARKER)]
        if len(output_lines) != 1 or len(marked) != 1:
            raise RuntimeError(
                "PowerShell AST/gate runner did not emit exactly one completion marker "
                f"(stdout lines={len(output_lines)}, marked={len(marked)})"
            )
        try:
            cls._response = json.loads(marked[0][len(RESULT_MARKER) :])
        except json.JSONDecodeError as error:
            raise RuntimeError("PowerShell AST/gate completion marker contained invalid JSON") from error

        expected_count = len(cls._cases)
        if expected_count != 37:
            raise RuntimeError(f"Expected 37 installer pin contract cases; found {expected_count}")
        results = cls._response.get("results")
        if cls._response.get("count") != expected_count or not isinstance(results, list) or len(results) != expected_count:
            raise RuntimeError(
                f"PowerShell AST/gate runner returned an incomplete result set: expected {expected_count}"
            )
        cls._rows = {row["id"]: row for row in results}
        if len(cls._rows) != expected_count or set(cls._rows) != set(cls._expectations):
            raise RuntimeError("PowerShell AST/gate runner returned missing or duplicate case identifiers")

    def _assert_accepted(self, identifier: str, expected_hash: str) -> None:
        row = self._rows[identifier]
        self.assertEqual("accepted", row["status"], row)
        self.assertEqual(expected_hash, row["value"], row)
        self.assertIsNone(row["error"], row)

    def _assert_rejected(self, identifier: str, expected_message: str) -> None:
        row = self._rows[identifier]
        self.assertEqual("rejected", row["status"], row)
        self.assertEqual(expected_message, row["error"], row)
        self.assertIsNone(row["value"], row)

    def test_hash_bytes_matches_an_independent_sha256_vector(self) -> None:
        self.assertEqual(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            self._response["hashVector"],
        )

    def test_first_install_and_unchanged_raw_pins_remain_valid(self) -> None:
        for name in CLIENTS:
            self._assert_accepted(f"first_install_{name}", _sha256(OLD_BYTES[name]))
            self._assert_accepted(f"unchanged_raw_{name}", _sha256(OLD_BYTES[name]))
        self._assert_accepted("unsupported_unchanged_raw", _sha256(b"old unsupported client\n"))
        name = "tools/jenkins-management/entry.py"
        root = "E:/client-source-memory"
        self.assertEqual({
            "legacy": f"{root}/{name}",
            "anchored": f"{root}/.jenkins/runtime/management/old/clients/{name}",
            "candidate": f"{root}/reviewed/{name}",
            "missing": "Reviewed candidate client is missing",
                "canonical": root + "/canonical/tools/jenkins-management/entry.py",
                "canonicalMissing": "Reviewed candidate client is missing",
        }, self._response["clientPaths"])

    def test_reviewed_hash_is_bound_to_each_exact_supported_client(self) -> None:
        for name in CLIENTS:
            self._assert_accepted(f"reviewed_change_{name}", _sha256(REVIEWED_BYTES[name]))
        self._assert_rejected(
            "submit_pin_is_not_a_frontend_pin",
            "Unreviewed public entry change",
        )

    def test_changed_or_unsupported_clients_cannot_borrow_a_submit_pin(self) -> None:
        self._assert_rejected("changed_submit_without_review", "Unreviewed public entry change")
        self._assert_rejected(
            "unsupported_changed_cannot_borrow_submit_pin",
            "Unreviewed public entry change",
        )
        self._assert_rejected(
            "submit_changed_after_review",
            "ExpectedSubmitSourceSha256 does not match current raw bytes",
        )
        self._assert_rejected(
            "submit_entry_pin_does_not_authorize_submit_change",
            "Unreviewed public entry change",
        )
        self._assert_rejected(
            "submit_review_pin_rejects_bom_byte_drift",
            "ExpectedSubmitSourceSha256 does not match current raw bytes",
        )
        self._assert_rejected(
            "submit_first_install_rejects_wrong_provided_pin",
            "ExpectedSubmitSourceSha256 does not match current raw bytes",
        )

    def test_wrong_empty_and_null_provided_pins_never_fall_back_to_old_raw(self) -> None:
        for _name, (_path, _pin_name, expected_error) in CLIENTS.items():
            self._assert_rejected(f"wrong_pin_old_raw_{_name}", expected_error)
            self._assert_rejected(f"empty_pin_old_raw_{_name}", expected_error)
            self._assert_rejected(f"null_pin_old_raw_{_name}", expected_error)

    def test_entry_alone_keeps_legacy_crlf_normalization(self) -> None:
        lf = b"entry line one\nentry line two\n"
        crlf = b"entry line one\r\nentry line two\r\n"
        self._assert_accepted("entry_legacy_lf_normalization", _sha256(crlf))
        self._assert_rejected(
            "local_cargo_does_not_inherit_entry_lf_normalization",
            "Unreviewed public entry change",
        )
        self._assert_rejected(
            "frontend_does_not_inherit_entry_lf_normalization",
            "Unreviewed public entry change",
        )
        self._assert_rejected(
            "submit_does_not_inherit_entry_lf_normalization",
            "Unreviewed public entry change",
        )
        self._assert_rejected(
            "entry_explicit_lf_pin_cannot_use_legacy_normalization",
            "Reviewed public entry pin differs",
        )
        self.assertNotEqual(_sha256(lf), _sha256(crlf))



@unittest.skipUnless(
    os.environ.get("JOB_NAME") == "zircon-management-tests"
    and os.environ.get("ZR_CANONICAL_MIGRATION_TEST_ROOT"),
    "Physical canonical migration probes run in the registered Jenkins fixture root",
)
class CanonicalClientMigrationTests(unittest.TestCase):
    """Exercise the real loader with actual sources; inject read faults without editing them."""

    @classmethod
    def setUpClass(cls):
        cls.root = INSTALLER.parents[2]
        cls.fixture = Path(os.environ["ZR_CANONICAL_MIGRATION_TEST_ROOT"]) / "canonical-manifests"
        cls.fixture.mkdir()
        installed_path = cls.root / ".jenkins/state/management/installed.json"
        installed_raw = installed_path.read_bytes()
        cls.installed = json.loads(installed_raw)
        cls.capsule = Path(cls.installed["capsule"])
        cls.metadata = json.loads((cls.capsule / "capsule.json").read_bytes())
        cls.clients = (
            "tools/dev/local_cargo.py", "tools/dev/local-cargo.ps1", "tools/dev/dev-fast-build.ps1",
            "tools/dev/submit-jenkins-management.ps1", "tools/jenkins/frontend.py",
            "tools/jenkins-management/entry.py",
        )
        cls.members = ("runner.py", "source_preparation.py", "zircon-management.groovy",
                       "checkout_guard.py", "checkout_product.py")
        cls.addons = {
            "ExpectedBranchIntegrationAddonSha256": "branch-integration.groovy",
            "ExpectedOldBranchCleanupAddonSha256": "branch_cleanup.groovy",
            "ExpectedDeliveryAddonSha256": "delivery.groovy",
            "ExpectedCheckoutCommitAddonSha256": "checkout-commit.groovy",
        }
        sha = lambda data: hashlib.sha256(data).hexdigest()
        cls.prefix = "".join(
            (cls.root / "tools/jenkins-management" / name).read_text(encoding="utf-8-sig")
            .replace("\r\n", "\n").rstrip("\r\n") + "\n\n"
            for name in cls.addons.values()
        )
        cls.manifest = {
            "schemaVersion": 1,
            "kind": "jenkins-management-canonical-client-migration-v1",
            "repositoryRoot": str(cls.root),
            "originalInstalledSha256": sha(installed_raw),
            "originalCapsuleSha256": cls.installed["capsuleSha256"],
            "driverDigest": cls.installed["driverDigest"],
            "runtimeOperationId": cls.installed["runtimeOperationId"],
            "installerSha256": sha(INSTALLER.read_bytes()),
            "files": {"tools/jenkins-management/" + name:
                      sha((cls.root / "tools/jenkins-management" / name).read_bytes())
                      for name in cls.members},
            "clientSources": {name: sha((cls.root / name).read_bytes()) for name in cls.clients},
            "addonSources": {key: sha((cls.root / "tools/jenkins-management" / name).read_bytes())
                             for key, name in cls.addons.items()},
            "combinedExtensionPrefixSha256": sha(cls.prefix.encode()),
        }
        source = INSTALLER.read_text(encoding="utf-8-sig")
        matches = re.findall(r"\$taskCandidateBootstrap = @'\n(.*?)\n'@", source, re.S)
        if len(matches) != 1:
            raise AssertionError("The actual installer must provide one shared reviewed-source loader")
        cls.bootstrap = matches[0]
        cls.cases = []
        cls.groups = {}
        def add(group, name, changed=None, *, raw=None, fault=None, canonical=True, expect="rejected", addon_args=None):
            value = copy.deepcopy(cls.manifest)
            if changed:
                changed(value)
            data = raw if raw is not None else json.dumps(value, separators=(",", ":")).encode()
            path = cls.fixture / (name + ".json")
            with path.open("xb") as stream:
                stream.write(data)
            item = {"id": name, "path": str(path), "sha256": sha(data),
                    "canonical": canonical, "fault": fault, "expected": expect}
            if addon_args is not None:
                item["addonArgs"] = addon_args
            cls.groups.setdefault(group, []).append(name)
            cls.cases.append(item)
        add("valid", "canonical-valid", expect="accepted")
        addon_keys = tuple(cls.manifest["addonSources"])
        for mask in range(16):
            subset = {key: cls.manifest["addonSources"][key] for index, key in enumerate(addon_keys)
                      if mask & (1 << index)}
            add("optionalAddons", "addon-subset-" + str(mask), addon_args=subset, expect="accepted")
        add("badAddonArgs", "addon-unknown", addon_args={"unreviewed": "0" * 64})
        add("badAddonArgs", "addon-conflict", addon_args={addon_keys[0]: "0" * 64})
        add("badAddonArgs", "addon-wrong-type", addon_args={addon_keys[0]: True})
        for key in ("schemaVersion", "kind", "repositoryRoot", "originalInstalledSha256",
                    "originalCapsuleSha256", "driverDigest", "runtimeOperationId"):
            wrong = True if key == "schemaVersion" else "wrong-original-identity"
            add("identity", "wrong-" + key, lambda value, key=key, wrong=wrong: value.__setitem__(key, wrong))
        for key in tuple(cls.manifest):
            add("schema", "missing-" + key, lambda value, key=key: value.pop(key))
        add("schema", "extra-top-level", lambda value: value.__setitem__("unreviewed", True))
        for key in ("files", "clientSources", "addonSources"):
            add("schema", "wrong-type-" + key, lambda value, key=key: value.__setitem__(key, []))
            for change in ("missing", "extra", "wrong-pin", "wrong-pin-type"):
                def mutate(value, key=key, change=change):
                    mapping = value[key]
                    first = next(iter(mapping))
                    if change == "missing":
                        mapping.pop(first)
                    elif change == "extra":
                        mapping["../outside.py"] = "0" * 64
                    elif change == "wrong-pin":
                        mapping[first] = "0" * 64
                    else:
                        mapping[first] = True
                add("schema", key + "-" + change, mutate)
        add("schema", "wrong-installer", lambda value: value.__setitem__("installerSha256", "0" * 64))
        add("schema", "malformed-prefix", lambda value: value.__setitem__("combinedExtensionPrefixSha256", "0" * 63))
        raw = json.dumps(cls.manifest, separators=(",", ":")).encode()
        add("duplicates", "duplicate-top", raw=b'{"schemaVersion":1,' + raw[1:])
        for key in ("files", "clientSources", "addonSources"):
            first = next(iter(cls.manifest[key]))
            marker = ('"' + key + '":{').encode()
            pair = json.dumps(first).encode() + b":" + json.dumps(cls.manifest[key][first]).encode() + b","
            duplicate = raw.replace(marker, marker + pair, 1)
            add("duplicates", "duplicate-" + key, raw=duplicate)
        add("duplicates", "managed-shared-duplicate", raw=b'{"schemaVersion":1,' + raw[1:], canonical=False)
        add("drift", "raw-manifest-sha", fault={"kind": "manifest"})
        for relative in [*cls.clients, *cls.manifest["files"],
                         *("tools/jenkins-management/" + name for name in cls.addons.values()),
                         "tools/dev/install-jenkins-management.ps1"]:
            add("drift", "source-drift-" + str(len(cls.cases)),
                fault={"kind": "read", "path": str(cls.root / relative)})
        for relative in [*cls.clients, *cls.metadata["files"]]:
            old_path = cls.capsule / ("clients/" + relative if relative in cls.clients else relative)
            add("oldIntegrity", "old-drift-" + str(len(cls.cases)),
                fault={"kind": "read", "path": str(old_path)})
        for path in (cls.root / cls.clients[0], cls.capsule / "clients" / cls.clients[0]):
            add("physical", "escape-" + str(len(cls.cases)),
                fault={"kind": "physical", "path": str(path)})
        add("physical", "manifest-physical", fault={"kind": "manifestPhysical"})
        for stage in ("before-publication", "before-post", "after-readback"):
            add("repeat", stage, fault={"kind": "repeat", "path": str(cls.root / cls.clients[0])})
        protected = [installed_path, cls.capsule / "capsule.json",
                     *(cls.capsule / name for name in cls.metadata["files"]),
                     *(cls.capsule / "clients" / name for name in cls.clients),
                     cls.root / ".jenkins/jenkins_home/jobs/zircon-management/config.xml"]
        before = {str(path): sha(path.read_bytes()) for path in protected}
        payload = {"repo": str(cls.root), "bootstrap": cls.bootstrap, "cases": cls.cases,
                   "installed": cls.manifest["originalInstalledSha256"],
                   "driver": cls.installed["driverDigest"], "runtime": cls.installed["runtimeOperationId"],
                   "capsule": cls.installed["capsuleSha256"],
                   "addons": cls.manifest["addonSources"]}
        worker = r"""
import base64,contextlib,hashlib,io,json,sys
from pathlib import Path
payload=json.load(sys.stdin)
source=compile(payload["bootstrap"],"<actual-installer-reviewed-source-loader>","exec")
read_bytes=Path.read_bytes
resolve=Path.resolve
results=[]
for case in payload["cases"]:
    path=Path(case["path"])
    fault=case.get("fault") or {}
    target=Path(fault["path"]) if "path" in fault else None
    if fault.get("kind")=="manifest":
        with path.open("ab") as stream: stream.write(b"\n")
    def injected_read(item):
        result=read_bytes(item)
        return result+b"\n# injected read drift\n" if item==target else result
    def injected_resolve(item,*args,**kwargs):
        if item==target or (fault.get("kind")=="manifestPhysical" and item==path):
            return Path(payload["repo"])/"outside-the-reviewed-physical-path"
        return resolve(item,*args,**kwargs)
    def execute():
        sys.argv=["installer-reviewed-source",payload["repo"],payload["repo"],str(path),case["sha256"],
                  payload["installed"],payload["driver"],payload["runtime"],payload["capsule"],base64.b64encode(b"[]").decode("ascii"),"false",
                  json.dumps(case["canonical"]),"true",base64.b64encode(json.dumps(case.get("addonArgs", payload["addons"])).encode()).decode("ascii")]
        output=io.StringIO()
        try:
            with contextlib.redirect_stdout(output):
                exec(source,{"__name__":"__main__"})
        except SystemExit as error:
            if error.code not in (None,0): raise
        return json.loads(output.getvalue())
    try:
        if fault.get("kind")=="repeat":
            first=execute()
            assert len(first["clientSources"])==6
            Path.read_bytes=injected_read
        elif fault.get("kind")=="read":
            Path.read_bytes=injected_read
        elif fault.get("kind") in ("physical","manifestPhysical"):
            Path.resolve=injected_resolve
        value=execute()
        results.append({"id":case["id"],"status":"accepted","value":value})
    except Exception as error:
        results.append({"id":case["id"],"status":"rejected","error":type(error).__name__+":"+str(error)})
    finally:
        Path.read_bytes=read_bytes
        Path.resolve=resolve
print(json.dumps(results))
"""
        completed = subprocess.run([sys.executable, "-B", "-X", "utf8", "-c", worker],
                                   input=json.dumps(payload), capture_output=True, text=True,
                                   encoding="utf-8", timeout=180)
        if completed.returncode:
            raise AssertionError("Actual loader probes failed: " + completed.stderr[-5000:])
        cls.results = {item["id"]: item for item in json.loads(completed.stdout)}
        if len(cls.results) != len(cls.cases):
            raise AssertionError("Every unique actual loader probe must return once")
        after = {str(path): sha(path.read_bytes()) for path in protected}
        if before != after:
            raise AssertionError("Read-only migration failures changed protected installed/capsule/job bytes")

    def assert_rejected_group(self, group):
        for case in self.groups[group]:
            with self.subTest(case=case):
                self.assertEqual("rejected", self.results[case]["status"], self.results[case])

    def test_actual_canonical_six_sources_and_complete_members_are_selected(self):
        result = self.results["canonical-valid"]
        self.assertEqual("accepted", result["status"], result)
        value = result["value"]
        self.assertEqual({name: str(self.root / name) for name in self.clients}, value["clientSources"])
        self.assertEqual(self.manifest["clientSources"], value["clientSourcePins"])
        self.assertEqual({name: str(self.root / "tools/jenkins-management" / name) for name in self.members},
                         value["files"])
        self.assertEqual(self.manifest["combinedExtensionPrefixSha256"], value["combinedExtensionPrefixSha256"])
        self.assertEqual(set(self.clients), set(value["originalClients"]))
        self.assertEqual({name: {"path": str(self.capsule / "clients" / name),
                                 "sha256": self.metadata["clientSources"][name]}
                          for name in self.clients}, value["originalClients"])
        for name in self.clients:
            self.assertNotEqual(value["clientSources"][name], value["originalClients"][name]["path"])

    def test_addon_cli_pins_are_optional_consistency_constraints(self):
        for case in self.groups["optionalAddons"]:
            with self.subTest(case=case):
                result = self.results[case]
                self.assertEqual("accepted", result["status"], result)
                self.assertEqual(self.manifest["addonSources"], result["value"]["addonSourcePins"])
        self.assert_rejected_group("badAddonArgs")

    def test_shared_prefix_state_preserves_default_subsets_and_manifest_order(self):
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        fixture = self.fixture / "prefix-state-input.json"
        with fixture.open("xb") as stream:
            stream.write(json.dumps({"repo": str(self.root), "installer": str(INSTALLER),
                                     "addons": self.manifest["addonSources"],
                                     "prefix": self.prefix}).encode())
        encoded = base64.b64encode(str(fixture).encode()).decode()
        program = r"""
$ErrorActionPreference='Stop'
$inputPath=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('INPUT_BASE64'))
$inputData=Get-Content -Raw -LiteralPath $inputPath|ConvertFrom-Json
$repo=$inputData.repo;$taskPrior=$true
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($inputData.installer,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Actual installer parse failed'}
foreach($name in @('Assert-Physical','Hash-Bytes','Get-ManagementJobExtensionPrefix','Get-ManagementJobExtensionState')){
 $f=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $name},$true))
 if($f.Count -ne 1){throw 'Actual prefix helper missing'}
 . ([scriptblock]::Create($f[0].Extent.Text))
}
$all=[ordered]@{};foreach($p in $inputData.addons.PSObject.Properties){$all[$p.Name]=[string]$p.Value}
$keys=@('ExpectedBranchIntegrationAddonSha256','ExpectedOldBranchCleanupAddonSha256','ExpectedDeliveryAddonSha256','ExpectedCheckoutCommitAddonSha256')
$pieces=@{}
foreach($key in $keys){$single=[ordered]@{};$single[$key]=$all[$key];$pieces[$key]=(Get-ManagementJobExtensionState -AddonPins $single).combinedPrefix}
$count=0
for($mask=0;$mask -lt 16;$mask++){
 $subset=[ordered]@{};$expected=''
 for($i=0;$i -lt 4;$i++){if($mask -band (1 -shl $i)){$subset[$keys[$i]]=$all[$keys[$i]];$expected+=$pieces[$keys[$i]]}}
 $state=Get-ManagementJobExtensionState -AddonPins $subset
 if($state.combinedPrefix -cne $expected){throw 'Default subset or ordering changed'}
 $count++
}
$full=Get-ManagementJobExtensionState -AddonPins $all -ExpectedCombinedSha256 (Hash-Bytes ([Text.Encoding]::UTF8.GetBytes($inputData.prefix)))
if($full.combinedPrefix -cne $inputData.prefix){throw 'Manifest full prefix differs'}
$bad=[ordered]@{};foreach($key in $keys){$bad[$key]=$all[$key]};$bad[$keys[0]]='0'*64
$reject=0
foreach($mode in @('wrong-source','wrong-combined','unknown','no-prior')){
 try{
  if($mode -ceq 'wrong-source'){Get-ManagementJobExtensionState -AddonPins $bad|Out-Null}
  elseif($mode -ceq 'wrong-combined'){Get-ManagementJobExtensionState -AddonPins $all -ExpectedCombinedSha256 ('0'*64)|Out-Null}
  elseif($mode -ceq 'unknown'){Get-ManagementJobExtensionState -AddonPins @{unknown='0'*64}|Out-Null}
  else{$taskPrior=$false;Get-ManagementJobExtensionState -AddonPins $all|Out-Null}
  throw 'UNEXPECTED_ACCEPTANCE'
 }catch{if($_.Exception.Message -ceq 'UNEXPECTED_ACCEPTANCE'){throw};$reject++}
 finally{$taskPrior=$true}
}
[Console]::Out.WriteLine(($count.ToString()+':'+$reject.ToString()))
""".replace("INPUT_BASE64", encoded)
        completed = subprocess.run([powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", program],
                                   capture_output=True, text=True, encoding="utf-8", timeout=60)
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual("16:4", completed.stdout.strip())

    def test_full_entry_rejects_a_valid_format_wrong_manifest_prefix(self):
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        value = copy.deepcopy(self.manifest)
        value["combinedExtensionPrefixSha256"] = "0" * 64
        raw = json.dumps(value, separators=(",", ":")).encode()
        manifest = self.fixture / "full-entry-wrong-prefix.json"
        with manifest.open("xb") as stream:
            stream.write(raw)
        base_command = [powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass",
                        "-File", str(INSTALLER), "-RepoRoot", str(self.root),
                        "-ExpectedDriverDigest", self.installed["driverDigest"],
                        "-ExpectedRuntimeOperationId", self.installed["runtimeOperationId"],
                        "-ExpectedInstalledSha256", self.manifest["originalInstalledSha256"],
                        "-EnableCheckoutProduct", "-EnableManagedWorktree", "-CheckOnly",
                        "-MigrateCanonicalManagementClients", "-CanonicalClientSourceManifestPath", str(manifest),
                        "-ExpectedCanonicalClientSourceManifestSha256", hashlib.sha256(raw).hexdigest()]
        protected = [self.root / ".jenkins/state/management/installed.json",
                     self.capsule / "capsule.json",
                     self.root / ".jenkins/jenkins_home/jobs/zircon-management/config.xml"]
        before = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in protected}
        for pins in ({}, self.manifest["addonSources"]):
            with self.subTest(provided_addons=len(pins)):
                command = list(base_command)
                for key, checksum in pins.items():
                    command.extend(["-" + key, checksum])
                completed = subprocess.run(command, capture_output=True, timeout=120)
                for name, data in (("command.json", json.dumps(command).encode()),
                                   ("stdout.bin", completed.stdout), ("stderr.bin", completed.stderr)):
                    label = "full-entry-wrong-prefix-" + str(len(pins)) + "-" + name
                    with (self.fixture / label).open("xb") as stream:
                        stream.write(data)
                self.assertNotEqual(0, completed.returncode)
                error = completed.stderr.decode("utf-8-sig", errors="replace")
                self.assertIn("Canonical combined addon prefix differs", error)
                self.assertNotIn("not enough values to unpack", error)
                self.assertNotIn("JSONDecodeError", error)
                self.assertEqual(before, {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                                          for path in protected})

    def test_manifest_identity_and_schema_are_exact(self):
        self.assert_rejected_group("identity")
        self.assert_rejected_group("schema")

    def test_duplicate_keys_are_rejected_by_the_shared_loader(self):
        self.assert_rejected_group("duplicates")
        for case in self.groups["duplicates"]:
            self.assertIn("candidate_manifest_duplicate_key", self.results[case]["error"])

    def test_every_reviewed_source_and_manifest_raw_digest_rejects_drift(self):
        self.assert_rejected_group("drift")

    def test_old_immutable_clients_and_members_are_verified_before_switch(self):
        self.assert_rejected_group("oldIntegrity")

    def test_physical_source_and_manifest_escape_is_rejected(self):
        self.assert_rejected_group("physical")

    def test_repeated_source_checks_reject_drift_without_a_new_plan(self):
        self.assert_rejected_group("repeat")

    def test_canonical_argument_group_and_mode_exclusion_fail_before_source_selection(self):
        powershell = shutil.which("powershell.exe") or shutil.which("powershell")
        installer = base64.b64encode(str(INSTALLER).encode()).decode()
        program = r"""
$ErrorActionPreference='Stop'
$path=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('INSTALLER_BASE64'))
$tokens=$null;$errors=$null
$ast=[Management.Automation.Language.Parser]::ParseFile($path,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Actual installer parse failed'}
$f=@($ast.FindAll({param($n) $n -is [Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq 'Get-ReviewedCandidateSources'},$true))
if($f.Count -ne 1){throw 'Actual reviewed source function missing'}
. ([scriptblock]::Create($f[0].Extent.Text))
$groups=@(
 @('CanonicalClientSourceManifestPath'),
 @('MigrateCanonicalManagementClients','CanonicalClientSourceManifestPath'),
 @('MigrateCanonicalManagementClients','ExpectedCanonicalClientSourceManifestSha256'))
$exclusive=@('CandidateSourceRoot','CandidateSourceManifestPath','ExpectedCandidateSourceManifestSha256',
 'ExpectedCandidateCapsulePaths','AnchorManagementClients','ReuseCapsuleSha256',
 'PinnedActiveBuildProofPath','ExpectedPinnedActiveBuildProofSha256','ExpectedPinnedActiveGuardSha256',
 'ExpectedRetainedPrefixSha256')
foreach($name in $exclusive){$groups+=,@('MigrateCanonicalManagementClients','CanonicalClientSourceManifestPath','ExpectedCanonicalClientSourceManifestSha256',$name)}
$count=0
foreach($group in $groups){
 $script:taskInstallerParameters=@{}
 foreach($name in $group){$script:taskInstallerParameters[$name]='present'}
 $MigrateCanonicalManagementClients=$true;$CanonicalClientSourceManifestPath='unread-input.json'
 $ExpectedCanonicalClientSourceManifestSha256='a'*64
 try{Get-ReviewedCandidateSources|Out-Null;throw 'UNEXPECTED_ACCEPTANCE'}
 catch{
  if($_.Exception.Message -notmatch 'Canonical migration (parameters must be supplied together|cannot mix source or pinned-active modes)'){throw}
  $count++
 }
}
[Console]::Out.WriteLine($count)
""".replace("INSTALLER_BASE64", installer)
        completed = subprocess.run([powershell, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", program],
                                   capture_output=True, text=True, encoding="utf-8", timeout=60)
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual(13, int(completed.stdout.strip()))


if __name__ == "__main__":
    unittest.main()
