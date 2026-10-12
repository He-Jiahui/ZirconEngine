"""Capture one actual held Win32 client frame and compare it with its GPU readback.

No process launch, focus change, window move, system-DPI change or render substitute.
This receipt proves only the exact pixel association, not whole UI acceptance.
"""
from __future__ import annotations
import argparse, ctypes, hashlib, json, os, struct, subprocess, time, uuid, zlib
from ctypes import wintypes as w
from pathlib import Path
from zircon_editor_normal_frame_evidence import validate_hold, validate_readback, validate_profile, raw_rgba, validate_live_request, validate_case_publication, validate_case_inputs, load_case_contract_configuration


def sha(data:bytes)->str:return hashlib.sha256(data).hexdigest()
def canonical_sha(value:object)->str:
    def normalize(value):
        if isinstance(value,dict):return {key:normalize(item) for key,item in value.items()}
        if isinstance(value,list):return [normalize(item) for item in value]
        return int(value) if isinstance(value,float) and value.is_integer() else value
    return sha(json.dumps(normalize(value),sort_keys=True,ensure_ascii=False,separators=(',',':'),allow_nan=False).encode())

def owned(path:Path)->Path:
    resolved=path.resolve(strict=True)
    for drive in ('D','E','F'):
        base=Path(f'{drive}:/cargo-targets/zircon-local')
        if base.exists():
            # Both operands are physically resolved, including Windows verbatim drive prefixes.
            try:
                if os.path.normcase(os.path.commonpath([resolved,base.resolve(strict=True)]))==os.path.normcase(str(base.resolve(strict=True))):return resolved
            except ValueError:pass
    raise ValueError(f'Output is not in an approved physical owned root: {resolved}')

def load(path:Path)->dict:
    if path.stat().st_size>8*1024*1024:raise ValueError(f'Unexpected metadata size: {path}')
    value=json.loads(path.read_text(encoding='utf-8'))
    if not isinstance(value,dict):raise ValueError('Expected metadata object')
    return value

def write_new(path:Path,data:bytes)->None:
    with path.open('xb') as stream:stream.write(data)

def publish_request(directory:Path,request:dict)->None:
    path=directory/'capture-request.json'
    if path.exists():
        old=load(path)
        if old.get('pid')!=request['pid']:raise ValueError('Capture directory contains another process request')
        if old.get('nonce')!=request['nonce'] and old.get('release') is not True:
            started,limit=old.get('requestedAtUnixMs'),old.get('deadlineMs')
            if not isinstance(started,int) or not isinstance(limit,int) or started+limit>time.time_ns()//1_000_000:raise ValueError('Capture directory contains another active nonce')
    temporary=directory/f"capture-request-{request['nonce']}.tmp"
    write_new(temporary,json.dumps(request,sort_keys=True).encode())
    os.replace(temporary,path)

def case_hash(path:Path,binding:dict)->str:
    case=load(path)
    local=canonical_sha(case)
    program="import fs from 'node:fs'; const m=await import(process.argv[1]); const c=JSON.parse(fs.readFileSync(process.argv[2],'utf8')); process.stdout.write(await m.caseSha256(c));"
    result=subprocess.run([binding['nodePath'],'--input-type=module','-e',program,Path(binding['modulePath']).as_uri(),str(path)],capture_output=True,text=True,timeout=15,cwd=path.parent)
    if result.returncode or result.stdout.strip()!=local:raise ValueError(f'Actual Python/official TS case hash mismatch: {result.stderr[:200]}')
    return local

class RECT(ctypes.Structure):_fields_=[('left',w.LONG),('top',w.LONG),('right',w.LONG),('bottom',w.LONG)]
class POINT(ctypes.Structure):_fields_=[('x',w.LONG),('y',w.LONG)]
class MONITORINFOEX(ctypes.Structure):_fields_=[('size',w.DWORD),('monitor',RECT),('work',RECT),('flags',w.DWORD),('device',w.WCHAR*32)]
class BITMAPINFOHEADER(ctypes.Structure):
    _fields_=[('size',w.DWORD),('width',w.LONG),('height',w.LONG),('planes',w.WORD),('bitcount',w.WORD),('compression',w.DWORD),('image_size',w.DWORD),('xppm',w.LONG),('yppm',w.LONG),('colors_used',w.DWORD),('colors_important',w.DWORD)]
class BITMAPINFO(ctypes.Structure):_fields_=[('header',BITMAPINFOHEADER),('colors',w.DWORD*3)]

class DesktopCapture:
    def __init__(self):
        if os.name!='nt':raise RuntimeError('Actual Win32 user desktop is required')
        self.user=ctypes.WinDLL('user32',use_last_error=True);self.gdi=ctypes.WinDLL('gdi32',use_last_error=True)
        self.kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        for name,args,result in [('OpenProcess',[w.DWORD,w.BOOL,w.DWORD],w.HANDLE),('QueryFullProcessImageNameW',[w.HANDLE,w.DWORD,w.LPWSTR,ctypes.POINTER(w.DWORD)],w.BOOL),('CloseHandle',[w.HANDLE],w.BOOL)]:
            function=getattr(self.kernel,name);function.argtypes=args;function.restype=result
        self.gdi.GdiFlush.argtypes=[];self.gdi.GdiFlush.restype=w.BOOL
        self.user.MonitorFromWindow.argtypes=[w.HWND,w.DWORD];self.user.MonitorFromWindow.restype=w.HANDLE
        self.user.GetMonitorInfoW.argtypes=[w.HANDLE,ctypes.POINTER(MONITORINFOEX)];self.user.GetMonitorInfoW.restype=w.BOOL
        declarations=[(self.user,'PrintWindow',[w.HWND,w.HDC,w.UINT],w.BOOL),(self.user,'IsWindow',[w.HWND],w.BOOL),(self.user,'IsWindowVisible',[w.HWND],w.BOOL),(self.user,'GetWindowThreadProcessId',[w.HWND,ctypes.POINTER(w.DWORD)],w.DWORD),(self.user,'GetClientRect',[w.HWND,ctypes.POINTER(RECT)],w.BOOL),(self.user,'ClientToScreen',[w.HWND,ctypes.POINTER(POINT)],w.BOOL),(self.user,'GetDpiForWindow',[w.HWND],w.UINT),(self.user,'GetDC',[w.HWND],w.HDC),(self.user,'ReleaseDC',[w.HWND,w.HDC],ctypes.c_int),(self.user,'RedrawWindow',[w.HWND,ctypes.c_void_p,ctypes.c_void_p,w.UINT],w.BOOL),(self.user,'SetThreadDpiAwarenessContext',[ctypes.c_void_p],ctypes.c_void_p),(self.gdi,'CreateCompatibleDC',[w.HDC],w.HDC),(self.gdi,'CreateDIBSection',[w.HDC,ctypes.POINTER(BITMAPINFO),w.UINT,ctypes.POINTER(ctypes.c_void_p),ctypes.c_void_p,w.DWORD],ctypes.c_void_p),(self.gdi,'SelectObject',[w.HDC,ctypes.c_void_p],ctypes.c_void_p),(self.gdi,'BitBlt',[w.HDC,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_int,w.HDC,ctypes.c_int,ctypes.c_int,w.DWORD],w.BOOL),(self.gdi,'DeleteObject',[ctypes.c_void_p],w.BOOL),(self.gdi,'DeleteDC',[w.HDC],w.BOOL)]
        for owner,name,args,result in declarations:
            function=getattr(owner,name);function.argtypes=args;function.restype=result
        self.old_dpi=self.user.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))
        if not self.old_dpi:raise ctypes.WinError(ctypes.get_last_error())
    def geometry(self,hwnd:int,pid:int,dpi:int)->tuple[int,int,int,int]:
        owner=w.DWORD();self.user.GetWindowThreadProcessId(hwnd,ctypes.byref(owner))
        if not self.user.IsWindow(hwnd) or not self.user.IsWindowVisible(hwnd) or owner.value!=pid:raise ValueError('Actual visible HWND/PID identity changed')
        if self.user.GetDpiForWindow(hwnd)!=dpi:raise ValueError('Actual window DPI differs from the selected real DPI case')
        rectangle=RECT();point=POINT()
        if not self.user.GetClientRect(hwnd,ctypes.byref(rectangle)) or not self.user.ClientToScreen(hwnd,ctypes.byref(point)):raise ctypes.WinError(ctypes.get_last_error())
        width,height=rectangle.right-rectangle.left,rectangle.bottom-rectangle.top
        if width<=0 or height<=0 or width*height*4>24*1024*1024:raise ValueError('Actual client extent exceeds bounded capture')
        return point.x,point.y,width,height
    def rgba(self,hwnd:int,pid:int,dpi:int,expected:tuple[int,int],print_window:bool=False)->bytes:
        x,y,width,height=self.geometry(hwnd,pid,dpi)
        if (width,height)!=expected:raise ValueError('Actual client extent changed')
        source=self.user.GetDC(None);destination=self.gdi.CreateCompatibleDC(source);bits=ctypes.c_void_p()
        info=BITMAPINFO();info.header=BITMAPINFOHEADER(ctypes.sizeof(BITMAPINFOHEADER),width,-height,1,32,0,width*height*4,0,0,0,0)
        bitmap=self.gdi.CreateDIBSection(source,ctypes.byref(info),0,ctypes.byref(bits),None,0);old=None
        try:
            if not source or not destination or not bitmap or not bits.value:raise ctypes.WinError(ctypes.get_last_error())
            old=self.gdi.SelectObject(destination,bitmap)
            if not old or old==ctypes.c_void_p(-1).value:raise ctypes.WinError(ctypes.get_last_error())
            if print_window:
                # Ask only the already validated visible owner HWND for its actual
                # client image. The unchanged hold and exact GPU bytes still gate it.
                if not self.user.PrintWindow(hwnd,destination,0x0001|0x0002):raise ctypes.WinError(ctypes.get_last_error())
            elif not self.gdi.BitBlt(destination,0,0,width,height,source,x,y,0x00CC0020):raise ctypes.WinError(ctypes.get_last_error())
            if not self.gdi.GdiFlush():raise ctypes.WinError(ctypes.get_last_error())
            raw=ctypes.string_at(bits,width*height*4)
            if self.geometry(hwnd,pid,dpi)!=(x,y,width,height):raise ValueError('Actual client global bounds changed during desktop capture')
            # Desktop DIB alpha is undefined; PNG is RGB with an explicitly opaque desktop representation.
            rgba=bytearray(raw)
            rgba[0::4]=raw[2::4]
            rgba[2::4]=raw[0::4]
            rgba[3::4]=bytes([255])*(width*height)
            return bytes(rgba)
        finally:
            if old and old!=ctypes.c_void_p(-1).value:self.gdi.SelectObject(destination,old)
            if bitmap:self.gdi.DeleteObject(bitmap)
            if destination:self.gdi.DeleteDC(destination)
            if source:self.user.ReleaseDC(None,source)
    def process_image(self,pid:int)->dict:
        process=self.kernel.OpenProcess(0x1000,False,pid)
        if not process:raise ctypes.WinError(ctypes.get_last_error())
        try:
            buffer=ctypes.create_unicode_buffer(32768);length=w.DWORD(len(buffer))
            if not self.kernel.QueryFullProcessImageNameW(process,0,buffer,ctypes.byref(length)):raise ctypes.WinError(ctypes.get_last_error())
            path=Path(buffer.value).resolve(strict=True)
            return {'path':str(path),'sha256':sha(path.read_bytes())}
        finally:self.kernel.CloseHandle(process)
    def monitor(self,hwnd:int)->dict:
        handle=self.user.MonitorFromWindow(hwnd,2);info=MONITORINFOEX();info.size=ctypes.sizeof(info)
        if not handle or not self.user.GetMonitorInfoW(handle,ctypes.byref(info)):raise ctypes.WinError(ctypes.get_last_error())
        return {'deviceName':info.device,'bounds':{'x':info.monitor.left,'y':info.monitor.top,'width':info.monitor.right-info.monitor.left,'height':info.monitor.bottom-info.monitor.top}}
    def close(self):self.user.SetThreadDpiAwarenessContext(self.old_dpi)

def png_rgba(width:int,height:int,rgba:bytes)->bytes:
    def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
    rows=b''.join(b'\x00'+rgba[offset:offset+width*4] for offset in range(0,len(rgba),width*4))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b'')

def held(directory:Path,nonce:str,pid:int,case_sha:str,size:tuple[int,int])->dict:
    ack=load(directory/'capture-hold-ack.json')
    validate_hold(ack,nonce,pid,case_sha,size,time.time_ns()//1_000_000)
    return ack

def matching_profile(directory:Path,hold:dict,size:tuple[int,int],scale:float,case:dict)->tuple[Path,dict] | None:
    for path in sorted(directory.glob('present-*/ui_profile_geometry.json'),reverse=True):
        try:profile=load(path)
        except (OSError,ValueError):continue
        evidence=profile.get('normal_window_evidence') or {}
        if evidence.get('captureHold')!=hold:continue
        bound=((case.get('data') or {}).get('normal_window_evidence') or {}).get('focusPublication')
        validate_profile(profile,hold,size,scale,bound)
        validate_case_publication(profile,case)
        return path,profile
    return None

def run(args):
    profile=owned(args.profile_directory);output=owned(args.output_directory);case=args.case_json.resolve(strict=True)
    if any(output.iterdir()):raise ValueError('Use an existing empty owned evidence directory; preserve foreign receipts')
    repo=Path(__file__).resolve().parents[3]
    binding=load_case_contract_configuration(args.contract_configuration,args.contract_configuration_sha256,repo)
    case_bytes=case.read_bytes();case_sha=case_hash(case,binding);case_value=load(case);validate_case_inputs(case_value,repo,args.project_root);nonce=uuid.uuid4().hex;api=DesktopCapture();request=None
    try:
        _,_,width,height=api.geometry(args.hwnd,args.pid,args.expected_dpi);size=(width,height)
        scale=args.expected_dpi/96
        if case_value.get('dpi')!=scale or case_value.get('viewport')!={'width':width/scale,'height':height/scale}:raise ValueError('Actual client extent/DPI differs from the immutable case')
        process_image=api.process_image(args.pid)
        monitor=api.monitor(args.hwnd)
        request={'nonce':nonce,'pid':args.pid,'hwnd':args.hwnd,'caseSha256':case_sha,'physicalSize':list(size),'requestedAtUnixMs':time.time_ns()//1_000_000,'deadlineMs':args.timeout_ms,'gpuReadback':True,'release':False}
        publish_request(profile,request)
        # One genuine own-window capture demand starts the actual normal render. Completion uses its event-loop owner.
        if not api.user.RedrawWindow(args.hwnd,None,None,0x0001|0x0100):raise ctypes.WinError(ctypes.get_last_error())
        deadline=time.monotonic()+args.timeout_ms/1000
        readback_path=profile/f'normal-gpu-{nonce}.json'
        while not readback_path.exists():
            if time.monotonic()>=deadline:raise TimeoutError('Actual normal GPU readback did not publish before bounded deadline')
            api.geometry(args.hwnd,args.pid,args.expected_dpi)
            time.sleep(.02)
        readback_bytes=readback_path.read_bytes();readback=load(readback_path);before=held(profile,nonce,args.pid,case_sha,size)
        validate_readback(readback,before,args.hwnd,size)
        corresponding=matching_profile(profile,before,size,scale,case_value)
        while corresponding is None:
            if time.monotonic()>=deadline:raise TimeoutError('Actual matching normal GPU/Winit/source profile was not published before the bounded deadline')
            held(profile,nonce,args.pid,case_sha,size)
            time.sleep(.02)
            corresponding=matching_profile(profile,before,size,scale,case_value)
        profile_path,source_profile=corresponding
        profile_bytes=profile_path.read_bytes()
        if source_profile.get('window_client_size')!={'width':width,'height':height}:raise ValueError('Actual corresponding GPU/source profile extent changed')
        if readback.get('rawFile')!=f'normal-gpu-{nonce}.color8':raise ValueError('Unexpected raw pixel path')
        raw_path=profile/readback['rawFile']
        if raw_path.stat().st_size!=width*height*4:raise ValueError('Actual GPU pixel extent changed')
        raw=raw_path.read_bytes()
        gpu=raw_rgba(readback,raw,size)
        method='win32-desktop-bitblt-gdiflush-own-client'
        desktop_attempt=None
        try:
            desktop=api.rgba(args.hwnd,args.pid,args.expected_dpi,size)
        except OSError as desktop_error:
            # Restricted Windows desktops can return ACCESS_DENIED or an invalid
            # screen DC to BitBlt. Preserve that actual failure, then ask only the
            # already validated owner HWND; unchanged hold and exact GPU bytes
            # remain mandatory for the resulting client capture.
            if getattr(desktop_error,'winerror',None) not in (5,6):raise
            attempt_path=output/f'{nonce}.desktop-attempt.json'
            attempt_bytes=json.dumps({'method':method,'pid':args.pid,'hwnd':args.hwnd,
                'winError':desktop_error.winerror,'error':str(desktop_error)},
                ensure_ascii=False,indent=2).encode()
            write_new(attempt_path,attempt_bytes)
            desktop_attempt={'method':method,'record':str(attempt_path),
                'recordSha256':sha(attempt_bytes),'winError':desktop_error.winerror}
            if held(profile,nonce,args.pid,case_sha,size)!=before:raise ValueError('Actual hold changed before own-window fallback')
            desktop=api.rgba(args.hwnd,args.pid,args.expected_dpi,size,print_window=True)
            method='win32-printwindow-client-renderfullcontent-gdiflush'
        if method=='win32-desktop-bitblt-gdiflush-own-client' and bytes(gpu)!=desktop:
            attempt_path=output/f'{nonce}.desktop-attempt.png'
            attempt_png=png_rgba(width,height,desktop);write_new(attempt_path,attempt_png)
            desktop_attempt={'method':method,'png':str(attempt_path),'pngSha256':sha(attempt_png),
                             'differingByteChannels':sum(a!=b for a,b in zip(gpu,desktop))}
            if held(profile,nonce,args.pid,case_sha,size)!=before:raise ValueError('Actual hold changed before own-window fallback')
            desktop=api.rgba(args.hwnd,args.pid,args.expected_dpi,size,print_window=True)
            method='win32-printwindow-client-renderfullcontent-gdiflush'
        captured_at=time.time_ns()//1_000_000;after=held(profile,nonce,args.pid,case_sha,size)
        api.geometry(args.hwnd,args.pid,args.expected_dpi)
        if before!=after:raise ValueError('Actual normal hold changed during Win32 client capture')
        if bytes(gpu)!=desktop:raise ValueError(f'Actual GPU/desktop pixel mismatch: {sum(a!=b for a,b in zip(gpu,desktop))} byte channels differ')
        png=png_rgba(width,height,desktop);png_path=output/f'{nonce}.client.png';write_new(png_path,png)
        if held(profile,nonce,args.pid,case_sha,size)!=before or case.read_bytes()!=case_bytes or readback_path.read_bytes()!=readback_bytes or profile_path.read_bytes()!=profile_bytes or api.process_image(args.pid)!=process_image or api.monitor(args.hwnd)!=monitor:raise ValueError('Actual held frame, immutable case, profile, readback, monitor or process changed before publication')
        validate_case_inputs(case_value,Path(__file__).resolve().parents[3],args.project_root)
        if load_case_contract_configuration(args.contract_configuration,args.contract_configuration_sha256,repo)!=binding:raise ValueError('Current Main case contract changed during capture')
        current=load(profile/'capture-request.json')
        validate_live_request(current,request,time.time_ns()//1_000_000)
        proof={'schema':'dev.zircon.editor.normal-frame-pixel-association','version':1,'pixelFrameVerified':True,'wholeUiAccepted':False,'captureMethod':method,'processImage':process_image,'nonce':nonce,'caseSha256':case_sha,'caseJsonSha256':sha(case_bytes),'officialCaseHashModuleSha256':binding['moduleSha256'],'officialCaseHashInputAuditSha256':binding['inputAuditSha256'],'caseContract':binding,'pid':args.pid,'hwnd':args.hwnd,'actualWindowDpi':args.expected_dpi,'physicalSize':list(size),'presentationCursor':before['presentationCursor'],'presentedFrameCount':before['presentedFrameCount'],'normalReadbackReceiptPath':str(readback_path),'normalReadbackReceipt':readback,'normalReadbackReceiptSha256':sha(readback_bytes),'rawGpuSha256':sha(raw),'decodedRgba8Sha256':sha(desktop),'clientPng':str(png_path),'clientPngSha256':sha(png),'capturedAtUnixMs':captured_at,'holdBefore':before,'holdAfter':after}
        proof['normalSourceProfile']=str(profile_path);proof['normalSourceProfileSha256']=sha(profile_bytes);proof['actualWinitScaleFactor']=source_profile['winit_scale_factor'];proof['focusPublication']=(source_profile.get('normal_window_evidence') or {}).get('focusPublication')
        proof['actualMonitor']=monitor
        if desktop_attempt is not None:proof['desktopAttempt']=desktop_attempt
        write_new(output/f'{nonce}.pixel-association.json',json.dumps(proof,ensure_ascii=False,indent=2).encode());print(json.dumps({'receipt':str(output/f'{nonce}.pixel-association.json'),'pixelFrameVerified':True,'wholeUiAccepted':False}))
    except Exception as error:
        pending={'schema':'dev.zircon.editor.normal-frame-pixel-association','version':1,'status':'pending','pixelFrameVerified':False,'wholeUiAccepted':False,'nonce':nonce,'caseSha256':case_sha,'pid':args.pid,'hwnd':args.hwnd,'reason':str(error)[:2000]}
        try:write_new(output/f'{nonce}.pending.json',json.dumps(pending,ensure_ascii=False,indent=2).encode())
        except OSError:pass
        raise
    finally:
        try:
            if request:
                try:current=load(profile/'capture-request.json')
                except (OSError,ValueError):current={}
                if current.get('nonce')==nonce and current.get('pid')==args.pid:request['release']=True;request['requestedAtUnixMs']=time.time_ns()//1_000_000;publish_request(profile,request)
        finally:api.close()

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--profile-directory',type=Path,required=True);parser.add_argument('--output-directory',type=Path,required=True);parser.add_argument('--case-json',type=Path,required=True);parser.add_argument('--contract-configuration',type=Path,required=True);parser.add_argument('--contract-configuration-sha256',required=True);parser.add_argument('--project-root',type=Path);parser.add_argument('--pid',type=int,required=True);parser.add_argument('--hwnd',type=int,required=True);parser.add_argument('--expected-dpi',type=int,choices=(96,144),required=True);parser.add_argument('--timeout-ms',type=int,default=10000);args=parser.parse_args()
    if not 50<=args.timeout_ms<=10000:parser.error('--timeout-ms must be 50..10000')
    run(args)
