# `agy-racer` — Developer & Maintenance Guide

This guide explains how the codebase is organized so you can easily modify physics, textures, biomes, tracks, and rendering **by hand** without needing an AI.

---

## 1. Quick-Reference: Where to Edit Things (No Recompile Needed!)

Press **`F5`** at any time while racing to live-reload these files:

| What you want to change | File to edit | How it works |
| :--- | :--- | :--- |
| **Ship speed, grip, airbrakes, boost, or camera** | [`assets/config/physics.json`](assets/config/physics.json) | Edit any number (e.g. `"max_speed": 95.0` or `"lateral_grip": 14.0`), save, and press **`F5`** in-game! |
| **Biome color palettes (sky, fog, road tint, neon)** | [`assets/config/biomes.json`](assets/config/biomes.json) | Add or tweak `[R, G, B]` color arrays, save, press **`F5`**, and press **`B`** to cycle biomes. |
| **Starting ship roster (names, colors, grid slots)** | [`assets/config/roster.json`](assets/config/roster.json) | Edit ship names, primary/accent `[R, G, B]` colors, and starting `(start_s, start_r)` offsets. |
| **Road & wall textures** | `assets/textures/*.png` | Automatically exported as standard `.png` images on first run (`road_cel.png`, `road_boost.png`, `road_recharge.png`, `road_ice.png`, `wall_hazard.png`). Open in Aseprite/GIMP/Krita, save, and press **`F5`**! |
| **3D Track layouts** | `assets/tracks/circuit_02.json` | Press **`Tab`** in-game to open the visual 3D Spline Editor, or edit the JSON control points directly. |

---

## 2. Codebase Map (`src/`)

Every file in `src/` has a single, clear responsibility:

- **[`src/main.rs`](src/main.rs):** Window setup, keyboard/gamepad input polling, fixed 60 Hz simulation loop, and chase camera interpolation.
- **[`src/config.rs`](src/config.rs):** Plain Rust structs (`PhysicsConfig`, `BiomePalette`) that load `assets/config/*.json`.
- **[`src/math.rs`](src/math.rs):** `Vec3` math and Catmull-Rom spline interpolation (`catmull_rom_pos`, `parallel_transport`).
- **[`src/track.rs`](src/track.rs):** Track data structures (`TrackDef`, `ControlPoint`) and the arc-length spline baker (`BakedTrack`). **Contains zero rendering code.**
- **[`src/physics.rs`](src/physics.rs):** Pure ship movement, airbrakes, shield/boost energy, and ship-to-ship collision math in `(s, r, h)` track coordinates. **Contains zero rendering code.**
- **[`src/render/`](src/render/mod.rs):** Modular Cel-Shaded 3D renderer:
  - [`src/render/mod.rs`](src/render/mod.rs): Shared rendering constants, lighting/fog helpers, and textured 3D quad emitter.
  - [`src/render/track.rs`](src/render/track.rs): Track deck slices, barriers, under-track keel trusses, pylons, and ship shadows (`draw_world_3d`).
  - [`src/render/ship.rs`](src/render/ship.rs): Ship mesh blueprint (`ship_dims`), articulated airbrake flaps, ink outlines, thrusters, and contrails (`draw_cel_ship`).
  - [`src/render/scenery.rs`](src/render/scenery.rs): Sky gradient, horizon sun, and track-defined cityscape towers (`draw_sky_backdrop`).
  - [`src/render/textures.rs`](src/render/textures.rs): `.png` texture loader and procedural fallback generators (`RenderAssets`).
- **[`src/editor.rs`](src/editor.rs):** The built-in Press-`Tab` 3D track editor.
- **[`src/ui.rs`](src/ui.rs):** 2D HUD (speedometer, shield bar, minimap, anime speed-lines).
- **[`src/ghost.rs`](src/ghost.rs):** Records 10 Hz keyframes of your best lap and replays a translucent hologram ghost.
- **[`src/net.rs`](src/net.rs):** 30 Hz UDP / P2P room-code packet synchronization.

---

## 3. Understanding the Track-Space Coordinate System `(s, r, h)`

Instead of fighting a 3D mesh-collision engine inside loops and half-pipes, every ship's position is stored in **3 simple floats** on `Ship` (`src/physics.rs`):
1. **`s` (Longitudinal distance):** Meters traveled along the track spline from the start line (`0.0 .. seg.total_length`).
2. **`r` (Lateral offset):** Meters left (`-`) or right (`+`) of the centerline (`-width/2 .. +width/2`).
3. **`h` (Hover height):** Meters above the track surface along the local surface normal (`~1.25m` when grounded).

To convert `(s, r, h)` into a 3D world position at any point:
```rust
let sample = track.segment_or_main(segment_id).sample_at(s);
let frame = sample.evaluate_cross_section(r);
let world_pos = frame.surface_pos + frame.normal * h;
```
Because `evaluate_cross_section(r)` smoothly bends the surface normal based on `curvature` (`0.0` = flat, `0.5` = half-pipe, `1.0` = 360° cylinder), the ship physics in `src/physics.rs` doesn't even have to know whether the track is upside-down or inside a tube!

---

## 4. Spline Baker, Cross-Section Morphing & 60 Hz Simulation

### 3D Catmull-Rom Splines & Parallel-Transport Frames
Tracks are authored as 3D Catmull-Rom splines (`TrackDef` and `TrackSegmentDef` in `src/track.rs`) and baked into high-resolution uniform arc-length sample points (`BakedTrack`). Reference frames are propagated along the spline curve using **parallel-transport frames** (Bishop frames). This avoids gimbal lock, mathematical singularities, and sudden 180° orientation flipping on loops, vertical climbs, corkscrews, or inverted sections. Banking angles specified at control points are smoothly applied as roll offsets around the forward tangent.

### Continuous Cross-Section Curvature
Each spline node supports continuous cross-section curvature in `[-1.0, 1.0]`:
- **Flat ribbons (`0.0`)**: Standard planar racing deck with vertical side barriers.
- **Half-pipes (`0.5`)**: U-shaped channel providing lateral banking that naturally guides craft toward the centerline.
- **Full 360° cylinders (`1.0`)**: Enclosed circular tubular tunnels where craft can race across floor, walls, and ceiling.
- **Outside ridges (`-0.5`)**: Convex crowned track decks where sliding too far laterally risks falling off.

At runtime, `sample.evaluate_cross_section(r)` analytically calculates the exact surface position, lateral cross-track tangent, and outward surface normal for any craft offset `r \in [-width/2, width/2]`.

### Wall Modes & Floorless Jump Chasms
- **Wall Modes**: Control nodes specify barrier presence via `WallMode` (`None`, `LeftOnly`, `RightOnly`, or `Both`). Barrier-less edges create lethal drop-off hazards on corners.
- **Floorless Jump Chasms**: Nodes can disable floor geometry (`has_floor = false`), omitting road deck slices to create mid-air chasms that require sufficient forward speed to jump across.

### 60 Hz Fixed-Timestep Simulation
The core physics loop runs on a deterministic 60 Hz fixed timestep accumulator in `src/main.rs`:
- Input polling and variable frame rendering are decoupled from the physics loop. The accumulator consumes delta time in discrete 1/60-second slices (`1.0 / 60.0`), preventing physics tunneling, variable jump trajectories, or framerate-dependent grip.
- Hovercraft mechanics in `src/physics.rs` run purely in track-space `(s, r, h)`, modeling hover spring repulsion, surface grip, dual airbrake drift friction, hyper-boost shield drain, lateral barge attacks, and craft-to-craft elastic collisions.
- Autonomous AI ships follow spline trajectories with lateral lane offset dispersion and obstacle avoidance.
