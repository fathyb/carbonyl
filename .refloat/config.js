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
                        image: 'fathyb/rust-cross',
                        cache: ['/usr/local/cargo/registry'],
                    }
                    : undefined,
            agent: { tags: platform === 'linux' ? ['docker'] : ['macos'] },
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
            // TODO: setup shared build dir
            name: `Build (${platform}/${arch})`,
            docker: 'fathyb/rust-cross',
            agent: { tags: ['docker'] },
            steps: [
                {
                    import: {
                        workspace: triple,
                    },
                },
                {
                    name: 'Fetch runtime',
                    command: 'scripts/runtime-pull.sh',
                },
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
                }
            ],
        },
    ])
