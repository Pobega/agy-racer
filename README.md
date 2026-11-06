# agy-racer

`agy-racer` is an early playable prototype of a cel-shaded, low-poly antigravity hover racer inspired by *Wipeout*, *F-Zero GX*, and *Jet Set Radio*.

---

## Project Status & Roadmap

The game is currently in **early prototype** form. The current build includes core hover physics, a single showcase circuit, AI test ships, local ghost recording, and an in-game 3D track editor.

### Planned Features / Roadmap

- **Refined Craft Handling & Physics**: Deeper airbrake drift feel, weight classes, and tighter track-surface feedback.
- **Ghost Racers & Time Trials**: Racing against personal-best and downloadable ghost holograms.
- **Zone Mode**: High-speed survival mode where ship speed increases continuously across colour-shifting zones.
- **Online Multiplayer**: Peer-to-peer and room-code multiplayer racing.
- **Expanded Circuits, Biomes & Audio**: Multiple tracks, distinct cel-shaded visual themes, and synth/drum-and-bass audio & SFX.

---

## Target Platforms

- **Linux Desktop** (primary development & tested platform)
- **Web / WASM** (browser play)
- **Steam / Desktop** (Windows & macOS)
- **PortMaster ARM64 Handhelds** (retro Linux handheld consoles)

---

## Controls

### Racing Controls

| Action | Keyboard | Gamepad (Controller) |
| :--- | :--- | :--- |
| **Accelerate** | `W` / `Up Arrow` | South / East Face Button (`A` / `B`) |
| **Steer (Left / Right)** | `A` / `D` or `Left` / `Right Arrow` | Left Stick X / D-Pad Left & Right |
| **Airbrakes (Drift)** | `Q` (Left) / `E` (Right) | Left Triggers (`L1`/`L2`) / Right Triggers (`R1`/`R2`) |
| **Pitch (Nose Up / Down)** | `S` or `K` (Up) / `Left Shift` or `I` (Down) | Left Stick Y |
| **Hyper Boost** | `Space` | North / West Face Button (`Y` / `X`) |
| **Side Attack (Barge)** | `Z` (Left) / `C` (Right) | — |
| **Cycle Biome Palette** | `B` | — |
| **Hot-Reload Configs & Textures** | `F5` | — |
| **Toggle UDP LAN Sync** | `F2` | — |
| **Reset Race Grid** | `Backspace` | — |
| **Toggle 3D Track Editor** | `Tab` | — |

### 3D Track Editor Controls (`Tab`)

| Action | Key / Input |
| :--- | :--- |
| **Free-Fly Camera Look** | Hold Right Mouse Button + Move Mouse |
| **Free-Fly Camera Movement** | Hold Right Mouse Button + `W` / `A` / `S` / `D` (Move), `E` / `Space` (Up), `Q` (Down) |
| **Fast Camera Speed** | Hold `Left Shift` while flying |
| **Select Control Point** | `.` or `Page Up` (Next Point), `,` or `Page Down` (Previous Point) |
| **Switch Spline Segment** | `V` |
| **Move Selected Point (Horizontal)** | Arrow Keys (`Up` / `Down` for Z, `Left` / `Right` for X) |
| **Move Selected Point (Elevation)** | `R` (Up / +Y), `F` (Down / -Y) |
| **Fast Node Move** | Hold `Left Shift` while translating |
| **Banking / Roll Angle** | `Q` (Roll Left) / `E` (Roll Right) *(RMB released)* |
| **Track Width** | `[` (Narrower) / `]` (Wider) |
| **Continuous Curvature** | `-` (Flatten/Invert) / `=` (Deepen Channel) |
| **Cycle Curvature Preset** | `C` (Cycles Flat Ribbon -> Half-Pipe -> 360° Cylinder -> Outside Ridge) |
| **Toggle Barrier Mode** | `M` (Cycles None -> Left -> Right -> Both) |
| **Toggle Track Floor / Chasm** | `G` (Removes floor geometry for jump chasms) |
| **Cycle Surface Type** | `T` (Cycles Normal -> Boost Pad -> Recharge Strip -> Low-Friction Ice) |
| **Insert New Node** | `N` (Splits segment after currently selected node) |
| **Delete Selected Node** | `Delete` or `Backspace` (Minimum 4 points per segment) |
| **Save Track to Disk** | `Ctrl + S` or `K` (Saves active track JSON in `assets/tracks/`) |

---

## Building & Running

Ensure you have Rust and Cargo installed ([rustup.rs](https://rustup.rs)). Building `raylib-sys` requires C/C++ development tools and system graphics libraries.

### Linux (Tested)

Linux is the primary development environment and the only platform actively tested so far.

#### Fedora
```bash
sudo dnf install alsa-lib-devel mesa-libGL-devel libX11-devel libXrandr-devel libXi-devel libXcursor-devel libXinerama-devel libwayland-client-devel libxkbcommon-devel cmake clang
```

#### Ubuntu / Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  libasound2-dev \
  libx11-dev \
  libxrandr-dev \
  libxi-dev \
  libgl1-mesa-dev \
  libglu1-mesa-dev \
  libxcursor-dev \
  libxinerama-dev \
  libwayland-dev \
  libxkbcommon-dev \
  cmake \
  clang
```

#### Run the Game
```bash
cargo run --release
```

#### Run Unit Tests
```bash
cargo test
```

### macOS & Windows (Untested)

Builds on macOS and Windows are currently **untested** as development has only exercised the Linux path so far. The standard prerequisites are:

- **macOS (Untested)**:
  ```bash
  xcode-select --install
  brew install cmake
  ```
- **Windows (Untested)**: Install Visual Studio C++ Build Tools and CMake.

---

## Documentation & Links

- **[Architecture & Engine Guide (`ARCHITECTURE.md`)](ARCHITECTURE.md)**: Developer guide detailing the track-space coordinate system, spline baking, cross-section morphing, 60 Hz simulation loop, and asset hot-reloading.
- **[Contributing Guide (`CONTRIBUTING.md`)](CONTRIBUTING.md)**: Our "Stake Your Reputation" policy and coding/maintainability standards for human and AI-assisted contributions.
- **[AI Development Disclosure (`DISCLOSURE.md`)](DISCLOSURE.md)**: Overview of AI involvement and human review guidelines.
- **[License (`LICENSE.md`)](LICENSE.md)**: Licensed under the Apache License, Version 2.0.
