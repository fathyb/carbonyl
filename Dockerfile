FROM debian:11 AS build-env

ARG WORKDIR
WORKDIR ${WORKDIR} 

ENV PATH="${PATH}:/depot_tools"
ENV CCACHE_DIR="${WORKDIR}/.ccache"
ENV GIT_CACHE_PATH="${WORKDIR}/.git_cache"
ENV DEBIAN_FRONTEND=noninteractive
ENV CHROMIUM_BUILDTOOLS_PATH="${WORKDIR}/electron/src/buildtools"
ENV CCACHE_DIR="${WORKDIR}/.ccache"
ENV CCACHE_CPP2=yes
ENV CCACHE_SLOPPINESS=time_macros
RUN apt-get update && \
        apt-get install -y git sudo curl ccache python3 bzip2 xz-utils \
        binutils binutils-aarch64-linux-gnu binutils-arm-linux-gnueabihf binutils-mips64el-linux-gnuabi64 binutils-mipsel-linux-gnu bison bzip2 cdbs curl dbus-x11 devscripts dpkg-dev elfutils fakeroot flex git-core gperf libasound2 libasound2-dev libatk1.0-0 libatspi2.0-0 libatspi2.0-dev libbluetooth-dev libbrlapi-dev libbrlapi0.8 libbz2-1.0 libbz2-dev libc6 libc6-dev libcairo2 libcairo2-dev libcap-dev libcap2 libcups2 libcups2-dev libcurl4-gnutls-dev libdrm-dev libdrm2 libegl1 libelf-dev libevdev-dev libevdev2 libexpat1 libffi-dev libffi7 libfontconfig1 libfreetype6 libgbm-dev libgbm1 libgl1 libglib2.0-0 libglib2.0-dev libglu1-mesa-dev libgtk-3-0 libgtk-3-dev libinput-dev libinput10 libjpeg-dev libkrb5-dev libnspr4 libnspr4-dev libnss3 libnss3-dev libpam0g libpam0g-dev libpango-1.0-0 libpangocairo-1.0-0 libpci-dev libpci3 libpcre3 libpixman-1-0 libpng16-16 libpulse-dev libpulse0 libsctp-dev libspeechd-dev libspeechd2 libsqlite3-0 libsqlite3-dev libssl-dev libstdc++6 libudev-dev libudev1 libuuid1 libva-dev libvulkan-dev libvulkan1 libwayland-egl1 libwayland-egl1-mesa libwww-perl libx11-6 libx11-xcb1 libxau6 libxcb1 libxcomposite1 libxcursor1 libxdamage1 libxdmcp6 libxext6 libxfixes3 libxi6 libxinerama1 libxkbcommon-dev libxrandr2 libxrender1 libxshmfence-dev libxslt1-dev libxss-dev libxt-dev libxtst-dev libxtst6 locales mesa-common-dev openbox p7zip patch perl pkg-config rpm ruby subversion uuid-dev wdiff x11-utils xcompmgr xz-utils zip zlib1g zstd && \
    curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    apt-get install -y nodejs && \
    git clone --depth 1 --single-branch https://chromium.googlesource.com/chromium/tools/depot_tools.git /depot_tools && \
    ccache --max-size=256G

# Release binaries
# ================
FROM --platform=$BUILDPLATFORM debian:11 AS html2svg-binaries

RUN apt-get update && apt-get install -y unzip

ARG TARGETARCH
COPY electron/src/out/release-$TARGETARCH/dist.zip /runtime.zip
RUN unzip /runtime.zip -d /runtime

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
        xvfb x11-xkb-utils xfonts-100dpi xfonts-75dpi xfonts-scalable xfonts-cyrillic x11-apps \
        fonts-arphic-ukai fonts-arphic-uming fonts-ipafont-mincho fonts-ipafont-gothic fonts-unfonts-core fonts-noto-core

WORKDIR /app
COPY package.json yarn.lock /app/
RUN yarn --production

COPY --from=html2svg-js /app/build /app/build
COPY --from=html2svg-binaries /runtime /app/build/runtime
COPY /scripts/docker-entrypoint.sh /app/scripts/docker-entrypoint.sh

ENTRYPOINT ["/app/scripts/docker-entrypoint.sh"]

