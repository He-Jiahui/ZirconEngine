# Orb Run blockout

Open `zircon-project.toml` and its default `res://scenes/main.scene.toml`.
The scene uses built-in cube geometry and the default material, so imported
meshes, material authoring and terrain are unnecessary for this blockout.
The eight diamond-shaped cubes named Orb 01–08 are the pickups.

This content completes the existing Orb Run script fixture. The script and
package manifests retain the previous live project's behavior. Camera (ID 1),
Sun (ID 2) and Player (ID 4), including their bindings and transforms, are
preserved. Added content is a 24 × 24 ground, four rails, eight pickups, a drone,
and the clock/score role entities. Their enabled bindings supply `hp = 61` and
`hp = 1` respectively, matching the script's one-point sentinel convention.
Pickups and the drone supply their reset positions; the drone speed is 2.2.
All scripted entities are Dynamic; ground, rails and Sun are Static.

The authored scene is also installed at the editor's existing live project:
`/workspace/ZirconEngine-review/evidence/orbrun-20261023/state/projects/OrbRun/assets/scenes/main.scene.toml`.
That project's manifest, source script, compiled module and scene metadata were
preserved. The repository example is the reproducible source for this content.

Content validation establishes the scene structure, roles, defaults and preserved
entities. Stable Game presentation, Start/input, pickup/outcomes, pause, retry,
save/reopen and repeated Play/Stop still need product acceptance. Reset callback
ordering and diagonal movement semantics remain with Claude; this content does
not change them.
