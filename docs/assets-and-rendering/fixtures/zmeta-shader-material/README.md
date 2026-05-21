# ZMeta Shader Material Fixture

This fixture mirrors a project `assets/` directory for the `.zmeta` shader/material assetization contract.

- `assets/shaders/unlit_shader.zmeta` is a compound shader root.
- `assets/shaders/unlit_shader/unlit.zshader` describes WGSL files, entry points, material properties, and texture slots.
- `assets/shaders/unlit_shader/unlit.wgsl` is the shader source and deliberately references every declared property and texture-slot name so the runtime WGSL capture diagnostics stay clean.
- `assets/materials/hero_unlit.zmaterial` references the shader with `{ uuid, url }` and stores overrides plus texture slots.
- `assets/materials/hero_unlit.zmaterial.zmeta` is the material sidecar.
