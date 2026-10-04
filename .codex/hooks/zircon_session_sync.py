from __future__ import annotations

import sys


def main() -> int:
    # Existing chats may retain an old hook registration until their next reload.
    # This terminal hook never imports, contacts or restores the retired runtime.
    if "Stop" in sys.argv[1:]:
        sys.stdout.write('{"continue":true}\n')
        sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
