# Mongoose: Research & Analysis Report

## I. Executive Summary & Core Mandate Analysis

### A. Project Objective

Analysis of the mongoose project concept, as defined by originator orpheus497, is complete. The objective is to architect a terminal-native, high-fidelity, privacy-first modern web browser. This application must function entirely within a TTY (teletype) terminal environment while providing features and rendering parity commonly associated with modern graphical browsers.

### B. Core Mandates

A breakdown of the project's non-negotiable mandates reveals a set of complex, interconnected, and partially conflicting requirements:

1. **Modern Web Parity**: The system must "run the most modern websites." This is a definitive requirement for a full JavaScript (JS) engine and rendering stack, including CSS3, HTML5, and potentially WebGL. This mandate immediately invalidates traditional terminal (TTY) browsers, which lack the requisite JS support.

2. **TTY-Native**: The application must rely "solely on the terminal." It cannot be a graphical application mirrored to the terminal; it must be a native TTY citizen.

3. **High-Fidelity ASCII**: The browser must display imagery by "rendering it as ascii art on a perfect duplication level." This implies a sophisticated image-to-character rendering pipeline that goes beyond simple block art.

4. **Privacy-Centric**: The browser must possess "features and capabilities much like the brave browser regarding privacy control." This mandates an integrated, high-performance engine for blocking advertisements and trackers.

5. **FOSS-First**: The architecture must leverage existing FOSS (Free and Open-Source Software) and not "build everything from the ground up."

### C. Executive Decision

The extensive domain analysis concludes that a single, viable architectural path exists that satisfies all five mandates. **The mongoose architecture must be a direct fork and significant enhancement of the fathyb/carbonyl project.**

The primary architectural decision point was a comparison between the two dominant modern TTY browser models. The Browsh "headless proxy" model was rejected. While it outsources web standards to Firefox, it is a complex, high-latency, and resource-intensive system, described as "super slow" and consuming up to "50x more CPU power" than alternatives.

The Carbonyl "native TTY renderer" model is the superior choice. It is a direct fork of Chromium that replaces the graphical Skia backend with a high-performance TTY renderer. It is not a proxy; it is the browser. This architecture natively provides 60fps performance, 0% idle CPU usage, and full support for all modern web APIs, including WebGL and video.

### D. Core Innovation

The mongoose project will not be a simple fork. The final design is a **Rust-centric, hybrid-rendering browser**.

The architecture will leverage Carbonyl's C++/Rust structure. Carbonyl is composed of a C++ Chromium runtime and a Rust shared library (libcarbonyl) that handles all TTY rendering and input. This provides a perfect "seam" for integration.

The mongoose architecture will:

- Fork the Carbonyl C++ runtime to create **mongoose-core**
- Fork and rename libcarbonyl to **libmongoose**, which will become the "brain" of the browser
- Create a new **mongoose-tui** Rust binary that replaces the default Carbonyl executable. This TUI will be built with ratatui and will provide the browser's "chrome" (tabs, URL bar, menus)
- Integrate the **adblock-rust** engine directly into libmongoose to fulfill the privacy mandate
- Integrate the **chafa** FOSS library into libmongoose to create a novel Hybrid Rendering Engine, satisfying the ASCII art mandate without sacrificing performance

## II. Domain Analysis: The Architectural "Great Fork"

### A. The Problem: The Unusable Modern Web in TTY

The project's necessity is defined by a fundamental gap: the modern web is unusable in traditional terminal browsers. This is not a matter of aesthetics but of function.

Traditional TTY browsers, such as lynx and w3m, were designed for a web of static hypertext documents. Their primary limitation is the complete absence of a JavaScript interpreter. As the modern web is "script-dependent", most websites will not load content, allow logins, or function in any meaningful way.

The user's mandate for "modern websites" is therefore a direct, non-negotiable dependency on a full-fledged JavaScript rendering engine (like Chromium's Blink/V8 or Firefox's Gecko). The "poor to non-existent" JS support in TTY-native browsers is a project-killing roadblock, forcing the architecture to incorporate a complete, modern browser engine.

### B. Architectural Model 1: The "Headless Proxy" (e.g., Browsh)

This model is a clever workaround. It does not try to be a browser in the terminal; it acts as a TTY client for a full, graphical browser running elsewhere.

**Architecture**: The system Browsh uses consists of a minimal Go-based CLI client and a browser webextension.

**Components**:

- **Go CLI Client**: The TTY application the user interacts with
- **Headless Firefox**: The Go client spawns a full Firefox (version 57+) process in headless mode
- **WebExtension**: The client uses a remote debugging connection to install a custom webextension. This extension injects scripts, forces the page content onto a rigid grid using custom CSS, and opens a websocket back to the Go client
- **Render Pipeline**: The extension "screenshots" the rendered page, scales this screenshot to the terminal's dimensions, and converts the pixel values into terminal color escape codes. Text is then layered on top

**Analysis**:

*Pros*: This model successfully outsources the problem of web standards compliance ("Someone Else's Problem"). It also allows the use of standard Firefox extensions, such as adblockers.

*Cons*: This architecture is fundamentally a "Rube Goldberg" machine of inter-process communication (CLI <-> Firefox Process <-> Extension <-> WebSocket <-> CLI). This complexity results in severe performance and resource penalties. It is described as "super slow" and requires a full Firefox instance to be running, leading to high CPU and RAM usage—reportedly "50x more CPU power" than the Carbonyl model. This fails any reasonable standard of usability for a primary browser.

### C. Architectural Model 2: The "Native TTY Renderer" (e.g., fathyb/carbonyl)

This model represents a fundamental architectural breakthrough. Instead of proxying a graphical browser, it makes the browser engine render natively to a non-graphical backend (the TTY).

**Architecture**: This model *is* the browser. Carbonyl is a direct fork of the Chromium open-source project.

**Components**:

- **carbonyl (C++)**: A modified headless_shell (the C++ runtime)
- **libcarbonyl (Rust)**: A shared Rust library dynamically loaded by the C++ shell. This Rust library contains all logic for TTY output, input parsing, and color conversion

**Render Pipeline**: Carbonyl's core innovation is a custom Skia device. Skia is Chromium's 2D graphics library. Carbonyl provides a new C++ backend for Skia called `TextCaptureDevice`.

When Blink (Chromium's layout engine) issues draw calls (e.g., "draw text here," "draw this rectangle"), they are sent to this custom device.

The TextCaptureDevice does not draw to a GPU bitmap. Instead, it converts these draw calls directly into TTY escape sequences.

It renders "pixels" using the Unicode lower half-block element (▄). By setting the foreground and background colors of a single character cell, it achieves two "pixels" of color, effectively doubling the vertical resolution of the terminal.

All this data is sent via Mojo (Chromium's IPC) to the libcarbonyl Rust library, which formats the final TTY output stream.

**Analysis**: This architecture achieves all mandates simultaneously. It supports all web APIs (JS, WebGL, video) because it is Chromium. It is TTY-native, requiring no window server. Most importantly, it is extraordinarily performant: 60 FPS, <1 second startup, and 0% idle CPU usage.

### D. Rationale & Decision for mongoose

**Decision**: The mongoose architecture must be a fork of fathyb/carbonyl.

The user's mandate is for a usable, production-ready browser. The Browsh model's "super slow" performance and extreme resource consumption render it a non-viable foundation for a daily-driver application. Carbonyl's 60fps/0%-idle performance is production-grade and definitively superior.

The Carbonyl C++/Rust hybrid architecture is the project's single greatest FOSS accelerator. The C++ fork (runtime) is a minimal set of hooks into Chromium's rendering and input. The logic (rendering, input, color management) is already isolated in a Rust shared library (libcarbonyl).

This C++/Rust architecture creates a perfect "seam" for all of the mongoose project's innovated features:

- **Privacy**: The adblock-rust engine can be integrated directly into the libmongoose Rust crate
- **ASCII Art**: The chafa library can be integrated directly into the libmongoose Rust crate
- **TUI**: The existing carbonyl executable can be replaced with a ratatui TUI application that links libmongoose as its core rendering and logic engine

This provides a clear, robust, and Rust-native path forward, fully aligning with the FOSS-first mandate.

## III. FOSS Technology Domain Analysis (Component Selection)

### A. TUI Application Shell (The "Chrome")

**Requirement**: A library to build the application's "chrome"—the frame, tabs, URL bar, and status line that will host the libmongoose rendering widget.

**Candidates**:

- **ratatui (Rust)**: The active, community-maintained fork of tui-rs. It is an immediate-mode rendering library
- **Bubbletea (Go)**: A popular Go library based on The Elm Architecture. This is not Rust and would require a complex IPC bridge to the libmongoose core, adding latency
- **Textual (Python)**: A feature-rich, async, retained-mode TUI framework. This is also not Rust

**Decision**: **ratatui**

**Rationale**: The choice of ratatui is strategic. As an immediate-mode library, it "draws what you tell it, right now" at every frame. This is architecturally perfect for hosting a 60fps widget. A retained-mode TUI (like Textual) would try to manage its own stateful widget tree, which would conflict with our high-fps, externally-rendered browser pane. With ratatui, the main application loop is simple and clean:

1. Draw the ratatui widgets (tabs, URL bar)
2. Get the pre-rendered TTY buffer from libmongoose
3. "Blit" that buffer into the ratatui frame

This model is flicker-free, fast (modern terminals are GPU accelerated and can handle 60fps), and architecturally sound.

### B. Privacy Engine (The "Shield")

**Requirement**: "Brave-like privacy control," including robust ad and tracker blocking.

**Candidates**:

- **adblock-rust (Rust)**: The actual FOSS engine powering Brave's native adblocker
- **adblock (Rust)**: A similar FOSS engine, also inspired by uBlock Origin
- **DNS-based (e.g., pihole)**: A DNS-level blocker. This is insufficient as it cannot perform cosmetic filtering (hiding page elements) or context-aware request blocking

**Decision**: **adblock-rust**

**Rationale**: The user's "Brave-like" requirement is a feature-set mandate. adblock-rust provides this entire feature set out-of-the-box: network blocking, cosmetic filtering (hiding CSS elements), resource replacement (scriptlet injection), and full support for EasyList/uBlock Origin syntax. It is high-performance and, being a Rust crate, integrates natively into the libmongoose core.

### C. High-Fidelity ASCII Art Rendering (The "Lens")

**Requirement**: "perfect duplication level" ASCII art for imagery. This is a specific mandate for high fidelity, not the default block rendering of Carbonyl.

**Candidates**:

- **chafa (C)**: The most advanced FOSS TTY image renderer. It uses a wide range of Unicode glyphs, not just ASCII, for high fidelity. Supports truecolor, sixels, and the Kitty graphics protocol. It has existing, safe Rust bindings (chafa-sys, chafa)
- **libcaca (C)**: The "classic" TTY graphics library. Research describes chafa as "far far better than caca," indicating chafa is the superior choice for fidelity. It also has Rust bindings
- **symisc/ascii_art (C)**: A single-file C library. Uses a decision-tree model. It is simpler but far less feature-rich and lower-fidelity than chafa
- **Pure-Rust crates (image_ascii, rascii_art)**: Good, but chafa is the established, battle-tested, state-of-the-art standard for fidelity

**Decision**: **chafa**, consumed via the chafa-sys Rust bindings

**Rationale**: The user's mandate for "perfect duplication" and "modern websites" (which implies video) are in direct conflict. High-fidelity ASCII art generation is computationally expensive and too slow for 60fps video. Carbonyl's default half-block (▄) rendering is fast enough for 60fps.

**A Hybrid Rendering Engine is the only solution**:

- Carbonyl's TextCaptureDevice already renders all content (text, CSS, video) using the fast half-block method. This will be mongoose's "Default Mode," providing 60fps
- An innovated "ASCII Art Mode" will be created
- When this mode is toggled on, the C++ fork will be modified to intercept SkBitmap (image) draw calls
- Instead of rendering them as blocks, it will pipe this raw bitmap data over the Mojo IPC bridge to a new handler in libmongoose
- libmongoose will use chafa to render this bitmap into a high-fidelity Unicode character buffer
- This buffer will then be composited onto the main frame

This hybrid approach provides the best of both worlds: 60fps block rendering for video and page structure, and "perfect duplication" chafa art for static images, fulfilling both mandates without compromise.

### D. Embedded Datastore (The "Memory")

**Requirement**: A persistent, embedded datastore for browser history, bookmarks, cookies, site permissions, and the ad-block filter list cache.

**Candidates (Relational/SQL)**: rusqlite. The underlying SQLite library is public domain.

**Candidates (Key-Value)**: rocksdb, sled, redb

**Decision**: A **hybrid datastore model**:

- **rusqlite** for structured, relational data (History, Bookmarks, Cookies, Site Permissions)
- **redb** for blob/cache data (Ad-block filter cache, Session state)

**Rationale**: This "right tool for the job" approach is optimal.

- **rusqlite**: For data that needs to be queried relationally (e.g., "Find all history items from 3 days ago matching 'X'"), rusqlite is the unassailable, mature FOSS standard
- **redb**: For the KV store, research indicates sled is likely "abandoned", and rocksdb (while fast) adds significant C++ build complexity. redb has emerged as the modern, maintained, pure-Rust, B-Tree-based KV store. It is perfect for storing large blobs (ad-block lists) and fast-access keys (session state). Using pure-Rust redb avoids adding another C++ dependency to a project already forking Chromium

## IV. Final FOSS Stack Selection & Rationale

The following table represents the definitive, "Zero Placeholder" list of all primary FOSS technologies to be used in the mongoose project, as determined by this analysis.

**Table 1: Final FOSS Dependency Stack**

| Component Type | Selected FOSS Project | License | Rationale for Selection |
|----------------|----------------------|---------|------------------------|
| Core Browser Engine | Chromium (via fathyb/carbonyl fork) | BSD-3-Clause | The only FOSS project providing a high-performance (60fps), TTY-native renderer for the entire modern web (JS, WebGL, video) |
| TUI Application Shell | ratatui | MIT | Actively maintained Rust TUI library. Its immediate-mode rendering model is architecturally ideal for hosting a high-fps, externally-rendered widget without state conflicts |
| Privacy Engine | adblock-rust | MPL-2.0 | The literal FOSS engine from Brave. Natively provides the user's "Brave-like" mandate, including network/cosmetic/scriptlet filtering |
| ASCII Art Engine | chafa | LGPL-3.0-or-later | The state-of-the-art, high-fidelity FOSS image-to-TTY converter. Required for the "perfect duplication" mandate. Used via chafa-sys Rust bindings |
| Relational Database | rusqlite | MIT | The industry-standard, mature Rust wrapper for SQLite. Perfect for structured, queryable data like history and bookmarks. SQLite core is Public Domain |
| Key-Value Store | redb | MIT | A modern, pure-Rust, high-performance, and maintained (unlike sled) KV store. Ideal for caches and blobs, avoiding the C++ build complexity of rocksdb |
| ANSI/Color Control | r3bl_ansi_color | Apache-2.0 | A pure-Rust library for robust detection and generation of 24-bit truecolor and ANSI 256-color, essential for the TTY rendering pipeline |
