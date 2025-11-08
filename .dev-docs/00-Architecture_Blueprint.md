# Mongoose Architecture Blueprint

## Current State Analysis

This document provides an analysis of the current Carbonyl codebase structure and outlines the transformation path to the mongoose architecture.

### Current Repository Structure

```
mongoose/ (currently a Carbonyl fork)
├── .cargo/                     # Cargo configuration
├── .github/                    # GitHub workflows
├── chromium/                   # Chromium source and build artifacts
│   ├── depot_tools/            # Google's build tools
│   ├── patches/                # Patches applied to Chromium
│   │   ├── chromium/           # 14 patches for Chromium modifications
│   │   ├── skia/               # Skia rendering patches
│   │   └── webrtc/             # WebRTC patches
│   └── .gclient                # Chromium build configuration
├── scripts/                    # Build and deployment scripts
│   ├── build.sh                # Main build script
│   ├── gclient.sh              # Chromium source sync
│   ├── gn.sh                   # GN configuration
│   ├── patches.sh              # Patch management
│   └── run.sh                  # Runtime launcher
├── src/                        # Rust library (libcarbonyl)
│   ├── browser/                # Browser control and Mojo interface
│   │   └── carbonyl.mojom      # IPC interface definition
│   ├── cli/                    # Command-line interface
│   ├── gfx/                    # Graphics and rendering
│   ├── input/                  # Input handling (keyboard, mouse)
│   ├── output/                 # TTY output formatting
│   ├── ui/                     # UI coordination
│   ├── utils/                  # Shared utilities
│   └── lib.rs                  # Library root
├── build.rs                    # Rust build script (sysroot linking)
├── Cargo.toml                  # Rust package manifest
└── Dockerfile                  # Container build definition
```

### Current Component Analysis

#### 1. Chromium Runtime (C++)

**Location**: Built from `chromium/` directory after applying patches

**Key Patches**:
- `0001-Add-Carbonyl-library.patch`: Integrates the Rust library into Chromium
- `0002-Add-Carbonyl-service.patch`: Sets up the Mojo IPC service
- `0009-Bridge-browser-into-Carbonyl-library.patch`: Main browser bridge
- `0013-Refactor-rendering-bridge.patch`: Rendering infrastructure

**Build Output**: `chromium/src/out/Default/headless_shell` (the browser binary)

#### 2. libcarbonyl (Rust)

**Location**: `src/`

**Current Modules**:
- **browser**: Mojo interface definitions and browser control
  - `carbonyl.mojom`: Defines `CarbonylRenderService` with `DrawText` method
- **cli**: Command-line argument parsing
- **gfx**: Graphics primitives and rendering logic
- **input**: Keyboard and mouse event handling
- **output**: TTY escape sequence generation
- **ui**: UI state management
- **utils**: Helper functions

**Current Mojo Interface** (`src/browser/carbonyl.mojom`):
```mojom
module carbonyl.mojom;

struct TextData {
    string contents;
    gfx.mojom.RectF bounds;
    skia.mojom.SkColor color;
};

interface CarbonylRenderService {
    DrawText(array<TextData> data);
};
```

#### 3. No Dedicated TUI

Currently, the Carbonyl project does not have a separate TUI application. The Rust library (`libcarbonyl`) is loaded as a shared library by the Chromium `headless_shell` binary, which directly outputs to the terminal.

## Mongoose Transformation Plan

### Phase 1: Rename and Restructure (Current Phase)

**Status**: ✅ **Initial documentation and planning complete**

**Completed**:
- [x] Created `.dev-docs/` with comprehensive design documentation
- [x] Updated `.gitignore` to exclude `.dev-docs/`
- [x] Created mongoose-branded `README.md` with FOSS attribution
- [x] Created `CHANGELOG.md` following Keep a Changelog format

**Next Steps**:
1. Rename `src/` to `libmongoose/`
2. Update `Cargo.toml` to rename library from "carbonyl" to "mongoose"
3. Update all internal module references
4. Update Chromium patches to reference "mongoose" instead of "carbonyl"

### Phase 2: Create mongoose-tui

**Goal**: Extract the terminal output logic from libmongoose into a dedicated ratatui-based TUI application

**New Directory Structure**:
```
mongoose-tui/
├── Cargo.toml                  # Dependencies: ratatui, crossterm, libmongoose
└── src/
    ├── main.rs                 # Main event loop, TUI initialization
    ├── app.rs                  # AppState struct, state management
    ├── ui.rs                   # Ratatui widget rendering
    ├── input.rs                # Input handling and keybindings
    └── command.rs              # Command palette logic
```

**Key Changes**:
- `libmongoose` becomes a pure logic library (no direct terminal output)
- `mongoose-tui` handles all terminal I/O via ratatui
- Communication: `mongoose-tui` calls `libmongoose` API, which communicates with `mongoose-core` via Mojo

### Phase 3: Enhance libmongoose

**Goal**: Add privacy, ASCII art, and data persistence capabilities

**New Directory Structure**:
```
libmongoose/
├── Cargo.toml                  # Add deps: adblock-rust, chafa-sys, rusqlite, redb
└── src/
    ├── browser/                # (existing) Browser control and Mojo interface
    │   └── mongoose.mojom      # (renamed from carbonyl.mojom)
    ├── cli/                    # (existing) Command-line interface
    ├── gfx/                    # (existing) Graphics and rendering
    ├── input/                  # (existing) Input handling
    ├── output/                 # (existing) TTY output formatting
    ├── ui/                     # (existing) UI coordination
    ├── utils/                  # (existing) Shared utilities
    ├── privacy.rs              # (new) PrivacyEngine with adblock-rust
    ├── render.rs               # (new) AsciiEngine with chafa
    ├── data.rs                 # (new) DataStore with rusqlite/redb
    ├── mojo_impl.rs            # (new) Mojo IPC handler implementation
    └── lib.rs                  # Updated library root
```

**New Mojo Methods** (to be added to `mongoose.mojom`):
```mojom
// Network blocking (Privacy Engine)
struct NetworkRequestData {
    url.mojom.Url url;
    url.mojom.Origin initiator;
    int32 resource_type;
};

struct BlockResult {
    bool should_block;
};

// Hybrid rendering (ASCII Art Engine)
struct BitmapData {
    handle<shared_buffer> bitmap;
    gfx.mojom.Rect bounds;
};

interface MongooseRenderService {
    // Existing
    DrawText(array<TextData> data);

    // New methods
    CheckNetworkRequest(NetworkRequestData request) => (BlockResult result);
    DrawImageBitmap(BitmapData data);
    SetHybridAsciiMode(bool enabled);
};
```

### Phase 4: Extend mongoose-core (C++)

**Goal**: Add network interception and hybrid rendering support to Chromium

**New Files to Create**:
```
chromium/patches/chromium/
├── 0015-Add-URL-loader-interceptor.patch
└── 0016-Add-hybrid-ASCII-mode-to-Skia.patch
```

**Patch 0015**: Implements `MongooseURLLoaderInterceptor`
- Hooks into Chromium's network stack
- Makes synchronous Mojo calls to `CheckNetworkRequest`
- Blocks requests based on PrivacyEngine decisions

**Patch 0016**: Modifies `TextCaptureDevice` (Skia backend)
- Adds `g_hybrid_ascii_mode` global flag
- Modifies `drawBitmap` to call `DrawImageBitmap` when flag is enabled
- Allows chafa rendering of static images

### Phase 5: Integration and Testing

**Goal**: Wire all components together and verify end-to-end functionality

**Key Integration Points**:
1. **mongoose-tui** spawns **mongoose-core** as a subprocess
2. **mongoose-core** loads **libmongoose** as a shared library
3. **mongoose-core** establishes Mojo IPC with **libmongoose**
4. **libmongoose** exposes a public API consumed by **mongoose-tui**

**Data Flow Example** (Ad Blocking):
```
User navigates to URL
    ↓
mongoose-tui → libmongoose::navigate()
    ↓
libmongoose → Mojo IPC → mongoose-core
    ↓
mongoose-core → Network Request → URLLoaderInterceptor
    ↓
URLLoaderInterceptor → Mojo IPC → libmongoose::CheckNetworkRequest()
    ↓
libmongoose → PrivacyEngine::check() → adblock-rust
    ↓
adblock-rust → BlockResult → libmongoose → Mojo IPC → mongoose-core
    ↓
mongoose-core → Block or Allow request
```

## Migration Checklist

### Documentation & Planning
- [x] Create `.dev-docs/` directory
- [x] Write comprehensive design documents
- [x] Update `.gitignore`
- [x] Create mongoose-branded `README.md`
- [x] Create `CHANGELOG.md`
- [x] Create this architecture blueprint

### Rename & Restructure
- [ ] Rename `src/` → `libmongoose/`
- [ ] Update `Cargo.toml` (library name)
- [ ] Update `src/browser/carbonyl.mojom` → `libmongoose/src/browser/mongoose.mojom`
- [ ] Update all patch files (references to "carbonyl" → "mongoose")
- [ ] Update build scripts

### Create mongoose-tui
- [ ] Create `mongoose-tui/` directory structure
- [ ] Create `Cargo.toml` with ratatui dependencies
- [ ] Implement basic event loop in `main.rs`
- [ ] Implement `AppState` in `app.rs`
- [ ] Implement ratatui widgets in `ui.rs`
- [ ] Implement keybindings in `input.rs`

### Enhance libmongoose
- [ ] Add `privacy.rs` with adblock-rust integration
- [ ] Add `render.rs` with chafa integration
- [ ] Add `data.rs` with rusqlite/redb integration
- [ ] Add `mojo_impl.rs` for Mojo handler implementations
- [ ] Update `lib.rs` with new public API
- [ ] Update `mongoose.mojom` with new methods

### Extend mongoose-core
- [ ] Create URLLoaderInterceptor patch
- [ ] Create hybrid ASCII mode Skia patch
- [ ] Apply and test patches
- [ ] Verify Chromium still builds

### Integration & Testing
- [ ] Wire mongoose-tui → libmongoose
- [ ] Wire libmongoose → mongoose-core
- [ ] Test ad blocking end-to-end
- [ ] Test hybrid ASCII mode
- [ ] Test data persistence
- [ ] Test multi-tab functionality

## Current vs. Future Architecture Comparison

### Current (Carbonyl)

```
Terminal
    ↑ (Raw TTY output)
    |
┌─────────────────────────┐
│  Chromium Runtime       │
│  (headless_shell)       │
│  + libcarbonyl.so       │
│    (Rust library)       │
└─────────────────────────┘
```

### Future (Mongoose)

```
Terminal
    ↑ (ratatui widgets)
    |
┌─────────────────────────┐
│  mongoose-tui           │
│  (Rust binary)          │
│  [Tabs, URL, Palette]   │
└─────────────────────────┘
    ↑ (Function calls)
    |
┌─────────────────────────┐
│  libmongoose            │
│  (Rust library)         │
│  [Privacy, ASCII, Data] │
└─────────────────────────┘
    ↑ (Mojo IPC)
    |
┌─────────────────────────┐
│  mongoose-core          │
│  (Chromium + patches)   │
│  [Blink, V8, Network]   │
└─────────────────────────┘
```

## Development Workflow

### Building the Current State

```bash
# Build libcarbonyl (Rust library)
cargo build --release

# Build Chromium runtime (requires ~100GB disk, ~1 hour)
./scripts/gclient.sh sync
./scripts/patches.sh apply
./scripts/gn.sh args out/Default
./scripts/build.sh Default

# Run
./scripts/run.sh Default https://example.com
```

### Building the Future State

```bash
# Build mongoose-core (Chromium runtime)
./scripts/gclient.sh sync
./scripts/patches.sh apply
./scripts/gn.sh args out/Default
./scripts/build.sh Default

# Build libmongoose (core logic library)
cd libmongoose
cargo build --release

# Build mongoose-tui (main executable)
cd mongoose-tui
cargo build --release

# Run
./target/release/mongoose-tui https://example.com
```

## Risk Mitigation

### High-Risk Areas

1. **Chromium Build Complexity**
   - **Risk**: Build failures, patch conflicts
   - **Mitigation**: Maintain detailed build documentation, pin Chromium version

2. **Mojo IPC Stability**
   - **Risk**: Deadlocks, performance degradation
   - **Mitigation**: Profile IPC calls, use synchronous calls sparingly

3. **Rendering Performance**
   - **Risk**: Chafa too slow for 60fps
   - **Mitigation**: Hybrid mode (blocks for video, chafa for static images)

### Medium-Risk Areas

1. **Rust Crate Dependencies**
   - **Risk**: Version conflicts, breaking changes
   - **Mitigation**: Pin versions in `Cargo.toml`, regular updates

2. **Terminal Compatibility**
   - **Risk**: Different terminals have different capabilities
   - **Mitigation**: Test on multiple terminals (alacritty, kitty, iTerm2, etc.)

## Success Criteria

The transformation to mongoose will be considered successful when:

1. **Functional Parity**: All Carbonyl features work in mongoose
2. **Enhanced Privacy**: Ad/tracker blocking is functional and effective
3. **Hybrid Rendering**: ASCII mode produces high-fidelity static images
4. **Persistent Data**: History, bookmarks, and settings persist across sessions
5. **Modern TUI**: Multi-tab interface with command palette is intuitive
6. **Performance**: Maintains 60fps rendering and 0% idle CPU

---

**Last Updated**: 2025-11-08
**Status**: Phase 1 Complete, Ready for Phase 2
