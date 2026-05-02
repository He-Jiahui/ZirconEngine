# Navigation Backend Notice

This crate defines Zircon's Recast/Detour backend boundary. Upstream Recast Navigation C++ sources are vendored under `vendor/recastnavigation`, including Recast, Detour, DetourCrowd, and DetourTileCache. The vendored sources retain their original `License.txt`.

The Rust public facade still owns Zircon asset serialization, deterministic polygon query tests, and runtime/editor integration contracts. The native bridge currently verifies that Recast, Detour, DetourCrowd, and DetourTileCache compile and are reachable through a C ABI; moving baking, Detour query, TileCache obstacle carving, and DetourCrowd simulation fully behind that ABI is tracked as follow-up backend work.
