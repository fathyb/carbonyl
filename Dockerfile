FROM debian:11 AS chromium-build

WORKDIR /app

ENV PATH="${PATH}:/app/depot_tools"
ENV CCACHE_DIR=/app/.ccache
ENV GIT_CACHE_PATH=/app/.git_cache
ENV DEBIAN_FRONTEND=noninteractive
ENV CHROMIUM_BUILDTOOLS_PATH=/app/electron/src/buildtools
RUN apt-get update && \
        apt-get install -y git sudo curl ccache python3 bzip2 xz-utils && \
    curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    git clone --depth 1 --single-branch https://chromium.googlesource.com/chromium/tools/depot_tools.git

COPY electron/.gclient electron/
COPY scripts/gclient.sh scripts/
RUN --mount=type=cache,target=/app/.git_cache scripts/gclient.sh

RUN electron/src/build/install-build-deps.sh

COPY scripts/patch.sh /app/scripts/
COPY src/chromium.patch /app/src/
RUN scripts/patch.sh

FROM chromium-build AS chromium-arm64

RUN electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=arm64

COPY scripts/gn.sh /app/scripts/
RUN GN_ARGS='target_cpu="arm64" cc_wrapper="env CCACHE_DIR=/app/.ccache CCACHE_SLOPPINESS=time_macros ccache"' \
        scripts/gn.sh release

COPY scripts/ninja.sh /app/scripts/
RUN --mount=type=cache,target=/app/.ccache \
    --mount=type=cache,target=/app/.git_cache \
    electron/src/build/linux/sysroot_scripts/install-sysroot.py --arch=arm64 && \
    scripts/ninja.sh release -j200

FROM chromium-arm64

COPY --from=chromium-build /app/electron/src/out /app/electron/src/out
