# `html2svg`

Convert HTML and `<canvas>` to SVG or PDF using Chromium. [Read the blog post](https://fathy.fr/html2svg).

## Usage

```shell
# export to SVG
$ docker run fathyb/html2svg https://google.com > google.svg
$ docker run fathyb/html2svg https://google.com --format svg > google.svg
# export to PDF
$ docker run fathyb/html2svg https://google.com --format pdf > google.pdf
# show help
$ docker run fathyb/html2svg --help
Usage: html2svg [options] <url>

Arguments:
  url                    URL to the web page to render

Options:
  -f, --full             capture the entire page
  -w, --wait <seconds>   set the amount of seconds to wait between the page loaded event and taking the screenshot (default: 1)
  -w, --width <width>    set the viewport width in pixels (default: 1920)
  -h, --height <height>  set the viewport height in pixels (default: 1080)
  -f, --format <format>  set the output format, should one of these values: svg, pdf (default: "svg")
  --help                 display help for command
```

## Development

Building Chromium is only officially supported on AMD64. If you'd like to target ARM64, cross-compile from AMD64 instead.

### Local

You'll need to install all tools required to build Chromium: https://www.chromium.org/developers/how-tos/get-the-code/

If you're running Linux, you can use [the Docker build instructions](#docker) to generate binaries.

1. Fetch dependencies:
    ```shell
    $ yarn
    ```
2. Clone Electron.js and Chromium using `gclient`:
    ```shell
    $ yarn gclient
    ```
3. Configure the build system using `gn` using one of these commands:
    ```shell
    # for local developement
    $ yarn gn testing
    # or for releasing
    $ yarn gn release
    # add --ide=xcode if you'd like to generate an Xcode project on macOS
    $ yarn gn release --ide=xcode
    ```
4. Build using `ninja` using one of these commands:
    ```shell
    # make a testing build
    $ yarn ninja testing
    # make a release build
    $ yarn ninja release
    ```

### Docker

We use `docker run` instead of `Dockerfile` for building Chromium to support incremental building.

```shell
# Create the build environment
$ docker build . --build-arg "WORKDIR=$(pwd)" --target build-env --tag html2svg-build-env
# Clone the Chromium/Electron code
$ docker run -ti -v $(pwd):$(pwd) html2svg-build-env scripts/gclient.sh --revision "src/electron@cb22573c3e76e09df9fbad36dc372080c04d349e"
# Apply html2svg patches
$ docker run -ti -v $(pwd):$(pwd) html2svg-build-env scripts/patch.sh
# Install build dependencies
$ docker run -ti -v $(pwd):$(pwd) html2svg-build-env electron/src/build/install-build-deps.sh
```

Now you'll have to build binaries, steps differs depending on the platform you'd like to target:
- AMD64:
  ```shell
  # Fetch compiler files
  $ docker run -ti -v $(pwd):$(pwd) html2svg-build-env electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=amd64
  # Generate build files
  $ docker run -ti -v $(pwd):$(pwd) --workdir $(pwd)/electron/src html2svg-build-env gn gen "out/release-amd64" --args="import(\"//electron/build/args/release.gn\") cc_wrapper=\"ccache\""
  # Build binaries
  $ docker run -ti -v $(pwd):$(pwd) html2svg-build-env scripts/build.sh release-amd64
  ```
- ARM64:
  ```shell
  # Fetch compiler files
  $ docker run -ti -v $(pwd):$(pwd) html2svg-build-env electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=arm64
  # Generate build files
  $ docker run -ti -v $(pwd):$(pwd) --workdir $(pwd)/electron/src html2svg-build-env gn gen "out/release-arm64" --args="import(\"//electron/build/args/release.gn\") cc_wrapper=\"ccache\" target_cpu=\"arm64\""
  # Build binaries
  $ docker run -ti -v $(pwd):$(pwd) html2svg-build-env scripts/build.sh release-arm64 --target-cpu=arm64
  ```

Finally, build the Docker image:
```shell
docker build .
```

