import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { prepareComponentReviewHost } from '../tools/zui-layout-component-hosts';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import {
  canonicalSha256,
  validateWorkbenchPresentation,
} from '../tools/zui-layout-workbench-presentation';
import { normalizeZuiDocument, parseZuiDocument } from './bridge/zui-document';
import { deriveTreeSelectionOwnerReview } from './bridge/zui-tree-selection-owner';
import { nativePainterComponent } from './bridge/zui-native-painter-role';
import { projectZuiDocument } from './bridge/penpot-projection';

const repo = resolve(
  process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../..'),
);
const frame = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const panel =
  'zircon_editor/assets/ui/editor/unreal/components/details.zui';
// The host palette is theme_tokens; the mounted Unreal sources import Starship.
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
type JsonObject = Record<string, unknown>;

// Unit contract DTOs; genuine Scene producers and JSON are covered by Rust owner tests.
function presentation(rotationDegrees: unknown = ['0', '90', '0']): JsonObject {
  const property = (
    id: string,
    label: string,
    value: string,
    editable = false,
  ) => ({ id, label, value, kind: 'f32', editable });
  const value: JsonObject = {
    schema: 'dev.zircon.editor.workbench-presentation',
    version: 1,
    activeLocale: 'en',
    sourceFingerprint: { sourcePath: frame, sha256: 'c'.repeat(64) },
    stateFingerprint: '',
    layout: {
      active_main_page: 'main',
      main_pages: [
        {
          WorkbenchPage: {
            id: 'main',
            title: 'Workbench',
            activity_window: 'main-window',
          },
        },
      ],
      activity_windows: {
        'main-window': {
          window_id: 'main-window',
          descriptor_id: 'editor.main',
          host_mode: 'EmbeddedMainFrame',
          activity_drawers: {},
          content_workspace: {
            Tabs: { node_id: 'document-tabs', tabs: [], active_tab: null },
          },
          menu_overflow_mode: 'Auto',
          region_overrides: {},
          view_overrides: {},
        },
      },
      floating_windows: [],
    },
    window: { id: 'main-window', title: 'Zircon Editor' },
    pages: {
      activeId: 'main',
      items: [
        { id: 'main', title: 'Workbench', activityWindowId: 'main-window' },
      ],
    },
    documents: { activeId: null, items: [] },
    drawers: [],
    hierarchy: { filterQuery: '', expandedIds: [], selectedIds: [], rows: [] },
    inspector: {
      entityId: '1',
      name: 'Camera',
      parent: '',
      translation: ['21', '2', '14.5'],
      rotationDegrees,
      scale: ['1', '1', '1'],
      renderLayerMask: 0,
      components: [
        {
          id: 'native.CameraComponent',
          title: 'Camera',
          properties: [
            property(
              'native.CameraComponent.fov_y_radians',
              'FOV Y (Radians)',
              '1.7453293',
            ),
            property('native.CameraComponent.z_near', 'Near Clip', '0.1'),
            property('native.CameraComponent.z_far', 'Far Clip', '200'),
          ],
        },
        {
          id: 'plugin.First',
          title: 'First',
          properties: [
            property('plugin.First.intensity', 'Intensity', '2', true),
          ],
        },
        {
          id: 'plugin.Second',
          title: 'Second',
          properties: [property('plugin.Second.intensity', 'Intensity', '3')],
        },
      ],
    },
    status: {
      primary: '',
      secondary: null,
      viewportLabel: '',
      projectPath: '',
    },
  };
  rehash(value);
  return value;
}
function rehash(value: JsonObject): void {
  const { stateFingerprint: ignored, ...state } = value;
  void ignored;
  value['stateFingerprint'] = canonicalSha256(state);
}


describe('whole authored Workbench Tree owner', () => {
 it('projects selected static, observed loss and return with full immutable source identity', async () => {
 const dependencies = new LayoutDependencies();
 const files = await readdir(resolve(repo, 'zircon_editor/assets/ui'), {recursive:true, withFileTypes:true});
 await dependencies.load(repo, files.filter(e=>e.isFile()&&e.name.endsWith('.zui')).map(e=>relative(repo,resolve(e.parentPath,e.name)).replaceAll('\\','/')));
 const source=parseZuiDocument(await readFile(resolve(repo,frame),'utf8')).document;
 const original=normalizeZuiDocument(source);
 const data=presentation();
 data['hierarchy']={filterQuery:'',expandedIds:[],selectedIds:['1'],rows:[{id:'1',parentId:null,name:'Camera',kind:'Entity',depth:0,active:true,hasChildren:false,generation:'1',subtreeHash:'1'}]};
 rehash(data); validateWorkbenchPresentation(data);
 const result=await prepareComponentReviewHost(repo,frame,source,dependencies,theme,data);
 expect(result).not.toBeNull();
 const document=result!.projection;
 const owners=Object.values(document.nodes!).filter(n=>n.control_id==='WorkbenchSceneTree');
 expect(owners).toHaveLength(1);
 const owner=owners[0]!;
 const identity={sourcePath:owner['penpot_review_source_path'],sourceNodeId:owner['penpot_review_source_node_id'],instancePath:owner['penpot_review_instance_path'],controlId:owner.control_id};
 for(const v of Object.values(identity)) expect(typeof v==='string'&&v.length>0).toBe(true);
 const selected=Object.entries(document.nodes!).filter(([,n])=>nativePainterComponent(n)==='TreeRow'&&n.props?.['selected']===true);
 expect(selected).toHaveLength(1);
 const id=selected[0]![0]; const before=normalizeZuiDocument(document);
 const focus={authority:'normal-native-pane-interaction',observed:true,focusedNativeControlId:'HierarchyListPanel',templateFocusVisible:true,sourceOwner:identity,sourceOwnerMappingComplete:true};
 for(const [publication,color,known] of [[undefined,'#0070e0',undefined],[{...focus,focusedNativeControlId:null,sourceOwner:null},'#40576f',false],[focus,'#0070e0',true]] as const){
 const review=structuredClone(document); deriveTreeSelectionOwnerReview(review,publication);
 expect(review.nodes![id].props!['tree_owner_focused']).toBe(known);
 expect(String((review.nodes![id].style!['self'] as Record<string, unknown>)['background_color']).toLowerCase()).toBe(color);
 expect(()=>projectZuiDocument(review)).not.toThrow();
 }
 expect(normalizeZuiDocument(document)).toEqual(before); expect(normalizeZuiDocument(source)).toEqual(original);
 },30000);
});
