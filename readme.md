# `html2svg`

Convert HTML and `<canvas>` to SVG using Chromium.

## Usage

```shell
$ docker run fathyb/html2svg https://google.com > google.svg
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
