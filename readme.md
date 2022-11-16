# `html2svg`

Convert HTML and `<canvas>` to SVG or PDF using Chromium.

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

> - Building Chromium for ARM on Linux or Windows is not officially supported, cross-compile from x64 instead.

### Docker

```shell
$ docker buildx build . --platform linux/arm64,linux/amd64
```

### Local

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
