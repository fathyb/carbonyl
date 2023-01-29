FROM debian:bullseye-slim

RUN groupadd -r carbonyl && useradd -r -g carbonyl carbonyl && \
    apt-get update && \
    apt-get install -y \
        libasound2 libatk-bridge2.0-0 libatk1.0-0 libatomic1 libatspi2.0-0 \
        libbrotli1 libc6 libcairo2 libcups2 libdbus-1-3 libdouble-conversion3 \
        libdrm2 libevent-2.1-7 libexpat1 libflac8 libfontconfig1 libfreetype6 \
        libgbm1 libgcc-s1 libglib2.0-0 libjpeg62-turbo libjsoncpp24 liblcms2-2 \
        libminizip1 libnspr4 libnss3 libopenjp2-7 libopus0 libpango-1.0-0 \
        libpng16-16 libpulse0 libre2-9 libsnappy1v5 libstdc++6 libwebp6 \
        libwebpdemux2 libwebpmux3 libwoff1 libx11-6 libxcb1 libxcomposite1 \
        libxdamage1 libxext6 libxfixes3 libxkbcommon0 libxml2 libxnvctrl0 \
        libxrandr2 libxslt1.1 zlib1g libgtk-3-0 && \
    rm -rf /var/lib/apt/lists/*

USER carbonyl

ARG TARGETARCH
COPY build/browser/${TARGETARCH:-amd64} /carbonyl

ENTRYPOINT ["/carbonyl/carbonyl", "--no-sandbox", "--disable-dev-shm-usage"]
