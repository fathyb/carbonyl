# Mongoose: Systems Design Document (SDD)

## I. Master Architecture

### A. Architectural Philosophy

The mongoose architecture is a **Rust-centric, hybrid-rendering browser**. It is a multi-process system composed of three primary, FOSS-based components that communicate over a high-performance IPC (Inter-Process Communication) bridge.

- **mongoose-core (C++)**: The forked fathyb/carbonyl runtime. This is a minimal set of C++ patches to the Chromium headless shell, network service, and Skia rendering backend. Its sole responsibility is to run the web page, execute JavaScript, and provide rendering and network data to the Rust core.

- **libmongoose (Rust)**: The "brain" of the browser. This is a Rust shared library (a fork of libcarbonyl) that is dynamically loaded by mongoose-core. It contains all application logic: the Privacy Engine (adblock-rust), the ASCII Art Engine (chafa), the DataStores (rusqlite+redb), and the rendering/input logic.

- **mongoose-tui (Rust)**: The "face" of the browser. This is a ratatui TUI application that consumes libmongoose as a library. It provides the application shell (tabs, URL bar, command palette) and hosts the main event loop.

### B. High-Level Component Diagram (Logical)

```
(User Terminal)
      |
      v
  <-- (User Input: Keys, Mouse)

┌──────────────────────────────────┐
│ mongoose-tui                     │
│ (ratatui TUI Shell)              │
│ (Main Event Loop)                │
│ (AppState)                       │
│                                  │
│ <--- (Function Calls) --->       │
│                                  │
└──────────────────────────────────┘
  <-- (TTY Output Buffer)

┌──────────────────────────────────┐
│ libmongoose                      │
│ (Renderer/Orchestrator)          │
│ (FrameBuffer)                    │
│ (PrivacyEngine [adblock-rust])   │
│ (AsciiEngine [chafa])            │
│ (DataStore [rusqlite, redb])     │
│                                  │
│ <--- (Mojo IPC Bridge) --->      │
│                                  │
└──────────────────────────────────┘

┌──────────────────────────────────┐
│ mongoose-core                    │
│ (Forked Chromium Headless Shell) │
│ (Blink/V8 Engine)                │
│ (MongooseTextCaptureDevice)      │
│ (MongooseURLLoaderInterceptor)   │
└──────────────────────────────────┘
```

### C. The fathyb/carbonyl Fork Strategy

The fathyb/carbonyl repository will be forked to create the mongoose project.

- The C++ runtime directory will be forked and renamed **mongoose-core**
- The Rust libcarbonyl directory will be forked and renamed **libmongoose**
- A new Rust binary crate, **mongoose-tui**, will be created at the root. This replaces the original carbonyl executable
- mongoose-tui will be the main application executable and will list libmongoose as a [dependency]
- The Chromium build scripts (gn, ninja) will be adapted to link the libmongoose shared library and build the mongoose-tui binary as the primary build artifact

## II. Feature Specification (Requested & Innovated)

### Core (from User orpheus497)

- ✓ **Modern Web Rendering**: Full support for HTML5, CSS3, JavaScript, video, and WebGL by leveraging the embedded Chromium engine
- ✓ **TTY-Native Execution**: Runs solely within a TTY terminal, requiring no X server or graphical environment
- ✓ **Privacy Controls**: Brave-like ad and tracker blocking functionality
- ✓ **High-Fidelity ASCII Imagery**: "Perfect duplication level" ASCII art for images

### Innovated (by Systems Architect)

- ✓ **Hybrid Rendering Engine**: A toggleable mode (`Ctrl+A`) to switch between:
  - a) Block Mode: Fast, 60fps Unicode half-block (▄) rendering for all content, including video
  - b) ASCII Mode: High-fidelity chafa-based rendering for static `<img>` tags, composited onto the Block Mode frame
- ✓ **Deep Network Interception**: Full adblock-rust engine integration, blocking network requests before they leave the browser core using the URLLoaderRequestInterceptor hook
- ✓ **Modern TUI Shell**: ratatui-based shell with multi-tab support, persistent URL bar, status line, and interactive menus
- ✓ **Persistent Data Store**: History, bookmarks, and cookie storage via rusqlite
- ✓ **Persistent Cache Store**: Ad-block filter lists and session data stored in redb
- ✓ **Command Palette**: A `Ctrl+Shift+P` style command palette for all browser functions (e.g., "Clear Cache," "Toggle ASCII Mode," "Add Bookmark")
- ✓ **Full Mouse Support**: Clickable links, focusable inputs, and scroll wheel support, as enabled by modern TTY emulators and crossterm
- ✓ **Configurable Settings**: Persistent user settings for Hybrid Mode, default filter lists, homepage, etc., stored in the rusqlite database

## III. Component: mongoose-core (C++ Chromium Fork)

### A. Build System

Forked from fathyb/carbonyl. The build will continue to use `gn` for configuration and `ninja` for compilation.

The root BUILD.gn file will be modified to define a `rust_shared_library` target (libmongoose) and a `rust_binary` target (mongoose-tui), ensuring the Rust TUI is the final executable.

### B. MongooseTextCaptureDevice (Skia Backend)

**Source**: Fork of Carbonyl's TextCaptureDevice

**Path**: `mongoose-core/ui/gfx/mongoose_text_capture_device.cc`

**Design**: This class implements the `skia::Device` API.

- `onDrawGlyphRunList`: Renders text glyphs. It converts SkGlyphID to Unichar (Unicode) and calls the DrawText Mojo method to send the text to libmongoose
- `drawRect`: Renders rectangles and other primitives. It converts these to half-blocks (▄) and calls the DrawBitmap Mojo method, sending the blocky "pixels" to libmongoose

**Innovation - drawBitmap**: This specific method will be modified to enable the Hybrid Renderer.

```cpp
// C++ Pseudocode for mongoose_text_capture_device.cc
#include "base/synchronization/lock.h"

// Global flag, thread-safe, controlled by Rust via Mojo
base::Lock g_hybrid_ascii_mode_lock;
bool g_hybrid_ascii_mode = false;

void MongooseTextCaptureDevice::drawBitmap(const SkBitmap& bitmap,...) {
  bool is_ascii_mode;
  {
    base::AutoLock lock(g_hybrid_ascii_mode_lock);
    is_ascii_mode = g_hybrid_ascii_mode;
  }

  if (is_ascii_mode) {
    // In ASCII mode, send the raw bitmap to Rust for chafa rendering
    // This calls the *new* Mojo method DrawImageBitmap
    mojo::SharedMemoryRegion region = ConvertBitmapToSharedMemory(bitmap);
    carbonyl_render_service_->DrawImageBitmap(std::move(region), bitmap.bounds());
  } else {
    // In Block mode, render it as blocks (Carbonyl's default)
    // This calls the *existing* Mojo method DrawBitmap
    DefaultDrawBitmapAsBlocks(bitmap);
  }
}
```

### C. MongooseURLLoaderInterceptor (Network Hook)

**Source**: New C++ class, as this does not exist in Carbonyl

**Path**: `mongoose-core/content/browser/mongoose_url_loader_interceptor.h` (and .cc)

**Design**: Implements the `content::URLLoaderRequestInterceptor` interface. This hook is installed into Chromium's ContentBrowserClient.

**MaybeCreateLoader**: This method is the core intercept. It will be implemented to perform a synchronous IPC call to libmongoose to determine if a request should be blocked.

```cpp
// C++ Pseudocode for mongoose_url_loader_interceptor.cc
void MongooseURLLoaderInterceptor::MaybeCreateLoader(
    const network::ResourceRequest& request,
    content::BrowserContext* browser_context,
    LoaderCallback callback,
    InterceptorCallback interceptor_callback) {

  // 1. Package request data
  carbonyl::mojom::NetworkRequestDataPtr data =
      carbonyl::mojom::NetworkRequestData::New();
  data->url = request.url;
  data->initiator = request.initiator.value_or(url::Origin());
  data->resource_type = request.resource_type;

  // 2. Get the Mojo service (must be on UI thread)
  mojo::Remote<carbonyl::mojom::CarbonylRenderService> service =
      GetRenderService();

  // 3. Make SYNCHRONOUS IPC call to Rust
  carbonyl::mojom::BlockResultPtr block_result;
  if (!service->CheckNetworkRequest(std::move(data), &block_result)) {
    // IPC failed, fail open (allow)
    std::move(callback).Run(nullptr);
    return;
  }

  // 4. Act on the result from libmongoose
  if (block_result->should_block) {
    // Block the request by running the callback with an empty loader
    std::move(callback).Run(
        network::URLLoaderCompletionStatus(net::ERR_BLOCKED_BY_CLIENT));
  } else {
    // Allow the request
    std::move(callback).Run(nullptr);
  }
}
```

This synchronous hook is critical. It halts the network stack for a few microseconds, allowing the adblock-rust engine in libmongoose to perform its high-speed in-memory check and return a definitive block/allow decision.

### D. Mojo IPC Bridge (mongoose.mojom)

**Source**: Fork of carbonyl.mojom

**Path**: `mongoose-core/mojo/mongoose.mojom`

This file defines the strict API contract between the C++ mongoose-core and the Rust libmongoose.

**Table 2: Mojo IPC API Contract (CarbonylRenderService interface)**

| Method | Direction | Sync/Async | Payload | Description |
|--------|-----------|------------|---------|-------------|
| DrawText | C++ -> Rust | Async | TextData | (Existing) Renders a list of text glyphs |
| DrawBitmap | C++ -> Rust | Async | SharedMemoryRegion, Bounds | (Existing) Renders a frame of Unicode blocks |
| SetTitle | C++ -> Rust | Async | String | (Existing) Set the TUI window title |
| OnKeyEvent | Rust -> C++ | Async | KeyEvent | (Existing) Send a key press to Blink |
| OnMouseDown | Rust -> C++ | Async | MouseEvent | (Existing) Send a mouse click to Blink |
| OnMouseMove | Rust -> C++ | Async | MouseEvent | (Existing) Send a mouse move to Blink |
| OnMouseUp | Rust -> C++ | Async | MouseEvent | (Existing) Send a mouse release to Blink |
| OnScroll | Rust -> C++ | Async | ScrollEvent | (Existing) Send a scroll wheel event to Blink |
| CheckNetworkRequest | C++ -> Rust | Sync | NetworkRequestData -> BlockResult | (Innovated) Asks libmongoose's PrivacyEngine if a network request should be blocked |
| DrawImageBitmap | C++ -> Rust | Async | SharedMemoryRegion, Bounds | (Innovated) Sends a raw image bitmap to libmongoose for chafa rendering (Hybrid ASCII Mode) |
| SetHybridAsciiMode | Rust -> C++ | Async | bool | (Innovated) Tells mongoose-core to enable/disable hybrid ASCII mode in the MongooseTextCaptureDevice |

## IV. Component: libmongoose (Rust Core Logic)

### A. Main Orchestrator (Renderer struct)

This struct is the main Rust entrypoint, implementing the `CarbonylRenderService` Mojo trait. It holds the state for all sub-services (PrivacyEngine, AsciiEngine, DataStore) within `Arc<Mutex<T>>` for thread-safe access.

- `fn DrawBitmap(...)`: Receives the block buffer, converts it to r3bl_ansi_color codes, and stores it in the FrameBuffer for the TUI to fetch
- `fn DrawImageBitmap(...)`: Receives the raw image, calls AsciiEngine::render(), and "stamps" the resulting chafa buffer onto the FrameBuffer
- `fn CheckNetworkRequest(...)`: Synchronously locks the PrivacyEngine and calls PrivacyEngine::check(), returning the BlockResult. This must be a non-blocking, in-memory-only operation to prevent deadlocking the Chromium UI thread

### B. Privacy Engine Service (PrivacyEngine struct)

**Crate**: adblock-rust

**State**: `engine: Arc<adblock::Engine>`

**Initialization**: On browser startup, libmongoose spawns a background thread to:

1. Fetch the user's enabled filter list URLs from DataStore (e.g., EasyList)
2. Check DataStore (the redb cache) for a fresh version of each list
3. If stale (e.g., > 24 hours old), download the new list, parse it, and save the binary representation to the redb cache
4. Load all rules from redb cache into `adblock::Engine::from_filter_set()`
5. Place the resulting Engine into the Arc for high-speed, thread-safe reads

**check(request: &NetworkRequestData) -> BlockResult (Sync)**: This function is called by the Mojo interceptor. It takes the NetworkRequestData from Mojo, converts it to `adblock::Request`, and calls `engine.check_network_urls()`. This is a pure, in-memory read, ensuring it meets the high-performance (microsecond) requirement.

### C. Hybrid ASCII Engine (AsciiEngine struct)

**Crate**: chafa and chafa-sys

**State**: `config: chafa::Config`

**Initialization**: Creates a `chafa::Config` object configured for:

- Symbols: Full range of Unicode glyphs (e.g., CHAFA_SYMBOL_MAP_ALL)
- Color: 24-bit Truecolor (CHAFA_COLOR_SPACE_RGB)
- Dither: Ordered dither

**render(bitmap: &[u8], width: u32, height: u32) -> CharBuffer**:

1. Creates a chafa::Canvas from the config
2. Draws the input bitmap (received from DrawImageBitmap Mojo call) onto the canvas
3. Exports the canvas to a UTF-8 string buffer, which includes all ANSI escape codes
4. Returns this buffer for compositing onto the main FrameBuffer

### D. Datastore Service (DataStore struct)

Manages a `rusqlite::Connection` and a `redb::Database` handle.

Provides a high-level API to the rest of libmongoose: `get_history()`, `add_bookmark()`, `get_cookie()`, `get_filter_list()`, `cache_filter_list()`, `get_session_tabs()`, `save_session_tabs()`.

## V. Component: mongoose-tui (Rust ratatui Shell)

### A. Main Event Loop (main.rs)

1. Initializes crossterm and enters the alternate screen
2. Initializes libmongoose, which in turn starts mongoose-core as a subprocess and establishes the Mojo IPC connection
3. Enters the main Terminal::draw loop
4. On every tick:
   - Polls for crossterm::event (key/mouse/resize)
   - Passes events to AppState::handle_input() for processing
   - AppState calls libmongoose methods as needed (e.g., OnKeyEvent, OnScroll, Navigate)
   - Calls Terminal::draw() with the ui::draw function

### B. Application State (AppState struct)

- `tabs: Vec<TabState { id: u32, title: String }>`
- `active_tab_index: usize`
- `input_mode: InputMode` (Enum: Normal, Insert (for URL bar), Command)
- `url_bar_content: String`
- `status_message: String`
- `command_palette: CommandPaletteState`
- `is_ascii_mode: bool`

### C. TUI Widgets (ui.rs module)

- `draw_tabs(frame: &mut Frame, state: &AppState)`: Renders `ratatui::widgets::Tabs`
- `draw_url_bar(frame: &mut Frame, state: &AppState)`: Renders `ratatui::widgets::Paragraph`, styled based on InputMode
- `draw_browser_pane(frame: &mut Frame, lib: &libmongoose)`: This is the core rendering widget
  - Fetches the latest frame: `let buffer = lib.get_current_frame_buffer()`
  - Creates a custom `ratatui::widgets::Widget` that directly blits this pre-rendered TTY buffer (which already contains all ANSI codes) into the frame's draw area
- `draw_command_palette(...)`: Renders a floating Paragraph (for input) and List (for suggestions) over the main UI

### D. Input & Keybinding Map

This defines the core user interaction model.

**Table 3: Keybinding Map**

| Key | Mode | Action | Component |
|-----|------|--------|-----------|
| Ctrl+q | Normal | Quit | mongoose-tui |
| Ctrl+t | Normal | Open New Tab | mongoose-tui |
| Ctrl+w | Normal | Close Current Tab | mongoose-tui |
| Ctrl+l | Normal | Focus URL Bar | mongoose-tui (sets Mode=Insert) |
| Enter | Insert (URL Bar) | Navigate to URL | mongoose-tui -> libmongoose::Navigate |
| Esc | Insert | Unfocus URL Bar | mongoose-tui (sets Mode=Normal) |
| Ctrl+Shift+P | Normal | Toggle Command Palette | mongoose-tui (sets Mode=Command) |
| j / k | Normal | Scroll Down / Up | mongoose-tui -> libmongoose::OnScroll |
| Ctrl+b | Normal | Add Bookmark | mongoose-tui -> libmongoose::add_bookmark() |
| Ctrl+a | Normal | Toggle Hybrid ASCII Mode | mongoose-tui -> libmongoose::toggle_ascii_mode() |

## VI. Data Models & Persistence

### A. rusqlite Schema (Path: $CONFIG_DIR/mongoose/mongoose.sqlite)

**Table 4: history**

- `id` (INTEGER PRIMARY KEY)
- `url` (TEXT NOT NULL)
- `title` (TEXT)
- `visit_time` (INTEGER NOT NULL, Unix timestamp)
- `visit_count` (INTEGER NOT NULL DEFAULT 1)

**Table 5: bookmarks**

- `id` (INTEGER PRIMARY KEY)
- `url` (TEXT NOT NULL)
- `title` (TEXT)
- `add_time` (INTEGER NOT NULL, Unix timestamp)

**Table 6: site_permissions**

- `origin` (TEXT PRIMARY KEY)
- `javascript` (INTEGER NOT NULL DEFAULT 1)
- `cookies` (INTEGER NOT NULL DEFAULT 1)
- `images` (INTEGER NOT NULL DEFAULT 1)

**Table 7: settings**

- `key` (TEXT PRIMARY KEY)
- `value` (TEXT)

### B. redb Schema (Path: $CONFIG_DIR/mongoose/cache.redb)

**Table 8: adblock_lists** (Table)

- key: (String) Filter list name (e.g., "easylist")
- value: (String) Filter list URL (e.g., "https://easylist.to/easylist/easylist.txt")

**Table 9: adblock_cache** (Table)

- key: (String) Filter list URL
- value: (Vec<u8>, i64) (Tuple of (bincode-serialized list_data, last_fetched_timestamp))

**Table 10: session** (Table)

- key: (String) "open_tabs"
- value: Vec<u8> (Bincode/JSON serialized Vec<String> of URLs)

## VII. User Flow Diagrams (Text-based)

### Flow 1: User Loads a Web Page (with Ad-Blocking)

1. **User**: Presses `Ctrl+l`. mongoose-tui enters Insert mode
2. **User**: Types "example.com", presses Enter
3. **TUI**: Calls `libmongoose.navigate("example.com")`
4. **Lib**: Calls Mojo `OnKeyEvent` to send navigation to Chromium
5. **Core**: Chromium's network service fires a request for "example.com"
6. **Core**: MongooseURLLoaderInterceptor intercepts the request
7. **Core**: Makes a Sync Mojo call `CheckNetworkRequest("example.com")` to libmongoose
8. **Lib**: PrivacyEngine checks "example.com". Result: Allow
9. **Lib**: Returns BlockResult::Allow
10. **Core**: Request proceeds. HTML is downloaded
11. **Core**: Chromium parses HTML, finds `<img src="ad-server.com/ad.png">`
12. **Core**: MongooseURLLoaderInterceptor intercepts
13. **Core**: Makes Sync Mojo call `CheckNetworkRequest("ad-server.com/ad.png")`
14. **Lib**: PrivacyEngine checks. Rules match "ad-server.com". Result: Block
15. **Lib**: Returns BlockResult::Block
16. **Core**: InterceptorCallback is run with `net::ERR_BLOCKED_BY_CLIENT`. The request for "ad.png" is terminated
17. **Core**: Blink/Skia render the page (without the ad) to the MongooseTextCaptureDevice
18. **Core**: `DrawBitmap` Mojo call is made to libmongoose with the frame data
19. **Lib**: Converts bitmap to TTY buffer, stores in FrameBuffer
20. **TUI**: `draw_browser_pane()` fetches this buffer and renders it to the screen

### Flow 2: User Toggles Hybrid ASCII-Art Mode

1. **User**: Presses `Ctrl+a` (innovated keybinding)
2. **TUI**: Calls `libmongoose.toggle_ascii_mode()`
3. **Lib**: Flips `is_ascii_mode` flag to true
4. **Lib**: Calls Mojo `SetHybridAsciiMode(true)`
5. **Core**: `g_hybrid_ascii_mode` global in `mongoose_text_capture_device.cc` is set to true
6. **User**: Loads a page with `<img src="image.png">`
7. **Core**: Ad-blocking (Flow 1) completes
8. **Core**: Skia calls `MongooseTextCaptureDevice::drawBitmap` for "image.png"
9. **Core**: The device checks `g_hybrid_ascii_mode`. It's true
10. **Core**: It skips block rendering and instead calls the new Mojo method `DrawImageBitmap` with the raw image data
11. **Lib**: The DrawImageBitmap handler receives the data
12. **Lib**: Calls `AsciiEngine::render()` on the bitmap
13. **Lib**: chafa generates a high-fidelity Unicode string
14. **Lib**: This string is "stamped" onto the FrameBuffer at the correct coordinates
15. **TUI**: Renders the frame, showing perfect chafa art for the image and Carbonyl-style blocks for all other page content

## VIII. Project File Blueprint

```
mongoose/
├── .dev-docs/                  (All AI-generated design docs)
│   ├── 01-Research_and_Analysis.md
│   ├── 02-Systems_Design_Document.md
│   └── 03-Final_Summary.md
├── mongoose-core/              (The C++ Chromium fork, based on carbonyl)
│   ├── BUILD.gn                (Main build file)
│   ├── content/
│   │   └── browser/
│   │       ├── mongoose_url_loader_interceptor.cc
│   │       └── mongoose_url_loader_interceptor.h
│   ├── services/network/       (Network service patches)
│   ├── ui/gfx/                 (Skia/rendering patches)
│   │   ├── mongoose_text_capture_device.cc
│   │   └── mongoose_text_capture_device.h
│   └── mojo/
│       └── mongoose.mojom      (Renamed carbonyl.mojom w/ new methods)
├── mongoose-tui/               (Rust binary crate - the main executable)
│   ├── Cargo.toml              (deps: ratatui, crossterm, libmongoose)
│   └── src/
│       ├── main.rs             (Main event loop, TUI initialization)
│       ├── app.rs              (AppState struct, state logic)
│       ├── ui.rs               (Ratatui widget drawing functions)
│       ├── input.rs            (Input handling, keybinding map)
│       └── command.rs          (Command Palette logic)
├── libmongoose/                (Rust library crate - the core logic)
│   ├── Cargo.toml              (deps: adblock-rust, chafa-sys, rusqlite, redb, r3bl_ansi_color)
│   └── src/
│       ├── lib.rs              (Main public API, Renderer struct, FrameBuffer)
│       ├── mojo_impl.rs        (Implementation of the mongoose.mojom service)
│       ├── privacy.rs          (PrivacyEngine struct, adblock-rust wrapper)
│       ├── render.rs           (AsciiEngine struct, chafa wrapper, buffer logic)
│       └── data.rs             (DataStore service, rusqlite/redb logic)
├── .gitignore
├── CHANGELOG.md
└── README.md
```
