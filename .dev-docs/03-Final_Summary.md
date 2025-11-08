# Mongoose: Final Summary & Implementation Plan

## I. Mandate Compliance Report

This report confirms that all "Non-Negotiable" mandates and rules from the user query have been followed in the production of the design documents.

1. **Design Completeness (Zero Placeholders)**: ✓ All architectural decisions, FOSS technology selections, data models, component interactions, and user flows have been fully specified in `.dev-docs/02-Systems_Design_Document.md`. There are no "TBD" items.

2. **Strict Project & AI Hygiene**: ✓
   - The `.dev-docs` folder has been created
   - All AI-generated documentation (this report, the SDD, the Research) is correctly placed within `.dev-docs/`
   - The main project directory remains pristine, populated only with README.md, CHANGELOG.md, .gitignore, and the .dev-docs folder
   - The root .gitignore file explicitly adds `.dev-docs/` to ensure no AI meta-work is committed

3. **FOSS-Only Ecosystem**: ✓ All proposed dependencies (Carbonyl, ratatui, adblock-rust, chafa, rusqlite, redb) are FOSS, and their licenses have been documented in both the Research Report and the README.md.

4. **Self-Contained Architecture**: ✓ The design relies solely on FOSS libraries and the forked Chromium core. It requires no external, closed-source, or private APIs.

5. **Transparent CHANGELOG Discipline**: ✓ The CHANGELOG.md file has been created and pre-filled under the "Unreleased" section with all initial and innovated features, referencing the SDD's feature specification list.

6. **FOSS Attribution**: ✓ The README.md file includes a dedicated "FOSS Attribution" section documenting all core dependencies, as specified in the SDD.

7. **Creator Attribution**: ✓ The README.md file explicitly credits orpheus497 as the project's designer and originator.

## II. Architectural Summary

The finalized mongoose architecture is a **Rust-centric, hybrid-rendering browser** built upon a fork of the fathyb/carbonyl (Chromium) project.

This architecture solves the user's conflicting mandates by:

1. Using the Carbonyl fork (mongoose-core) to provide 60fps, modern web (JS/WebGL) rendering to a TTY
2. Replacing the default Carbonyl executable with a ratatui-based TUI shell (mongoose-tui) to provide a modern, tabbed interface
3. Injecting all logic into a Rust shared library (libmongoose) that sits between the C++ core and the Rust TUI
4. Integrating adblock-rust into libmongoose and hooking it into Chromium's network stack via a URLLoaderRequestInterceptor to provide Brave-like privacy
5. Integrating chafa into libmongoose and patching Chromium's Skia renderer (TextCaptureDevice) to create a Hybrid Renderer that provides fast block-rendering for video and high-fidelity chafa art for static images
6. Using a hybrid rusqlite + redb datastore for persistent, high-performance storage of history, bookmarks, and caches

## III. Next Steps for Development Team

The development team should proceed with the following high-level implementation plan.

### Phase 0: Environment Setup

#### Environment Setup (C++)
**Priority**: CRITICAL (Highest-risk task)

The team must successfully set up the Chromium build environment (depot_tools, gn, ninja) and perform a clean compilation of the unmodified fathyb/carbonyl fork. This validates the build chain.

**Tasks**:
- Install depot_tools
- Run `./scripts/gclient.sh sync` to fetch Chromium sources
- Apply Carbonyl patches: `./scripts/patches.sh apply`
- Configure build: `./scripts/gn.sh args out/Default`
- Build: `./scripts/build.sh Default`
- Verify the build produces a working `headless_shell` binary

#### Environment Setup (Rust)
Standard rustup setup for libmongoose and mongoose-tui.

**Tasks**:
- Verify Rust toolchain is installed (stable or nightly)
- Build the existing libcarbonyl: `cargo build`
- Verify the build produces `libcarbonyl.so`/`.dylib`/`.dll`

### Phase 1: The "Spike" (Proof of Concept)

**Goal**: Establish the basic mongoose-tui → mongoose-core → libmongoose IPC connection and get the unmodified Carbonyl renderer to paint inside a ratatui widget.

**Tasks**:

1. **Fork and Rename**:
   - Copy the existing Carbonyl Rust `src/` directory to a new `libmongoose/` directory
   - Update `Cargo.toml` to rename the library from "carbonyl" to "mongoose"
   - Verify the renamed library still compiles

2. **Create mongoose-tui Scaffold**:
   - Create a new `mongoose-tui/` directory with a basic Cargo.toml
   - Add dependencies: `ratatui`, `crossterm`, and path dependency on `libmongoose`
   - Write a minimal `main.rs` that:
     - Initializes crossterm and enters alternate screen
     - Creates a basic ratatui layout with a placeholder "browser pane"
     - Polls for keyboard events (just `q` to quit for now)

3. **Establish IPC**:
   - In libmongoose, add a public API function `initialize()` that spawns the mongoose-core subprocess
   - In mongoose-tui's `main.rs`, call `libmongoose::initialize()`
   - Verify that the subprocess starts and the Mojo IPC connection is established
   - Add a public API function `get_frame_buffer()` in libmongoose that returns the current TTY buffer
   - In mongoose-tui, create a custom ratatui widget that renders this buffer
   - Navigate to a simple page (e.g., `https://example.com`) and verify it renders in the TUI

**Success Criteria**: A working mongoose-tui binary that displays a web page (using the unmodified Carbonyl renderer) inside a ratatui frame.

### Phase 2: Data & Privacy

**Goal**: Implement persistent storage and privacy blocking.

**Tasks**:

1. **Implement DataStore Service** (`libmongoose/src/data.rs`):
   - Add rusqlite and redb dependencies to `libmongoose/Cargo.toml`
   - Create the `DataStore` struct
   - Implement initialization logic to create the SQLite database at `$CONFIG_DIR/mongoose/mongoose.sqlite`
   - Create the database schema (history, bookmarks, site_permissions, settings tables)
   - Implement initialization logic to create the redb database at `$CONFIG_DIR/mongoose/cache.redb`
   - Create the redb schema (adblock_lists, adblock_cache, session tables)
   - Implement public API methods: `get_history()`, `add_bookmark()`, `get_filter_list()`, etc.

2. **Implement PrivacyEngine Service** (`libmongoose/src/privacy.rs`):
   - Add adblock-rust dependency to `libmongoose/Cargo.toml`
   - Create the `PrivacyEngine` struct
   - Implement initialization logic:
     - Spawn a background thread
     - Fetch enabled filter list URLs from DataStore
     - Check redb cache for fresh versions
     - Download and parse lists if stale (>24 hours)
     - Build `adblock::Engine` from cached filter sets
     - Store Engine in an `Arc<Mutex<Engine>>` for thread-safe access
   - Implement `check(request: &NetworkRequestData) -> BlockResult` method

3. **Implement C++ Network Interceptor**:
   - Create `mongoose-core/content/browser/mongoose_url_loader_interceptor.h` and `.cc`
   - Implement the `content::URLLoaderRequestInterceptor` interface
   - In `MaybeCreateLoader`, make a synchronous Mojo call to `CheckNetworkRequest`
   - Hook the interceptor into Chromium's ContentBrowserClient

4. **Update Mojo API** (`mongoose-core/mojo/mongoose.mojom`):
   - Add the `CheckNetworkRequest` method signature
   - Define `NetworkRequestData` and `BlockResult` Mojo structs
   - Regenerate Mojo bindings

5. **Wire Up in libmongoose**:
   - In `libmongoose/src/mojo_impl.rs`, implement the `CheckNetworkRequest` Mojo handler
   - Call `PrivacyEngine::check()` and return the result

**Success Criteria**: Navigate to a page with ads. Verify in browser devtools (or logs) that ad requests are blocked with `net::ERR_BLOCKED_BY_CLIENT`.

### Phase 3: Hybrid Rendering

**Goal**: Implement the chafa-based ASCII art mode for static images.

**Tasks**:

1. **Implement AsciiEngine Service** (`libmongoose/src/render.rs`):
   - Add chafa and chafa-sys dependencies to `libmongoose/Cargo.toml`
   - Create the `AsciiEngine` struct
   - In initialization, create a `chafa::Config` with appropriate settings (full Unicode, truecolor, ordered dither)
   - Implement `render(bitmap: &[u8], width: u32, height: u32) -> String` method that uses chafa to convert the bitmap to a Unicode string

2. **Patch MongooseTextCaptureDevice**:
   - In `mongoose-core/ui/gfx/mongoose_text_capture_device.cc`, add the global `g_hybrid_ascii_mode` flag
   - Modify the `drawBitmap` method to check the flag
   - If true, call the new `DrawImageBitmap` Mojo method instead of the default block rendering

3. **Update Mojo API**:
   - Add `DrawImageBitmap` method to `mongoose.mojom`
   - Add `SetHybridAsciiMode` method to `mongoose.mojom`
   - Regenerate Mojo bindings

4. **Wire Up in libmongoose**:
   - In `libmongoose/src/mojo_impl.rs`, implement the `DrawImageBitmap` handler
   - Call `AsciiEngine::render()` on the received bitmap
   - Composite the resulting chafa buffer onto the main FrameBuffer
   - Implement a public API method `toggle_ascii_mode()` that flips the internal flag and calls the `SetHybridAsciiMode` Mojo method

5. **Add TUI Keybinding**:
   - In `mongoose-tui/src/input.rs`, add a handler for `Ctrl+A`
   - Call `libmongoose::toggle_ascii_mode()`

**Success Criteria**: Navigate to a page with images. Press `Ctrl+A`. Verify that images are now rendered as high-fidelity chafa art while the rest of the page remains as fast block rendering.

### Phase 4: TUI & Polish

**Goal**: Build out the full user interface with tabs, URL bar, command palette, and settings.

**Tasks**:

1. **Implement AppState** (`mongoose-tui/src/app.rs`):
   - Define the `AppState` struct with fields for tabs, URL bar, input mode, etc.
   - Implement state transition logic (switching between Normal/Insert/Command modes)
   - Implement tab management (create, close, switch)

2. **Implement UI Widgets** (`mongoose-tui/src/ui.rs`):
   - `draw_tabs()`: Render the tab bar using `ratatui::widgets::Tabs`
   - `draw_url_bar()`: Render the URL input bar
   - `draw_browser_pane()`: Fetch the frame buffer from libmongoose and render it
   - `draw_command_palette()`: Render a floating command palette overlay

3. **Implement Input Handling** (`mongoose-tui/src/input.rs`):
   - Implement the full keybinding map from Table 3 in the SDD
   - Handle mode-specific input (URL bar typing, command palette search, normal browsing)

4. **Implement Command Palette** (`mongoose-tui/src/command.rs`):
   - Define available commands (e.g., "Clear Cache", "Add Bookmark", "Toggle ASCII Mode")
   - Implement fuzzy search/filtering
   - Wire up command execution to libmongoose API calls

5. **Implement Settings UI**:
   - Create a settings screen accessible from the command palette
   - Allow users to:
     - Enable/disable filter lists
     - Set default homepage
     - Configure ASCII mode default state
   - Persist settings to the rusqlite database

6. **Implement Bookmarks/History UI**:
   - Create browsable lists of bookmarks and history
   - Allow filtering and navigation
   - Wire up to DataStore API

**Success Criteria**: A fully functional, polished TUI browser with all features from the SDD working end-to-end.

### Phase 5: Testing & Documentation

**Tasks**:

1. **Automated Testing**:
   - Write Rust unit tests for libmongoose components (PrivacyEngine, AsciiEngine, DataStore)
   - Write integration tests for the TUI
   - Set up CI/CD pipeline for automated testing

2. **User Documentation**:
   - Write user-facing documentation (installation, usage, keybindings)
   - Create a troubleshooting guide
   - Document common workflows (bookmarking, privacy settings, etc.)

3. **Developer Documentation**:
   - Document the build process
   - Create a contribution guide
   - Document the architecture for future maintainers

**Success Criteria**: Comprehensive test coverage and documentation enabling both users and contributors to work effectively with mongoose.

## IV. Risk Assessment

### High-Risk Components

1. **Chromium Build Environment**: Setting up and maintaining the Chromium build is the single highest-risk task. It requires significant disk space (~100GB), time (~1 hour on a fast machine), and expertise. Mitigation: Dedicate a team member with C++ and build system experience to own this.

2. **Synchronous Network Interceptor**: The synchronous Mojo call in the network interceptor could potentially deadlock or cause performance issues if not implemented correctly. Mitigation: Ensure the adblock-rust check is purely in-memory and profiled for microsecond-level performance.

3. **Mojo IPC Complexity**: Extending the Mojo API and maintaining compatibility with Chromium updates is complex. Mitigation: Keep the Mojo surface area minimal and well-documented.

### Medium-Risk Components

1. **chafa Integration**: The chafa library is C-based and may have edge cases or performance issues. Mitigation: Thoroughly test with a variety of image formats and sizes.

2. **TUI Performance**: Rendering at 60fps in a terminal requires careful optimization. Mitigation: Profile early and often. Use ratatui's double-buffering and minimize redraws.

### Low-Risk Components

1. **DataStore**: rusqlite and redb are mature, well-tested libraries. Risk is minimal.

2. **ratatui TUI**: ratatui is actively maintained and widely used. Risk is minimal.

## V. Success Metrics

The mongoose project will be considered successful when:

1. **Functional Completeness**: All features from the SDD are implemented and working
2. **Performance**: The browser maintains 60fps rendering and 0% idle CPU usage
3. **Privacy**: Ad and tracker blocking matches or exceeds Brave's effectiveness
4. **Usability**: The TUI is intuitive and supports all common browsing workflows
5. **Stability**: The browser can browse the top 100 websites without crashes
6. **FOSS Purity**: All dependencies remain FOSS with no proprietary components

## VI. Conclusion

The mongoose project is architecturally sound, technically feasible, and aligned with all user mandates. The use of Carbonyl as a foundation, combined with strategic Rust integrations (adblock-rust, chafa, ratatui), provides a clear path to a production-ready, privacy-first, terminal-native web browser.

The phased implementation plan minimizes risk by establishing core functionality first (Phase 1), then adding privacy and data persistence (Phase 2), then enhancing the rendering (Phase 3), and finally polishing the user experience (Phase 4).

With dedicated focus on the high-risk Chromium build environment and careful attention to IPC performance, the development team has everything needed to bring mongoose from design to reality.

**Project Designer & Originator**: orpheus497

**Architecture & Systems Design**: AI Research Team

**Status**: Design Complete, Ready for Implementation

---

*This document represents the final deliverable of the mongoose research and design phase. All subsequent work should reference this document and the accompanying Systems Design Document (02-Systems_Design_Document.md) as the authoritative source of architectural truth.*
