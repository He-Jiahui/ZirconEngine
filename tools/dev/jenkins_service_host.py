"""Enter the installed lifecycle host with the selected immutable backend."""
from pathlib import Path
import json
import os
import runpy
import sys

sys.dont_write_bytecode = True
repository = Path(__file__).absolute().parents[2]
selected = json.loads((repository / '.jenkins/state/deployment/driver.json').read_text(encoding='utf-8'))
root = repository / '.jenkins/runtime/drivers' / selected['driverDigest']
if Path(selected['root']).absolute() != root or root.is_symlink():
    raise SystemExit('Invalid selected Jenkins runtime')
sys.path.insert(0, str(root))
from tools.jenkins.deployment.driver import verify_driver

verify_driver(root, expected_digest=selected['driverDigest'])
os.environ['PYTHONDONTWRITEBYTECODE'] = '1'
runpy.run_path(str(repository / 'tools/jenkins/deployment/independent_host.py'), run_name='__main__')
