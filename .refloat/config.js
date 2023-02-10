import pkg from "../package.json";

const { version } = JSON.parse(pkg);
const triple = (platform, arch) => `${archs[arch]}-${platforms[platform]}`
const sharedLib = {
    macos: "dylib",
    linux: "so",
};
const platforms = {
    macos: "apple-darwin",
    linux: "unknown-linux-gnu",
};
const archs = {
    arm64: "aarch64",
    amd64: "x86_64",
};

export const jobs = ["macos", "linux"].flatMap(platform => {
    return [
        {
            name: `Build runtime (${platform})`,
            agent: { tags: ["chromium-src", platform] },
            steps: [
                {
                    parallel: ['arm64', 'amd64'].map(arch => (
                        { import: { workspace: `core-${triple(platform, arch)}` } }
                    ))
                },
                {
                    name: 'Fetch Chromium',
                    command: `
                        if [ -z "$CHROMIUM_ROOT" ]; then
                            echo "Chromium build environment not setup"

                            exit 2
                        fi

                        if ! scripts/runtime-pull.sh arm64; then
                            touch skip-build-arm64
                        fi
                        if ! scripts/runtime-pull.sh amd64; then
                            touch skip-build-amd64
                        fi

                        if [ ! -f skip-build-amd64 ] || [ ! -f skip-build-amd64 ]; then
                            cp chromium/.gclient "$CHROMIUM_ROOT"

                            scripts/gclient.sh sync
                            scripts/patches.sh apply

                            rm -rf "$CHROMIUM_ROOT/src/carbonyl"
                            mkdir "$CHROMIUM_ROOT/src/carbonyl"
                            ln -s "$(pwd)/src" "$CHROMIUM_ROOT/src/carbonyl/src"
                            ln -s "$(pwd)/build" "$CHROMIUM_ROOT/src/carbonyl/build"
                        fi
                    `
                },
                {
                    parallel: ['arm64', 'amd64'].map(arch => ({
                        serial: [
                            {
                                name: 'Build Chromium',
                                command: `
                                    if [ ! -f skip-build-${arch} ]; then
                                        scripts/build.sh ${arch}
                                        scripts/copy-binaries.sh ${arch} ${arch}
                                    fi
                                `
                            },

                            {
                                parallel: [
                                    {
                                        name: 'Push binaries to CDN',
                                        command: `
                                            if [ ! -f skip-build-${arch} ]; then
                                                scripts/runtime-push.sh ${arch}
                                            fi
                                        `,
                                        env: {
                                            CDN_ACCESS_KEY_ID: { secret: true },
                                            CDN_SECRET_ACCESS_KEY: { secret: true }
                                        }
                                    },
                                    {
                                        export: {
                                            workspace: `runtime-${triple(platform, arch)}`,
                                            path: `build/pre-built/${triple(platform, arch)}`
                                        }
                                    }
                                ]
                            }
                        ]
                    }))
                }
            ],
        },
        ...['arm64', 'amd64'].flatMap((arch) => {
            const triple = `${archs[arch]}-${platforms[platform]}`;
            const lib = `build/${triple}/release/libcarbonyl.${sharedLib[platform]}`;

            return [
                {
                    name: `Build core (${platform}/${arch})`,
                    docker:
                        platform === "linux"
                            ? {
                                image: "fathyb/rust-cross",
                                cache: ["/usr/local/cargo/registry"],
                            }
                            : undefined,
                    agent: { tags: platform === "linux" ? ["docker"] : ["macos"] },
                    steps: [
                        {
                            name: "Install Rust toolchain",
                            command: `rustup target add ${triple}`,
                        },
                        {
                            name: "Build core library",
                            command: `cargo build --target ${triple} --release`,
                            env: { MACOSX_DEPLOYMENT_TARGET: "10.13" },
                        },
                        {
                            name: "Set core library install name",
                            command:
                                platform === "macos"
                                    ? `install_name_tool -id @executable_path/libcarbonyl.dylib ${lib}`
                                    : "echo not necessary",
                        },
                        {
                            export: {
                                workspace: `core-${triple}`,
                                path: "build/*/release/*.{dylib,so,dll}",
                            },
                        },
                    ],
                },
                {
                    // TODO: setup shared build dir
                    name: `Package (${platform}/${arch})`,
                    docker: "fathyb/rust-cross",
                    agent: { tags: ["docker"] },
                    steps: [
                        {
                            import: { workspace: `runtime-${triple}` },
                        },
                        {
                            name: "Zip binaries",
                            command: `
                            mkdir build/zip
                            cp -r build/pre-built/${triple} build/zip/carbonyl-${version}
    
                            cd build/zip
                            zip -r package.zip carbonyl-${version}
                        `,
                        },
                        {
                            export: {
                                artifact: {
                                    name: `carbonyl.${platform}-${arch}.zip`,
                                    path: "build/zip/package.zip",
                                },
                            },
                        },
                    ],
                },
            ]
        })
    ]
})
