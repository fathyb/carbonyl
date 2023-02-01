const env = {
    command: `
        export GIT_CACHE_PATH="$HOME/.cache/git"
        export CCACHE_DIR="$HOME/.cache/ccache"
        export CCACHE_CPP2=yes
        export CCACHE_SLOPPINESS=time_macros
    `,
    env: {
        export: [
            'GIT_CACHE_PATH',
            'CCACHE_DIR',
            'CCACHE_CPP2',
            'CCACHE_SLOPPINESS',
        ],
    },
}

export const jobs = [
    {
        name: 'Build (macOS/arm64)',
        using: env,
        agent: { tags: ['macos', 'arm64'] },
        steps: [
            'ccache --set-config=max_size=256G',
            './scripts/gclient.sh sync',
            './scripts/patches.sh apply',
            [
                './scripts/gn.sh',
                'gen',
                'out/Default',
                `--args='import("//carbonyl/src/browser/args.gn") use_lld=false is_debug=false symbol_level=0 is_official_build=true'`,
            ].join(' '),
            './scripts/build.sh Default',
        ],
    },
]
