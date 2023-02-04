const platforms = {
    macos: 'apple-darwin',
}
const archs = {
    arm64: 'aarch64',
    amd64: 'x86_64',
}
const triple = (arch, platform) => `${archs[arch]}-${platforms[platform]}`
const lib = (triple) => `build/${triple}/release/libcarbonyl.dylib`

export const jobs = [
    ...[triple('arm64', 'macos'), triple('amd64', 'macos')].map((target) => ({
        name: `Build core (${target})`,
        steps: [
            {
                name: 'Install toolchain',
                command: `rustup target add ${target}`,
            },
            {
                name: 'Build library',
                command: `cargo build --target ${target} --release`,
                env: { MACOSX_DEPLOYMENT_TARGET: '10.13' },
            },
            {
                name: 'Set library install name',
                command: `install_name_tool -id @executable_path/libcarbonyl.dylib ${lib(
                    target,
                )}`,
            },
            {
                export: { artifact: lib(target) },
            },
        ],
    })),

    ...['arm64', 'amd64'].map((arch) => ({
        name: `Build Chromium (macOS/${arch})`,
        agent: { tags: ['macos', arch] },
        steps: [
            {
                import: { artifact: lib(triple(arch, 'macos')) },
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
                        export: {
                            artifact: `build/pre-build/${triple(
                                arch,
                                'macos',
                            )}.tgz`,
                        },
                    },
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
                ],
            },
        ],
    })),
]
