"""Create the empty cooked scene used by web-canvas-format.html in a new directory.

Run `python canvas_scene.py path/to/site/canvas-scene`. Serve its parent directory
alongside web-canvas-format.html and the wasm-bindgen `zircon_app` module.
"""
import argparse
import json
from pathlib import Path

SCENE = b'{"entities":[],"resources":[]}\n'
SCENE_BLAKE3 = 'c7ffff4cdeac48cec019c235656b22cc46d4d525672feb77157cbc5975b438a0'


def write_scene(root):
    root.mkdir(parents=True, exist_ok=False)
    scenes = root / 'scenes'
    scenes.mkdir()
    (scenes / (SCENE_BLAKE3 + '.json')).write_bytes(SCENE)
    catalog = {
        'schema_version': 1,
        'target_platform': 'wasm32-webgpu',
        'assets': [],
        'scene': {'payload_bytes': len(SCENE), 'content_hash': list(bytes.fromhex(SCENE_BLAKE3))},
    }
    (root / 'catalog.json').write_text(json.dumps(catalog) + '\n', encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    write_scene(parser.parse_args().directory)
