const env = {
    command: `
        export GIT_CACHE_PATH="$HOME/.cache/git"
        export CCACHE_DIR="$HOME/.cache/ccache"
        export CCACHE_CPP2=yes
        export CCACHE_BASEDIR="$HOME/Library/Refloat"
        export CCACHE_SLOPPINESS=file_macro,time_macros,include_file_mtime,include_file_ctime,file_stat_matches,pch_defines
    `,
    env: {
        export: [
            'GIT_CACHE_PATH',
            'CCACHE_DIR',
            'CCACHE_CPP2',
            'CCACHE_BASEDIR',
            'CCACHE_SLOPPINESS',
        ],
    },
}

export const jobs = ['arm64', 'amd64'].map((arch) => ({
    name: `Build (macOS/${arch})`,
    using: env,
    agent: { tags: ['macos', arch] },
    steps: [
        'ccache --set-config=max_size=256G',
        './scripts/gclient.sh sync',
        './scripts/patches.sh apply',
        [
            './scripts/gn.sh',
            'gen',
            'out/Default',
            `--args='import("//carbonyl/src/browser/args.gn") use_lld=false is_debug=false symbol_level=0 cc_wrapper="ccache"'`,
        ].join(' '),
        './scripts/build.sh Default',
    ],
}))
