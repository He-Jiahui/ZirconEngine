# Texture Importer Plugin

`texture_importer` owns the source-linked image, container, cubemap, texture
array, mip generation, normal convention, and supported transcode paths for the
first-party texture importer family. Runtime catalog selection remains the
authority that decides whether this provider is admitted.

Cubemap equirectangular conversion preserves the existing wrapped-U and
clamped-V bilinear result while loading each four-pixel neighborhood once per
output texel. Stacked RGBA array sources are validated for positive, evenly
divisible dimensions and copied as contiguous layer byte ranges instead of
passing full-width layers through the generic crop path.

Manifest source reads are still constrained by the current project-relative
path admission. Moving those reads behind the planned immutable source broker
and recording source dependency hashes remain separate Plugins07 milestones.
