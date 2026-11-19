"""Check owner bindings at both zircon_build entry points, without building."""

import runpy
import sys
from functools import cache
from importlib import import_module
from unittest import mock


@cache
def _entry_namespaces():
    from tools.build import zircon_build

    # A file path has no package context, so this takes the script import path.
    # The default run name is not __main__; restore sys.path after the fallback.
    with mock.patch.object(sys, "path", sys.path.copy()):
        script = runpy.run_path(zircon_build.__file__)
    return (("package", vars(zircon_build)), ("script", script))


def assert_entry_bindings(test, owner_name, *symbols):
    owner = import_module(f"tools.build.{owner_name}")
    for mode, namespace in _entry_namespaces():
        for symbol in symbols:
            with test.subTest(mode=mode, symbol=symbol):
                test.assertIs(
                    namespace.get(symbol),
                    getattr(owner, symbol),
                    f"{mode} zircon_build.{symbol} must come from {owner.__name__}",
                )
