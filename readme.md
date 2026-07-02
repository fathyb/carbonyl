<table align="center">
  <tbody>
    <tr>
      <td>
        <p></p>
        <pre>
   O    O
    \  /
O —— Cr —— O
    /  \
   O    O</pre>
      </td>
      <td><h1>Carbonyl</h1></td>
    </tr>
  </tbody>
</table>

Carbonyl is a Chromium based browser built to run in a terminal. [Read the blog post](https://fathy.fr/carbonyl).

It supports pretty much all Web APIs including WebGL, WebGPU, audio and video playback, animations, etc..

It's snappy, starts in less than a second, runs at 60 FPS, and idles at 0% CPU usage. It does not require a window server (i.e. works in a safe-mode console), and even runs through SSH.

Carbonyl originally started as [`html2svg`](https://github.com/fathyb/html2svg) and is now the runtime behind it.

## Usage

> Carbonyl on Linux without Docker requires the same dependencies as Chromium.

### Docker

```shell
$ docker run --rm -ti fathyb/carbonyl https://youtube.com
```

### npm

```console
$ npm install --global carbonyl
$ carbonyl https://github.com
```

### Binaries

- [macOS amd64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.macos-amd64.zip)
- [macOS arm64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.macos-arm64.zip)
- [Linux amd64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.linux-amd64.zip)
- [Linux arm64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.linux-arm64.zip)

## Demo

<table>
  <tbody>
    <tr>
      <td>
        <video src="https://user-images.githubusercontent.com/5746414/213682926-f1cc2de7-a38c-4125-9257-92faecfc7e24.mp4">
      </td>
      <td>
        <video src="https://user-images.githubusercontent.com/5746414/213682913-398d3d11-1af8-4ae6-a0cd-a7f878efd88b.mp4">
      </td>
    </tr>
    <tr>
      <td colspan="2">
        <video src="https://user-images.githubusercontent.com/5746414/213682918-d6396a4f-ee23-431d-828e-4ad6a00e690e.mp4">
      </td>
    </tr>
  </tbody>
</table>

## Keyboard navigation (vimium-style)

Carbonyl ships with an opt-in keyboard navigation layer modelled on
[Vimium](https://github.com/philc/vimium). Enable it with `--vim` (or
`CARBONYL_ENV_VIM=1`); it is off by default so every key reaches the page as
usual. When enabled and the URL bar is unfocused, the following keys are
handled by Carbonyl itself rather than forwarded to the page; press `i`
(Insert mode) before typing into page inputs. Arrows, Enter and Tab always
pass through.

Note: the find highlights, hint labels and status bar are drawn as terminal
cells, so they are not visible under `--graphics`; the scrolling and history
keys still work there.

### Scrolling

| Key  | Action                       |
| ---- | ---------------------------- |
| `j`  | Scroll down one line         |
| `k`  | Scroll up one line           |
| `d`  | Scroll down half a page      |
| `u`  | Scroll up half a page        |
| `gg` | Scroll to the top            |
| `G`  | Scroll to the bottom         |
| `h`  | Send a Left arrow to the page (useful for carousels / horizontal scroll containers) |
| `l`  | Send a Right arrow to the page |

### History

| Key | Action       |
| --- | ------------ |
| `H` | Go back      |
| `L` | Go forward   |
| `r` | Reload page  |

### Find on page

| Key   | Action                                       |
| ----- | -------------------------------------------- |
| `/`   | Open the find prompt (status bar at bottom)  |
| Enter | Confirm the query and highlight matches      |
| `n`   | Cycle to the next match                      |
| `N`   | Cycle to the previous match                  |
| `Esc` | Clear the active query and highlights        |

Find searches the currently visible viewport only, ASCII case-insensitive.

### Link hints

| Key   | Action                                                                |
| ----- | --------------------------------------------------------------------- |
| `f`   | Show hint labels on the candidate clickables in the current viewport  |
| `Esc` | Cancel hint mode                                                      |

Type the label letters shown next to a target to send a synthetic click at that
position. Single-letter labels are used when there are 26 or fewer candidates;
longer labels appear otherwise. The status bar at the bottom shows how many
candidates were found and what you have typed so far.

Because Carbonyl does not currently expose the DOM to the Rust side, link
candidates are detected from rendered text (short isolated runs, or runs whose
colour differs from the dominant body colour). The label set is therefore a
superset of the real anchors on the page: clicking a label that happens to land
on plain text is harmless and simply does nothing.

### Modes

| Mode    | How to enter                                            | How to leave |
| ------- | ------------------------------------------------------- | ------------ |
| Normal  | Default whenever the URL bar is unfocused                | n/a          |
| Insert  | `i`. Keys are passed through to the page (typing, etc.) | `Esc`        |
| Find    | `/`                                                     | `Esc` or `Enter` |
| Hint    | `f`                                                     | `Esc`, type a unique label, or type a prefix with no remaining matches |

`Esc` in Normal mode also clears any leftover find highlights and pending
prefixes (e.g. a half-typed `g`).

## Known issues

- Fullscreen mode not supported yet

## Comparisons

### Lynx

Lynx is the original terminal web browser, and the oldest one still maintained.

#### Pros

- When it understands a page, Lynx has the best layout, fully optimized for the terminal

#### Cons

> Some might sound like pluses, but Browsh and Carbonyl let you disable most of those if you'd like

- Does not support a lot of modern web standards
- Cannot run JavaScript/WebAssembly
- Cannot view or play media (audio, video, DOOM)

### Browsh

Browsh is the original "normal browser into a terminal" project. It starts Firefox in headless mode and connects to it through an automation protocol.

#### Pro

- It's easier to update the underlying browser: just update Firefox
- This makes development easier: just install Firefox and compile the Go code in a few seconds
- As of today, Browsh supports extensions while Carbonyl doesn't, although it's on our roadmap

#### Cons

- It runs slower and requires more resources than Carbonyl. 50x more CPU power is needed for the same content in average, that's because Carbonyl does not downscale or copy the window framebuffer, it natively renders to the terminal resolution.
- It uses custom stylesheets to fix the layout, which is less reliable than Carbonyl's changes to its HTML engine (Blink).

## Operating System Support

As far as tested, the operating systems under are supported:

- Linux (Debian, Ubuntu and Arch tested)
- MacOS
- Windows 11 and WSL

## Contributing

Carbonyl is split in two parts: the "core" which is built into a shared library (`libcarbonyl`), and the "runtime" which dynamically loads the core (`carbonyl` executable).

The core is written in Rust and takes a few seconds to build from scratch. The runtime is a modified version of the Chromium headless shell and takes more than an hour to build from scratch.

If you're just making changes to the Rust code, build `libcarbonyl` and replace it in a release version of Carbonyl.

### Core

```console
$ cargo build
```

### Runtime

Few notes:

- Building the runtime is almost the same as building Chromium with extra steps to patch and bundle the Rust library. Scripts in the `scripts/` directory are simple wrappers around `gn`, `ninja`, etc..
- Building Chromium for arm64 on Linux requires an amd64 processor
- Carbonyl is only tested on Linux and macOS, other platforms likely require code changes to Chromium
- Chromium is huge and takes a long time to build, making your computer mostly unresponsive. An 8-core CPU such as an M1 Max or an i9 9900k with 10 Gbps fiber takes around ~1 hour to fetch and build. It requires around 100 GB of disk space.

#### Fetch

> Fetch Chromium's code.

```console
$ ./scripts/gclient.sh sync
```

#### Apply patches

> Any changes made to Chromium will be reverted, make sure to save any changes you made.

```console
$ ./scripts/patches.sh apply
```

#### Configure

```console
$ ./scripts/gn.sh args out/Default
```

> `Default` is the target name, you can use multiple ones and pick any name you'd like, i.e.:
>
> ```console
> $ ./scripts/gn.sh args out/release
> $ ./scripts/gn.sh args out/debug
> # or if you'd like to build a multi-platform image
> $ ./scripts/gn.sh args out/arm64
> $ ./scripts/gn.sh args out/amd64
> ```

When prompted, enter the following arguments:

```gn
import("//carbonyl/src/browser/args.gn")

# uncomment this to build for arm64
# target_cpu = "arm64"

# comment this to disable ccache
cc_wrapper = "env CCACHE_SLOPPINESS=time_macros ccache"

# comment this for a debug build
is_debug = false
symbol_level = 0
is_official_build = true
```

#### Build binaries

```console
$ ./scripts/build.sh Default
```

This should produce the following outputs:

- `out/Default/headless_shell`: browser binary
- `out/Default/icudtl.dat`
- `out/Default/libEGL.so`
- `out/Default/libGLESv2.so`
- `out/Default/v8_context_snapshot.bin`

#### Build Docker image

```console
# Build arm64 Docker image using binaries from the Default target
$ ./scripts/docker-build.sh Default arm64
# Build amd64 Docker image using binaries from the Default target
$ ./scripts/docker-build.sh Default amd64
```

#### Run

```
$ ./scripts/run.sh Default https://wikipedia.org
```
