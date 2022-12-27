# `carbonyl`

Carbonyl is a Chromium based browser built to run in a terminal. [Read the blog post](https://fathy.fr/carbonyl).

It supports pretty much all Web APIs including WebGL, WebGPU, audio and video playback, animations, etc..

It's snappy, starts in less than a second, runs at 60 FPS, and idles at 0% CPU usage. It does not requires a window server (ie. works in a safe-mode console), and even runs through SSH.

Carbonyl originally started as [`html2svg`](https://github.com/fathyb/html2svg) and is now the runtime behind it.

## Usage

> Currently building...

```shell
# Watch YouTube inside a Docker container
$ docker run fathyb/carbonyl youtube.com
```

## Development

### Fetch

```console
$ cd chromium
$ gclient sync
```

### Configure

> You need to disable `lld` on macOS because of a linking bug related to Rust and `compact_unwind`

```console
$ cd chromium/src
$ gn gen out/Default
```

### Build

```console
$ cd chromium/src
$ ninja -C out/Default headless:headless_shell
```
