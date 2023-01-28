# `carbonyl`

Carbonyl is a Chromium based browser built to run in a terminal. [Read the blog post](https://fathy.fr/carbonyl).

It supports pretty much all Web APIs including WebGL, WebGPU, audio and video playback, animations, etc..

It's snappy, starts in less than a second, runs at 60 FPS, and idles at 0% CPU usage. It does not require a window server (i.e. works in a safe-mode console), and even runs through SSH.

Carbonyl originally started as [`html2svg`](https://github.com/fathyb/html2svg) and is now the runtime behind it.

## Usage

```shell
# Watch YouTube inside a Docker container
$ docker run -ti fathyb/carbonyl https://youtube.com
```

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

## Know issues

- Fullscreen mode not supported yet

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
