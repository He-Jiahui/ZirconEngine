from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.cargo_storage import (
    managed_cargo_server_port,
    managed_cargo_server_temp_path,
    managed_cargo_storage_root,
)


class CargoStorageTests(unittest.TestCase):
    def test_server_temp_is_stable_outside_job_scratch(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            engine = Path(temporary) / "zircon-engine"
            target = engine / "pool" / "compatibility" / "release"

            server_temp = managed_cargo_server_temp_path(target)

            self.assertEqual(engine / "cache" / "sccache-temporary", server_temp)
            self.assertNotIn("scratch", server_temp.parts)

    @unittest.skipUnless(os.name == "nt", "managed build roots are Windows drives")
    def test_server_ports_are_distinct_per_approved_drive(self) -> None:
        self.assertEqual(42260, managed_cargo_server_port(r"D:\cargo-targets\pool\d"))
        self.assertEqual(42261, managed_cargo_server_port(r"E:\cargo-targets\pool\e"))
        self.assertEqual(42262, managed_cargo_server_port(r"F:\cargo-targets\pool\f"))
        self.assertEqual(42263, managed_cargo_server_port(r"D:\targets\pool\d"))
        self.assertEqual(42264, managed_cargo_server_port(r"E:\targets\pool\e"))
        self.assertEqual(42265, managed_cargo_server_port(r"F:\targets\pool\f"))
        self.assertEqual(42266, managed_cargo_server_port(r"D:\ZirconBuilds\pool\d"))
        self.assertEqual(42267, managed_cargo_server_port(r"E:\ZirconBuilds\pool\e"))
        self.assertEqual(42268, managed_cargo_server_port(r"F:\ZirconBuilds\pool\f"))

    @unittest.skipUnless(os.name == "nt", "managed build roots are Windows drives")
    def test_explicit_target_uses_the_approved_roots_engine_storage(self) -> None:
        target = Path(r"E:\cargo-targets\explicit-validation-target")

        self.assertEqual(
            Path(r"E:\cargo-targets\zircon-engine"),
            managed_cargo_storage_root(target),
        )
        self.assertEqual(
            Path(r"E:\cargo-targets\zircon-engine\cache\sccache-temporary"),
            managed_cargo_server_temp_path(target),
        )

    @unittest.skipUnless(os.name == "nt", "managed build roots are Windows drives")
    def test_server_port_rejects_an_unapproved_drive(self) -> None:
        with self.assertRaisesRegex(ValueError, "unsupported managed Cargo storage root"):
            managed_cargo_server_port(r"C:\temp\target")


if __name__ == "__main__":
    unittest.main()
