"""Native create, select/edit/add, Play/Stop, discard and clean reopen acceptance."""
from pathlib import Path
import argparse
import json
import time
import traceback
from PIL import Image, ImageStat
from linux_editor_interaction_driver import NativeSession

def select(session,name):
    state=session.state()
    entry=next(row for row in state['scene'] if row['name']==name)
    row=next(c for c in state['controls'] if c['id'].startswith('WorkbenchScene')
             and c['text']==name and c['frame'][3]>0)
    x,y,w,h=row['frame']
    session.click(x+w*.35,y+h/2)
    session.wait(lambda:(s:=session.state()).get('inspector')
        and s['inspector']['id']==entry['id']
        and [r['id'] for r in s['scene'] if r['selected']]==[entry['id']],15)
    return entry['id']

def visible_game(session):
    deadline=time.monotonic()+45
    while time.monotonic()<deadline:
        state=session.state()
        image=state.get('game_image') or {}
        if image.get('valid') and image.get('generation',0)>0:
            session.shot('playing')
            control=session.control('WorkbenchViewportSurface')
            x,y,w,h=control['frame'] if control else state['viewport_frame']
            comparisons=[]
            with Image.open(session.root/'playing.png') as picture:
                rgb=picture.convert('RGB')
                deviation=max(ImageStat.Stat(rgb.crop((round(x+8),round(y+40),round(x+w-8),round(y+h-8)))).stddev)
                for sample in image.get('samples') or []:
                    actual=rgb.getpixel((int(x+w*sample['x']),int(y+h*sample['y'])))
                    expected=sample['rgba'][:3]
                    comparisons.append({'actual':actual,'expected':expected,
                        'matches':max(abs(a-b) for a,b in zip(actual,expected))<=12})
            if deviation>5 and len(comparisons)==5 and all(p['matches'] for p in comparisons) and len({tuple(p['expected']) for p in comparisons})>1:
                session.record.update(game_pixel_stddev=deviation,game_pixel_comparisons=comparisons)
                return
        time.sleep(.2)
    raise AssertionError('native Game pixels did not match the runtime image')

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--stage',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    args.output.mkdir(parents=True,exist_ok=False)
    state_root=args.output/'state'
    session=NativeSession(args.stage,args.output/'create',state_root,True)
    try:
        initial=session.state()
        camera=select(session,'Camera')
        x,y,w,h=session.state()['viewport_frame']
        session.click(x+w/2,y+h/2)
        session.wait(lambda:(s:=session.state()).get('inspector') and s['inspector']['id']!=camera
            and s['inspector']['name']=='Cube',15)
        session.record['viewport_pick_verified']=True
        session.shot('viewport-pick')
        select(session,'Camera')
        before=session.state()['inspector']['translation']
        value=float(before[0])+1.5
        session.click_control('WorkbenchTransformPositionX')
        session.key('ctrl+a')
        session.type(str(value))
        session.key('Return')
        session.wait(lambda:(s:=session.state()).get('inspector') and s['inspector']['id']==camera
            and abs(float(s['inspector']['translation'][0])-value)<.001,15)
        session.record.update(camera_transform_before=before,camera_transform_after=session.state()['inspector']['translation'])
        session.shot('camera-details')
        before_add=session.state()['scene']
        existing_ids={row['id'] for row in before_add}
        count=len(before_add)
        session.click_control('WorkbenchToolbarAddActor')
        popup=session.wait(lambda:c if (c:=session.control('UnrealAddMenuPopup')) and c['popup_open'] else None,10)
        session.shot('add-menu')
        row=next(i for i,item in enumerate(popup['menu_items']) if 'action=scene.node.create_cube' in item)
        x,y,w,h=popup['frame']
        session.click(x+w/2,y+6+row*30+14)
        session.wait(lambda:len(session.state().get('scene',[]))==count+1,15)
        added_state=session.state()
        added=added_state['inspector']
        new_rows=[row for row in added_state['scene'] if row['id'] not in existing_ids]
        assert len(new_rows)==1 and new_rows[0]['name']=='Cube 1', new_rows
        assert added['id']==new_rows[0]['id'] and added['name']==new_rows[0]['name']
        assert [r['id'] for r in added_state['scene'] if r['selected']]==[added['id']]
        session.record['added_cube']=added
        session.shot('added-cube')
        session.key('Escape')
        select(session,'Camera')
        session.record['outliner_selection_verified']=True
        edited=session.state()
        session.shot('edit-before-play')
        requested=time.monotonic()
        session.click_control('WorkbenchRunPlay')
        session.wait_mode('Playing')
        session.record['play_requested_to_playing_ms']=(time.monotonic()-requested)*1000
        visible_game(session)
        session.record['play_requested_to_visible_game_ms']=(time.monotonic()-requested)*1000
        requested=time.monotonic()
        session.click_control('WorkbenchRunStop')
        session.wait_mode('Edit')
        session.wait(lambda:session.control('WorkbenchRunPlay') and not session.control('WorkbenchRunPlay')['disabled'],15)
        session.record['stop_requested_to_edit_ms']=(time.monotonic()-requested)*1000
        session.wait(lambda:session.state()['scene']==edited['scene'] and session.state()['inspector']==edited['inspector'],15)
        session.record['edit_restoration_verified']=True
        session.shot('stopped')
        session.close(discard=True)
        assert session.record.get('discarded')
        reopened=NativeSession(args.stage,args.output/'reopen',state_root,False)
        try:
            actual=reopened.state()
            assert {(r['id'],r['name']) for r in actual['scene']}=={(r['id'],r['name']) for r in initial['scene']}
            select(reopened,'Camera')
            original_camera=next(r for r in initial['scene'] if r['name']=='Camera')
            assert reopened.state()['inspector']['id']==original_camera['id']
            assert reopened.state()['inspector']['translation']==before
            reopened.record['discard_reopen_verified']=True
            reopened.shot('reopened')
            reopened.close(discard=False)
        except Exception:
            reopened.save()
            raise
        (args.output/'result.json').write_text(json.dumps({'passed':True,'create':session.record,'reopen':reopened.record},indent=2)+'\n')
        print('PASS: native interaction acceptance, clean exit and discarded reopen',flush=True)
    except Exception:
        session.record['failure']=traceback.format_exc()
        session.save()
        raise

if __name__=='__main__':
    main()
