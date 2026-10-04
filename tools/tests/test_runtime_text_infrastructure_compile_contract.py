"""Canonical loader for folder-backed Runtime Text infrastructure contracts."""

import unittest

from tools.tests.runtime_text_infrastructure_compile_contract import (
    cache_table_geometry,
    context_geometry,
    rich_parser_admission,
    semantic_structure,
)


def load_tests(loader: unittest.TestLoader, _tests: unittest.TestSuite, _pattern: str):
    suite = unittest.TestSuite()
    for module in (
        cache_table_geometry,
        rich_parser_admission,
        context_geometry,
        semantic_structure,
    ):
        suite.addTests(loader.loadTestsFromModule(module))
    return suite
