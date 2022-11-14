# Build environment
# ==========================
FROM --platform=linux/amd64 debian:11 AS chromium-build

WORKDIR /app

ENV PATH="${PATH}:/app/depot_tools"
ENV CCACHE_DIR=/app/.ccache
ENV GIT_CACHE_PATH=/app/.git_cache
ENV DEBIAN_FRONTEND=noninteractive
ENV CHROMIUM_BUILDTOOLS_PATH=/app/electron/src/buildtools
RUN apt-get update && \
        apt-get install -y git sudo curl ccache python3 bzip2 xz-utils && \
    curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get install -y nodejs && \
    git clone --depth 1 --single-branch https://chromium.googlesource.com/chromium/tools/depot_tools.git

COPY electron/.gclient electron/
COPY scripts/gclient.sh scripts/
RUN --mount=type=cache,target=/app/.git_cache scripts/gclient.sh --revision "src/electron@cb22573c3e76e09df9fbad36dc372080c04d349e"

RUN electron/src/build/install-build-deps.sh

COPY scripts/patch.sh /app/scripts/
COPY src/chromium.patch /app/src/
COPY src/skia.patch /app/src/
RUN scripts/patch.sh && ccache --max-size=0

ENV CCACHE_DIR=/app/.ccache
ENV CCACHE_CPP2=yes
ENV CCACHE_SLOPPINESS=time_macros


# ARM64 binaries
# ==============
FROM --platform=linux/amd64 chromium-build AS chromium-arm64

RUN electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=arm64

COPY scripts/gn.sh /app/scripts/
RUN GN_ARGS='cc_wrapper="ccache" target_cpu="arm64"' \
        scripts/gn.sh release

COPY scripts/ninja.sh /app/scripts/
RUN --mount=type=cache,target=/app/.ccache \
    --mount=type=cache,target=/app/.git_cache \
    scripts/ninja.sh release -j200

RUN electron/src/electron/script/strip-binaries.py -d electron/src/out/release --target-cpu=arm64 && \
    ninja -C electron/src/out/release electron:electron_dist_zip


# AMD64 binaries
# ==============
FROM --platform=linux/amd64 chromium-build AS chromium-amd64

RUN electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=amd64

COPY scripts/gn.sh /app/scripts/
RUN GN_ARGS='cc_wrapper="ccache"' \
        scripts/gn.sh release

COPY scripts/ninja.sh /app/scripts/
RUN --mount=type=cache,target=/app/.ccache \
    --mount=type=cache,target=/app/.git_cache \
    scripts/ninja.sh release -j200

RUN electron/src/electron/script/strip-binaries.py -d electron/src/out/release && \
    ninja -C electron/src/out/release electron:electron_dist_zip


# Release binaries
# ================
FROM debian:11 AS html2svg-binaries

RUN apt-get update && apt-get install -y unzip

COPY --from=chromium-arm64 /app/electron/src/out/release/dist.zip /arm64.zip
COPY --from=chromium-amd64 /app/electron/src/out/release/dist.zip /amd64.zip
RUN unzip /arm64.zip -d /arm64
RUN unzip /amd64.zip -d /amd64

# TypeScript build
# ================
FROM --platform=$BUILDPLATFORM node:18 AS html2svg-js 

WORKDIR /app
COPY package.json yarn.lock /app/
RUN yarn

COPY tsconfig.json /app/
COPY src /app/src
RUN yarn tsc -b

# Main image
# ==========
FROM node:18

RUN apt-get update && \
    apt-get install --yes \
        libglib2.0-0 libnss3 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libgtk-3-0 libgbm1 libasound2 \
        xvfb x11-xkb-utils xfonts-100dpi xfonts-75dpi xfonts-scalable xfonts-cyrillic x11-apps

WORKDIR /app
COPY package.json yarn.lock /app/
RUN yarn --production

ARG TARGETARCH
COPY --from=html2svg-js /app/build /app/build
COPY --from=html2svg-binaries /${TARGETARCH} /runtime
COPY /scripts/docker-entrypoint.sh /app/scripts/docker-entrypoint.sh

ENTRYPOINT ["/app/scripts/docker-entrypoint.sh"]

