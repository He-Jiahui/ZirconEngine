# Licenses and source provenance

The World of Claudecraft source used as the behavioral reference is MIT
licensed. Its checked-in license is owned by:

- `dev/world-of-claudecraft/LICENSE`

ZirconEngine is licensed under MIT OR Apache-2.0 as declared by the root
workspace.

The first Eastbrook Vale MVP asset closure is generated from
`contracts/m8_asset_selection.json`. `tools/m8_asset_codegen.mjs` reads every
file from pinned Git commit `7c10f280`, writes it below `assets/m8`, and records
its SHA-256, byte length, role and license id in `contracts/m8_assets.json`.
The pinned source `CREDITS.md` and `LICENSE` are materialized as
`assets/m8/licenses/source-CREDITS.md` and `source-LICENSE.txt`.

The selected closure uses these source license classes:

- KayKit Adventurers 2.0 character models, player skin textures and merged
  Character Animations 1.1 clips: CC0 1.0. The generated license row records
  the official Adventurers and Character Animations product pages instead of
  relying on the pinned target's older Adventures 1.0 credit;
- Quaternius creatures, foliage and village props: CC0 1.0;
- Kenney world props: CC0 1.0;
- ambientCG terrain textures: CC0 1.0;
- Poly Haven Vale HDR environment: CC0 1.0;
- three.js water normals: MIT;
- Cinzel and Alegreya Sans fonts: SIL Open Font License 1.1;
- CraftPix warrior ability icons: premium royalty-free commercial license held
  by the target project owner;
- @jamiecypher quest sound effects: CC0 1.0;
- World of ClaudeCraft backdrop, item icons and generated UI sound effects:
  project assets.

Author, source URL and license label are retained per license id in the generated
manifest. A second official source is retained when one generated asset combines
model and animation inputs. No license is inferred from a filename.
