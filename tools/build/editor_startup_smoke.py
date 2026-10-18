"""Bounded staged-editor startup proof, separate from packaging/ABI acceptance."""
from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
import re
import subprocess

from tools.analysis.visual.zircon_pbr_visual_oracle import decode_rgba_png


def inspect_first_frame(path: Path, expected_size: tuple[int, int] | None = None) -> dict:
    image = decode_rgba_png(path)
    if expected_size is not None and (image.width, image.height) != expected_size:
        raise ValueError(f'First-frame size differs from window: {image.width}x{image.height}')
    if image.width < 640 or image.height < 360:
        raise ValueError('First frame is smaller than a usable editor window (640x360)')
    count = image.width * image.height
    total = total_squared = 0.0
    for offset in range(0, len(image.rgba), 4):
        red, green, blue, alpha = image.rgba[offset:offset + 4]
        luma = (0.2126 * red + 0.7152 * green + 0.0722 * blue) * alpha / 255
        total += luma
        total_squared += luma * luma
    deviation = math.sqrt(max(0.0, total_squared / count - (total / count) ** 2))
    if deviation <= 8:
        raise ValueError(f'Blank first frame: luma stddev={deviation:.3f}, required > 8')
    return {'path': str(path), 'width': image.width, 'height': image.height,
            'luma_stddev': deviation, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}


def run_startup_smoke(staging: Path, evidence: Path, environment: dict[str, str], *,
                      timeout_seconds: float = 120, creationflags: int = 0) -> dict:
    """Caller validates physical output paths; keep failed logs/receipt and never publish success."""
    from tools.build.build_editor import output_path
    staging, evidence = output_path(staging), output_path(evidence)
    executable = staging / 'zircon_editor.exe'
    capture = output_path(evidence / 'first-frame.png')
    receipt_path = output_path(evidence / 'startup-smoke-receipt.json')
    if capture.exists() or receipt_path.exists():
        raise ValueError('Startup smoke evidence must be fresh')
    profile = output_path(evidence / 'profile')
    profile.mkdir()
    projects = output_path(evidence / 'projects')
    projects.mkdir()
    child_environment = dict(environment)
    # Keep user state and generated UI caches out of the staged product and user profile.
    child_environment.update({
        'ZIRCON_ASSET_ROOT': str(staging / 'assets'),
        'ZIRCON_RUNTIME_LIBRARY': str(staging / 'zircon_runtime.dll'),
        'ZIRCON_CONFIG_PATH': str(profile / 'editor-config.json'),
        'ZIRCON_LOG_ROOT': str(evidence / 'logs'),
        'ZIRCON_UI_COMPILED_ARTIFACT_CACHE': str(evidence / 'ui-cache'),
        'APPDATA': str(profile), 'LOCALAPPDATA': str(profile),
        'ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME': '1',
        'ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG': str(capture),
    })
    receipt = {'status': 'failed', 'first_frame_verified': False, 'commands': [],
               'timeout_seconds': timeout_seconds}
    try:
        for label, arguments in [
            ('help', ['--help']),
            ('first-frame', ['--create-project', '--project-name', 'StartupSmoke',
                             '--location', str(projects), '--template', 'renderable-empty']),
        ]:
            command = [str(executable), *arguments]
            receipt['commands'].append(command)
            with output_path(evidence / f'{label}.stdout.log').open('xb') as stdout, \
                    output_path(evidence / f'{label}.stderr.log').open('xb') as stderr:
                completed = subprocess.run(command, cwd=staging, env=child_environment,
                                           stdout=stdout, stderr=stderr, check=False,
                                           timeout=timeout_seconds, creationflags=creationflags)
            if completed.returncode:
                raise ValueError(f'Editor {label} failed ({completed.returncode}); inspect {evidence}')
            if label == 'help':
                help_output = (evidence / f'{label}.stdout.log').read_bytes() + (evidence / f'{label}.stderr.log').read_bytes()
                if b'Usage: zircon_editor' not in help_output:
                    raise ValueError('Editor --help did not identify the editor CLI')
        log = (evidence / 'first-frame.stderr.log').read_text(encoding='utf-8', errors='replace')
        if 'gpu_backend=' not in log:
            raise ValueError('Editor first frame has no selected GPU backend diagnostic')
        if 'falling back to softbuffer' in log:
            raise ValueError('Editor first frame used the software presenter; GPU startup is unverified')
        selections = re.findall(
            r'^\[zircon_editor\] startup_boundary marker=presenter_selected '
            r'presenter_backend=(\S+) runtime_shared=(true|false)\s*$', log, re.MULTILINE)
        if len(selections) != 1:
            raise ValueError('Editor first frame requires exactly one final presenter selection')
        presenter_backend, runtime_shared = selections[0]
        if presenter_backend != 'gpu':
            raise ValueError(f'Editor first frame used {presenter_backend}; GPU startup is unverified')
        receipt.update(status='passed', first_frame_verified=True,
                       first_frame=inspect_first_frame(capture),
                       presenter_backend=presenter_backend, runtime_shared=runtime_shared == 'true')
    except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired) as error:
        receipt['error'] = str(error)
        raise ValueError(f'Editor startup smoke failed: {error}') from error
    finally:
        receipt_path.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return receipt
