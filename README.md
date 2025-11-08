# mongoose

**Project Designer & Originator**: orpheus497

mongoose is a modern, privacy-first web browser that runs entirely within your terminal.

It is designed to provide a high-fidelity browsing experience for the modern web (including JavaScript, CSS, and video) without requiring a graphical environment. It combines the full rendering power of the Chromium engine with a TTY-native interface, advanced privacy controls, and a unique hybrid ASCII-art rendering engine.

## Core Features

### Modern Web, In-Terminal
Renders all modern web content—including HTML5, CSS3, JavaScript, and video—by leveraging a modified Chromium backend.

### Brave-Like Privacy
Integrates a high-performance ad and tracker blocking engine (adblock-rust, the same engine used by Brave) to block unwanted requests before they are ever made.

### Hybrid Rendering Engine
Features a unique, toggleable rendering system:
- **Block Mode**: A high-performance 60fps renderer that uses Unicode blocks to represent graphics and video, ideal for smooth browsing
- **ASCII Mode**: A high-fidelity mode that uses the chafa library to render static images as "perfect duplication level" ASCII/Unicode art, while retaining block rendering for all other content

### FOSS-First
Built entirely on a foundation of best-in-class FOSS components.

## FOSS Attribution

This project stands on the shoulders of giants. mongoose is made possible by integrating and modifying the following core FOSS projects.

| Component Type | Selected FOSS Project | License |
|----------------|----------------------|---------|
| Core Browser Engine | [Chromium](https://www.chromium.org/) (via [fathyb/carbonyl](https://github.com/fathyb/carbonyl) fork) | BSD-3-Clause |
| TUI Application Shell | [ratatui](https://github.com/ratatui-org/ratatui) | MIT |
| Privacy Engine | [adblock-rust](https://github.com/brave/adblock-rust) | MPL-2.0 |
| ASCII Art Engine | [chafa](https://hpjansson.org/chafa/) | LGPL-3.0-or-later |
| Relational Database | [rusqlite](https://github.com/rusqlite/rusqlite) | MIT |
| Key-Value Store | [redb](https://github.com/cberner/redb) | MIT |
| ANSI/Color Control | [r3bl_ansi_color](https://github.com/r3bl-org/r3bl-open-core) | Apache-2.0 |

## Architecture

mongoose is a **Rust-centric, hybrid-rendering browser** composed of three primary components:

- **mongoose-core (C++)**: A forked Chromium headless shell with custom rendering and network interception hooks
- **libmongoose (Rust)**: The core logic library containing the privacy engine, ASCII art engine, and data stores
- **mongoose-tui (Rust)**: A ratatui-based TUI application providing the browser interface (tabs, URL bar, command palette)

For detailed architectural information, see the design documents in `.dev-docs/` (if available in the development repository).

## Status

🚧 **This project is currently in the design and early development phase.** 🚧

The architecture has been fully specified and is ready for implementation. See the `.dev-docs/` directory (in the development repository) for comprehensive design documentation.

## Building (Future)

Detailed build instructions will be provided once the implementation reaches a stable state.

The build process will involve:
1. Setting up the Chromium build environment (depot_tools, gn, ninja)
2. Building the mongoose-core C++ runtime
3. Building the libmongoose Rust library
4. Building the mongoose-tui Rust executable

## Usage (Future)

Once implemented, mongoose will support:

### Basic Navigation
```bash
# Launch mongoose and navigate to a URL
mongoose https://example.com
```

### Keybindings
- `Ctrl+L`: Focus URL bar
- `Ctrl+T`: Open new tab
- `Ctrl+W`: Close current tab
- `Ctrl+A`: Toggle hybrid ASCII art mode
- `Ctrl+Shift+P`: Open command palette
- `Ctrl+B`: Add bookmark
- `j/k`: Scroll down/up
- `Ctrl+Q`: Quit

### Command Palette
Press `Ctrl+Shift+P` to access the command palette for advanced functions:
- Clear cache
- Manage bookmarks
- Configure privacy settings
- Toggle ASCII mode
- And more...

## Privacy Features

mongoose includes integrated ad and tracker blocking powered by adblock-rust:

- **Network-level blocking**: Requests to known ad/tracker domains are blocked before they leave your browser
- **Cosmetic filtering**: Page elements are hidden using CSS rules
- **Scriptlet injection**: Advanced blocking of inline ads and tracking scripts
- **EasyList compatibility**: Supports all major filter list formats (EasyList, uBlock Origin, etc.)

## Contributing (Future)

Contributions will be welcome once the project reaches a stable implementation state.

mongoose is split into three main components:
- **mongoose-core** (C++): Modified Chromium headless shell (long build time)
- **libmongoose** (Rust): Core logic library (fast build)
- **mongoose-tui** (Rust): TUI application (fast build)

For most feature development, you'll only need to modify the Rust components, which compile in seconds.

## License

This project inherits licenses from its components:
- Chromium components: BSD-3-Clause
- Rust components: MIT (unless otherwise specified)
- See individual component licenses in the FOSS Attribution table above

## Acknowledgments

Special thanks to:
- The [Chromium](https://www.chromium.org/) team for the incredible browser engine
- [fathyb](https://github.com/fathyb) for the groundbreaking [Carbonyl](https://github.com/fathyb/carbonyl) project
- The [Brave](https://brave.com/) team for open-sourcing [adblock-rust](https://github.com/brave/adblock-rust)
- All the FOSS projects listed in the attribution table

---

**Status**: Design Complete, Implementation In Progress
