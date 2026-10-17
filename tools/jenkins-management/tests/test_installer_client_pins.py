"""In-memory contract tests for the public installer client pin gate."""
from __future__ import annotations

import base64
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
$definitions = $hashFunctions[0].Extent.Text + [Environment]::NewLine + $gateFunctions[0].Extent.Text
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
$payload = [ordered]@{{
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


if __name__ == "__main__":
    unittest.main()
