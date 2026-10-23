"""Owned Linux X11 editor session. Inputs never target an unverified window."""
from pathlib import Path
import json
import os
import subprocess
import threading
import time

import psutil
from PIL import ImageGrab

class NativeSession:
    def __init__(self, stage, evidence, state_root, create):
        self.stage = Path(stage).resolve()
        self.root = Path(evidence).resolve()
        self.root.mkdir(parents=True, exist_ok=False)
        self.state_root = Path(state_root).resolve()
        for name in ('profile', 'projects', 'appdata', 'ui-cache', 'mesa-cache'):
            (self.state_root / name).mkdir(parents=True, exist_ok=True)
        (self.root / 'xdg-runtime').mkdir(mode=0o700)
        self.env = dict(os.environ, DISPLAY=':3', WGPU_BACKEND='vulkan',
            ZIRCON_ALLOW_SOFTWARE_ADAPTER='1',
            XDG_RUNTIME_DIR=str(self.root / 'xdg-runtime'),
            XDG_CONFIG_HOME=str(self.state_root / 'profile/config'),
            XDG_CACHE_HOME=str(self.state_root / 'profile/cache'),
            XDG_DATA_HOME=str(self.state_root / 'profile/data'),
            ZIRCON_CONFIG_PATH=str(self.state_root / 'profile/editor-config.json'),
            ZIRCON_LOG_ROOT=str(self.root / 'logs'),
            ZIRCON_ASSET_ROOT=str(self.stage / 'assets'),
            ZIRCON_RUNTIME_LIBRARY=str(self.stage / 'libzircon_runtime.so'),
            ZIRCON_UI_COMPILED_ARTIFACT_CACHE=str(self.state_root / 'ui-cache'),
            MESA_SHADER_CACHE_DIR=str(self.state_root / 'mesa-cache'),
            ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG=str(self.root / 'first-frame.png'),
            ZIRCON_EDITOR_INTERACTION_EVIDENCE=str(self.root / 'state.json'),
            ZIRCON_TRACE_EDITOR_INTERACTION='1', ZIRCON_TRACE_PLAY_STOP='1')
        for key in ('LD_PRELOAD','ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME','ZIRCON_PROFILE_CAPTURE'):
            self.env.pop(key, None)
        editor = [str(self.stage / 'zircon_editor')]
        self.project = self.state_root / 'projects/InteractionAcceptance'
        if create:
            editor += ['--create-project', '--project-name', self.project.name,
                       '--location', str(self.project.parent), '--template', 'renderable-empty']
        else:
            editor += ['--project', str(self.project)]
        command = ['bwrap', '--die-with-parent', '--bind', '/', '/', '--bind',
                   str(self.state_root / 'appdata'), str(Path.home() / '.zircon'),
                   '--chdir', str(self.stage), '--', *editor]
        self.record = {'command': command, 'environment': {k:v for k,v in self.env.items()
            if k.startswith(('ZIRCON_', 'XDG_', 'MESA_', 'WGPU_')) or k == 'DISPLAY'},
            'events': [], 'boundaries': [], 'scope': 'Linux local only'}
        (self.root / 'command.json').write_text(json.dumps(self.record, indent=2)+'\n')
        self.started = time.monotonic()
        self.wall_started = time.time()
        self.proc = subprocess.Popen(command, cwd=self.stage, env=self.env,
            stdout=(self.root/'stdout.log').open('wb'), stderr=subprocess.PIPE)
        self.reader = threading.Thread(target=self.drain, daemon=True)
        self.reader.start()
        self.pid = self.wait(self.find_editor, 15)
        self.process = psutil.Process(self.pid)
        self.birth = self.process.create_time()
        self.record.update(pid=self.pid, process_create_time=self.birth,
                           executable=self.process.exe())
        self.window = self.wait(self.find_window, 180)
        self.verify()
        self.xd('windowactivate', '--sync', self.window)
        self.wait(lambda: (self.root/'first-frame.png').exists() and self.state().get('project_open'), 180)
        self.record['first_workbench_seconds'] = time.monotonic()-self.started
        self.wait(lambda: self.control('WorkbenchRunPlay'), 30)
        self.shot('workbench')

    def drain(self):
        with (self.root/'stderr.log').open('wb') as output:
            for line in self.proc.stderr:
                output.write(line)
                output.flush()
                if any(key in line for key in (b'boundary', b'editor_play_transition', b'command_result', b'command_failure')):
                    self.record['boundaries'].append({'seconds': time.monotonic()-self.started,
                        'line': line.decode(errors='replace').strip()})

    def find_editor(self):
        for process in psutil.Process(self.proc.pid).children(recursive=True):
            try:
                if Path(process.exe()) == self.stage/'zircon_editor':
                    return process.pid
            except psutil.Error:
                pass
        return None

    def find_window(self):
        result = subprocess.run(['xdotool','search','--onlyvisible','--pid',str(self.pid)],
            env=self.env, capture_output=True, text=True)
        return result.stdout.splitlines()[-1] if result.returncode == 0 and result.stdout.strip() else None

    def verify(self):
        assert self.process.is_running() and self.process.create_time() == self.birth
        assert Path(self.process.exe()) == self.stage/'zircon_editor'
        window_pid = int(self.xd('getwindowpid', self.window))
        assert window_pid == self.pid, (window_pid, self.pid)

    def xd(self, *args):
        return subprocess.check_output(['xdotool', *map(str,args)], env=self.env,
            text=True, timeout=10).strip()

    def wait(self, predicate, timeout=30):
        deadline = time.monotonic()+timeout
        while time.monotonic() < deadline:
            if self.proc.poll() is not None:
                raise RuntimeError(f'owned editor exited early: {self.proc.returncode}')
            value = predicate()
            if value:
                return value
            time.sleep(.025)
        raise TimeoutError(f'{predicate!r} exceeded {timeout}s')

    def state(self):
        try:
            return json.loads((self.root/'state.json').read_text())
        except (OSError,json.JSONDecodeError):
            return {}

    def control(self, identity):
        return next((c for c in self.state().get('controls',[])
                     if c['id']==identity and c['frame'][2]>0 and c['frame'][3]>0),None)

    def click(self,x,y):
        self.verify()
        self.xd('windowactivate','--sync', self.window)
        self.xd('mousemove','--window',self.window,round(x),round(y),'click',1)

    def click_control(self,identity):
        control=self.wait(lambda:self.control(identity))
        assert not control['disabled'], identity
        x,y,w,h=control['frame']
        self.click(x+w/2,y+h/2)

    def key(self,chord):
        self.verify()
        self.xd('windowactivate','--sync',self.window)
        self.xd('key','--window',self.window,'--clearmodifiers',chord)

    def type(self,value):
        self.verify()
        self.xd('type','--window',self.window,'--clearmodifiers','--delay',25,value)

    def shot(self,label,settle=.3):
        self.verify()
        self.xd('windowactivate','--sync',self.window)
        # The read-only observer publishes committed state before GPU presentation.
        # Allow that frame to become visible; the capture still needs visual review.
        time.sleep(settle)
        self.verify()
        geometry=dict(line.split('=',1) for line in self.xd('getwindowgeometry','--shell',self.window).splitlines())
        x,y,w,h=(int(geometry[k]) for k in ('X','Y','WIDTH','HEIGHT'))
        ImageGrab.grab(bbox=(x,y,x+w,y+h),xdisplay=':3').save(self.root/(label+'.png'))
        (self.root/(label+'.json')).write_text(json.dumps(self.state(),indent=2)+'\n')

    def wait_mode(self,expected,timeout=45):
        observed=set()
        def poll():
            state=self.state()
            mode=state.get('play_mode')
            if mode in ('Starting','Stopping') and mode not in observed:
                observed.add(mode)
                self.record['events'].append({'mode':mode,'seconds':time.monotonic()-self.started})
                self.shot(mode.lower())
            return state if mode==expected else None
        result=self.wait(poll,timeout)
        self.record['events'].append({'mode':expected,'seconds':time.monotonic()-self.started})
        return result

    def close(self,discard):
        self.key('alt+F4')
        deadline=time.monotonic()+45
        while self.proc.poll() is None and time.monotonic()<deadline:
            prompt=self.state().get('close_prompt') or {}
            if prompt.get('visible'):
                assert discard, 'unexpected unsaved scene after reopen'
                self.shot('discard-prompt')
                x,y,w,h=prompt['discard_frame']
                self.click(x+w/2,y+h/2)
                self.record['discarded']=True
                break
            time.sleep(.05)
        self.proc.wait(timeout=max(1,deadline-time.monotonic()))
        self.reader.join(timeout=5)
        assert self.proc.returncode==0, self.proc.returncode
        self.record.update(exit_code=self.proc.returncode,total_seconds=time.monotonic()-self.started,
            clock_discrepancy_seconds=time.time()-self.wall_started-(time.monotonic()-self.started),
            native_process_gone=not psutil.pid_exists(self.pid))
        assert self.record['native_process_gone']
        self.save()

    def save(self):
        (self.root/'metrics.json').write_text(json.dumps(self.record,indent=2)+'\n')
