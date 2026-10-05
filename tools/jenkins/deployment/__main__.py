from __future__ import annotations

import argparse
import json
from pathlib import Path

from .manager import DeploymentManager
from .paths import resolve_paths
from .spec import load_spec
from ..contracts import JenkinsError, response


def main() -> int:
    parser = argparse.ArgumentParser(description="Formal Jenkins deployment executor")
    parser.add_argument("spec_positional", type=Path, nargs="?")
    parser.add_argument("--spec", dest="spec_option", type=Path)
    parser.add_argument("action", choices=("health", "start", "stop", "reconcile", "recover-activation"))
    parser.add_argument("--java", type=Path)
    parser.add_argument("--war", type=Path)
    parser.add_argument("--transition-digest")
    args = parser.parse_args()
    spec_path = args.spec_option or args.spec_positional or (Path.cwd() / ".jenkins" / "deployment-spec.json")
    try:
        spec = load_spec(spec_path); paths = resolve_paths(spec)
        java = args.java or Path(spec.controller["java"]["executable"])
        war = args.war or Path(spec.controller["warPath"])
        manager = DeploymentManager(spec, paths, java, war)
        if args.action == "recover-activation":
            result = manager.recover_activation(transition_digest=args.transition_digest)
        else:
            result = getattr(manager, args.action)()
    except JenkinsError as error:
        print(json.dumps(response('waiting' if error.retryable else 'blocked', reason_code=error.code,
                                 retryable=error.retryable, message=str(error), details=error.details), sort_keys=True))
        return 2
    print(json.dumps(result, sort_keys=True))
    return 2 if result.get('state') == 'failed' or result.get('status') in {'waiting', 'blocked', 'unknown', 'failed'} else 0


if __name__ == "__main__": raise SystemExit(main())
