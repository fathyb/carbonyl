export const jobs = [
    ...['aarch64-apple-darwin', 'x86_64-apple-darwin'].map((target) => ({
        name: `Build core (${target})`,
        steps: [
            {
                command: `rustup target add ${target}`,
            },
            {
                command: `cargo build --target ${target} --release`,
            },
            {
                export: {
                    artifact: `build/${target}/release/libcarbonyl.dylib`,
                },
            },
        ],
    })),

    ...['arm64', 'amd64'].map((arch) => ({
        name: `Build Chromium (macOS/${arch})`,
        agent: { tags: ['macos', arch] },
        steps: [
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
                name: 'Push pre-built binaries',
                env: {
                    CDN_ACCESS_KEY_ID: { secret: true },
                    CDN_SECRET_ACCESS_KEY: { secret: true },
                },
                command: `
                    if -d chromium/src/out/Default; then
                        scripts/runtime-push.sh
                    fi
                `,
            },
        ],
    })),
]
