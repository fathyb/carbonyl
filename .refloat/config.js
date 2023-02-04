const platforms = {
    macos: 'apple-darwin',
    linux: 'unknown-linux-gnu',
}
const archs = {
    arm64: 'aarch64',
    amd64: 'x86_64',
}

export const jobs = ['arm64', 'amd64']
    .flatMap((arch) =>
        ['macos', 'linux'].map((platform) => ({ platform, arch })),
    )
    .map(({ platform, arch }) => {
        const triple = `${archs[arch]}-${platforms[platform]}`
        const lib = `build/${triple}/release/libcarbonyl.dylib`

        return { platform, arch, triple, lib }
    })
    .flatMap(({ platform, arch, triple, lib }) => [
        {
            name: `Build core (${platform}/${arch})`,
            docker:
                platform === 'linux'
                    ? {
                          image: 'rust:1.67',
                          cache: ['/usr/local/cargo/registry'],
                      }
                    : undefined,
            agent: { tags: platform === 'linux' ? ['docker'] : [] },
            steps: [
                {
                    name: 'Install Rust toolchain',
                    command: `rustup target add ${triple}`,
                },
                {
                    name: 'Build core library',
                    command: `cargo build --target ${triple} --release`,
                    env: { MACOSX_DEPLOYMENT_TARGET: '10.13' },
                },
                {
                    name: 'Set core library install name',
                    command:
                        platform === 'macos'
                            ? `install_name_tool -id @executable_path/libcarbonyl.dylib ${lib}`
                            : 'echo not necessary',
                },
                {
                    export: {
                        workspace: triple,
                        path: 'build/*/release/*.{dylib,so,dll}',
                    },
                },
            ],
        },
        {
            name: `Build (${platform}/${arch})`,
            agent: { tags: [platform, platform === 'macos' ? arch : 'amd64'] },
            steps: [
                {
                    import: {
                        workspace: triple,
                    },
                },
                {
                    name: 'Build Chromium',
                    command: `
                        if ! scripts/runtime-pull.sh; then
                            export GIT_CACHE_PATH="$HOME/.cache/git"
                            export CCACHE_DIR="$HOME/.cache/ccache"
                            export CCACHE_CPP2=yes
                            export CCACHE_BASEDIR="/Volumes/Data/Refloat"
                            export CCACHE_SLOPPINESS=file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches,pch_defines
        
                            ccache --set-config=max_size=32G

                            scripts/gclient.sh sync
                            scripts/patches.sh apply
                            scripts/gn.sh gen out/Default --args='import("//carbonyl/src/browser/args.gn") use_lld=false is_debug=false symbol_level=0 cc_wrapper="ccache"'
                            scripts/build.sh Default
                            scripts/copy-binaries.sh Default
                        fi
                    `,
                },
                {
                    parallel: [
                        {
                            name: 'Push pre-built binaries',
                            env: {
                                CDN_ACCESS_KEY_ID: { secret: true },
                                CDN_SECRET_ACCESS_KEY: { secret: true },
                            },
                            command: `
                                if [ -d chromium/src/out/Default ]; then
                                    scripts/runtime-push.sh
                                fi
                            `,
                        },
                        {
                            serial: [
                                {
                                    command: `
                                        mkdir build/zip
                                        cp -r build/pre-built/${triple} build/zip/${triple}
                                        cp ${lib} build/zip/${triple}

                                        cd build/zip/${triple}
                                        zip -r package.zip .
                                    `,
                                },
                                {
                                    export: {
                                        artifact: {
                                            name: `carbonyl.${platform}-${arch}.zip`,
                                            path: `build/zip/${triple}/package.zip`,
                                        },
                                    },
                                },
                            ],
                        },
                    ],
                },
            ],
        },
    ])
