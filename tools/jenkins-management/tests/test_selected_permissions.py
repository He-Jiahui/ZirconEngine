"""The selected Full Access profile permits scoped commits without widening strict."""
from pathlib import Path
import importlib.util
import pytest
SOURCE = Path(__file__).parents[1] / 'checkout_commit.py'
spec = importlib.util.spec_from_file_location('selected_permissions_checkout', SOURCE)
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)

def fixture(tmp_path, selected):
    repo = tmp_path / 'repo'
    (repo / '.codex').mkdir(parents=True)
    (repo / 'tools').mkdir()
    (repo / 'tools/task.py').write_text('owned source')
    prefix = '' if selected is None else 'default_permissions=' + repr(selected) + '\n'
    config = prefix + '[permissions.zircon-strict.filesystem]\n' + repr(str(repo / 'tools')) + '="read"\n'
    (repo / '.codex/config.toml').write_text(config)
    return repo

def test_selected_full_access_uses_real_profile_without_widening_strict(tmp_path):
    repo = fixture(tmp_path, ':danger-full-access')
    config = repo / '.codex/config.toml'
    before = config.read_bytes()
    assert c.selected_paths(repo, ['tools/task.py'], {}, {str(repo / 'tools'): 'read'}) == ['tools/task.py']
    assert config.read_bytes() == before

@pytest.mark.parametrize('selection', ['zircon-strict', ':read-only', ':workspace-write', 'missing-profile'])
def test_other_profile_does_not_inherit_a_full_access_grant(tmp_path, selection):
    repo = fixture(tmp_path, selection)
    with pytest.raises(ValueError, match='write grant'):
        c.selected_paths(repo, ['tools/task.py'], {}, {str(repo): 'write'})

def test_legacy_grants_are_preserved_when_no_profile_is_selected(tmp_path):
    repo = fixture(tmp_path, None)
    assert c.selected_paths(repo, ['tools/task.py'], {}, {str(repo): 'write'}) == ['tools/task.py']
    with pytest.raises(ValueError, match='write grant'):
        c.selected_paths(repo, ['tools/task.py'], {}, {})

@pytest.mark.parametrize('name', ['.git/index', '.jenkins/state/secret.json', '../foreign.py', 'tools/*.py', 'tools', 'tools/../task.py'])
def test_full_access_still_requires_literal_individual_non_git_paths(tmp_path, name):
    repo = fixture(tmp_path, ':danger-full-access')
    with pytest.raises(ValueError):
        c.selected_paths(repo, [name], {}, {})

def test_profile_change_during_path_check_is_rejected(tmp_path, monkeypatch):
    repo = fixture(tmp_path, ':danger-full-access')
    original = c.physical
    def observed(path):
        result = original(path)
        if Path(path).name == 'task.py':
            (repo / '.codex/config.toml').write_text('default_permissions=":read-only"\n')
        return result
    monkeypatch.setattr(c, 'physical', observed)
    with pytest.raises(ValueError, match='permissions changed'):
        c.selected_paths(repo, ['tools/task.py'], {}, {})
