"""A runner-only update preserves installed helpers and registered scope checks."""
import ast
from pathlib import Path
import subprocess
import unittest

INSTALLER = Path(__file__).absolute().parents[2] / "dev/install-jenkins-management.ps1"


class CandidateUpdateTests(unittest.TestCase):
    def members(self, proof):
        source = INSTALLER.read_text(encoding="utf-8")
        bootstrap = source.split("$taskCandidateBootstrap = @'\n", 1)[1].split("\n'@", 1)[0]
        tree = ast.parse(bootstrap)
        nodes = [n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name in {"require", "candidate_members"}]
        self.assertEqual(len(nodes), 2)
        context = {}
        exec(compile(ast.Module(body=nodes, type_ignores=[]), str(INSTALLER), "exec"), context)
        return context["candidate_members"](proof)

    def test_runner_only_and_directory_writer_scope(self):
        self.assertEqual(self.members({"files": {"tools/jenkins-management/runner.py": "a" * 64},
            "ownedPaths": ["tools", "docs"]}), {"runner.py": "tools/jenkins-management/runner.py"})

    def test_exact_paths_and_both_members_still_work(self):
        names = ["tools/jenkins-management/runner.py", "tools/jenkins-management/source_preparation.py"]
        self.assertEqual(set(self.members({"files": dict.fromkeys(names, "a" * 64),
            "ownedPaths": names + ["tools/dev/install-jenkins-management.ps1"]})), {"runner.py", "source_preparation.py"})

    def test_foreign_paths_and_sibling_prefix_do_not_gain_authority(self):
        for proof in ({"files": {"tools/jenkins-management/runner.py": "a"}, "ownedPaths": ["tool"]},
                      {"files": {"tools/jenkins-management/foreign.py": "a"}, "ownedPaths": ["tools"]},
                      {"files": {}, "ownedPaths": ["tools"]}):
            with self.subTest(proof=proof), self.assertRaisesRegex(ValueError, "candidate_member_scope_invalid"):
                self.members(proof)

    def test_real_powershell_source_selection_preserves_unspecified_helper(self):
        # Execute the production selector, rather than reimplementing it in Python.
        command = r'''
$ErrorActionPreference='Stop'
$tokens=$null; $errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile('__INSTALLER__',[ref]$tokens,[ref]$errors)
if ($errors.Count) { throw 'Installer parse failed' }
$functions=@($tree.FindAll({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Get-ManagementCapsuleSource'},$true))
if ($functions.Count -ne 1) { throw 'Source selector missing' }
. ([ScriptBlock]::Create($functions[0].Extent.Text))
function Assert-Physical([string]$Path) { return $Path }
$taskSourceRoot='E:/retained-installed-capsule'
$taskCandidateSources=[pscustomobject]@{files=[pscustomobject]@{'runner.py'='E:/owned-candidate/runner.py'}}
if ((Get-ManagementCapsuleSource 'runner.py') -cne 'E:/owned-candidate/runner.py') { throw 'Owned runner not selected' }
if ((Get-ManagementCapsuleSource 'source_preparation.py') -cne 'E:/retained-installed-capsule/source_preparation.py') { throw 'Installed helper was replaced' }
if ((Get-ManagementCapsuleSource 'zircon-management.groovy') -cne 'E:/retained-installed-capsule/zircon-management.groovy') { throw 'Installed pipeline was replaced' }
'''.replace("__INSTALLER__", str(INSTALLER).replace("'", "''"))
        result = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
                                capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr[-2000:])


if __name__ == "__main__":
    unittest.main()
