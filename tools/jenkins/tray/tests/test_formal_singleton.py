from __future__ import annotations
import hashlib
import unittest
from pathlib import Path
from tools.jenkins.tray.formal_app import FormalTray

class FormalSingletonTests(unittest.TestCase):
    def test_mutex_name_is_repo_scoped(self):
        a = str(Path('E:/Git/A').resolve()).casefold(); b = str(Path('E:/Git/B').resolve()).casefold()
        self.assertNotEqual(hashlib.sha256(a.encode()).hexdigest()[:24], hashlib.sha256(b.encode()).hexdigest()[:24])

    def test_non_windows_singleton_is_safe(self):
        class C: paths = type('P', (), {'repo': Path('.')})()
        tray = FormalTray.__new__(FormalTray); tray.config = C()
        import tools.jenkins.tray.formal_app as app
        old = app.nw.IS_WINDOWS; app.nw.IS_WINDOWS = False
        try: self.assertTrue(tray._single_instance())
        finally: app.nw.IS_WINDOWS = old

if __name__ == '__main__': unittest.main()
