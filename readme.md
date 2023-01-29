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

-   Fullscreen mode not supported yet

## Development

Few notes:

-   You need to build Chromium
-   Building Carbonyl is almost the same as building Chromium with extra steps to patch and bundle the Rust library. Scripts in the `scripts/` directory are simple wrappers around `gn`, `ninja`, etc..
-   Building Chromium for arm64 on Linux requires an amd64 processor
-   Carbonyl is only tested on Linux and macOS, other platforms likely require code changes to Chromium
-   Chromium is huge and takes a long time to build, making your computer mostly unresponsive. An 8-core CPU such as an M1 Max or an i9 9900k with 10 Gbps fiber takes around ~1 hour to fetch and build. It requires around 100 GB of disk space.

### Fetch

> Fetch Chromium's code.

```console
$ ./scripts/gclient.sh sync
```

### Apply patches

> Any changes made to Chromium will be reverted, make sure to save any changes you made.

```console
$ ./scripts/patches.sh apply
```

### Configure

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
# target_cpu="arm64"

# uncomment this to enable ccache
# cc_wrapper="env CCACHE_SLOPPINESS=time_macros ccache"

# uncomment this if you're building for macOS
# use_lld=false

# uncomment this for a release build
# is_debug=false
# symbol_level=0
```

### Build binaries

```console
$ ./scripts/build.sh Default
```

This should produce the following outputs:

-   `out/Default/headless_shell`
-   `out/Default/icudtl.dat`
-   `out/Default/libEGL.so`
-   `out/Default/libGLESv2.so`
-   `out/Default/v8_context_snapshot.bin`

### Build Docker image

```console
# Build arm64 Docker image using binaries from the Default target
$ ./scripts/docker.sh arm64 Default
# Build amd64 Docker image using binaries from the Default target
$ ./scripts/docker.sh amd64 Default
```

### Run

```
$ ./scripts/run.sh Default https://wikipedia.org
```
