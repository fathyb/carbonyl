import fs from 'fs/promises'
import path from 'path'
import { fileURLToPath } from 'url'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const pkg = JSON.parse(await fs.readFile(path.resolve(dirname, '../package.json'), 'utf-8'))
const version = process.env.RELEASE_MODE ? pkg.version : `${pkg.version}-next.${process.env.VERSION_ID}`
const manifest = {
    version,
    license: 'BSD-3-Clause',
    description: 'Chromium running in your terminal',
    homepage: 'https://github.com/fathyb/carbonyl',
    repository: 'fathyb/carbonyl',
    bugs: 'https://github.com/fathyb/carbonyl/issues',
    author: {
        name: 'Fathy Boundjadj',
        email: 'hey@fathy.fr',
        url: 'https://fathy.fr'
    }
}

async function buildMain() {
    const root = path.resolve(dirname, '../build/packages/carbonyl')

    await fs.rm(root, { recursive: true, force: true })
    await fs.mkdir(root, { recursive: true })
    await Promise.all([
        Promise.all(
            ['readme.md', 'license.md'].map(file =>
                fs.cp(path.join(dirname, '..', file), path.join(root, file)),
            )
        ),
        fs.writeFile(
            path.join(root, 'package.json'),
            JSON.stringify(
                {
                    name: 'carbonyl',
                    ...manifest,
                    files: ['index.sh', 'index.sh.js', 'index.js'],
                    bin: { carbonyl: 'index.sh' },
                    optionalDependencies: {
                        '@fathyb/carbonyl-linux-amd64': version,
                        '@fathyb/carbonyl-linux-arm64': version,
                        '@fathyb/carbonyl-macos-amd64': version,
                        '@fathyb/carbonyl-macos-arm64': version
                    }
                },
                null,
                4
            )
        ),
        fs.writeFile(
            path.join(root, 'index.sh'),
            [
                '#!/usr/bin/env bash',
                `"$(node "$(realpath "$0")".js)" "$@"`
            ].join('\n'),
            { mode: '755' }
        ),
        fs.writeFile(
            path.join(root, 'index.sh.js'),
            `process.stdout.write(require('.'))`
        ),
        fs.writeFile(
            path.join(root, 'index.js'),
            `
                function tryModule(name) {
                    try {
                        return require(name)
                    } catch {
                        return null
                    }
                }
    
                const path = (
                    tryModule('@fathyb/carbonyl-linux-amd64') ||
                    tryModule('@fathyb/carbonyl-linux-arm64') ||
                    tryModule('@fathyb/carbonyl-macos-amd64') ||
                    tryModule('@fathyb/carbonyl-macos-arm64')
                )
    
                if (path) {
                    module.exports = path
                } else {
                    throw new Error('Could not find a Carbonyl runtime installed')
                }
            `
        ),
    ])

    return root
}
async function buildPlatform([os, npmOs, llvmOs], [cpu, npmCpu, llvmCpu]) {
    const pkg = `carbonyl-${os}-${cpu}`
    const root = path.resolve(dirname, `../build/packages/${pkg}`)

    await fs.rm(root, { recursive: true, force: true })
    await fs.mkdir(root, { recursive: true })
    await Promise.all([
        Promise.all(
            ['readme.md', 'license.md'].map(file =>
                fs.cp(path.join(dirname, '..', file), path.join(root, file)),
            )
        ),
        fs.cp(
            path.join(dirname, `../build/pre-built/${llvmCpu}-${llvmOs}`),
            path.join(root, 'build'),
            { recursive: true }
        ),
        fs.writeFile(
            path.join(root, 'package.json'),
            JSON.stringify(
                {
                    name: `@fathyb/${pkg}`,
                    ...manifest,
                    files: ['build', 'index.js'],
                    os: [npmOs],
                    cpu: [npmCpu],
                },
                null,
                4
            )
        ),
        fs.writeFile(
            path.join(root, 'index.js'),
            `module.exports = __dirname + '/build/carbonyl'`
        )
    ])

    return root
}

const [root, platforms] = await Promise.all([
    buildMain(),

    Promise.all([
        ['macos', 'darwin', 'apple-darwin'],
        ['linux', 'linux', 'unknown-linux-gnu']
    ].map(async (os) =>
        await Promise.all(
            [
                ['arm64', 'arm64', 'aarch64'],
                ['amd64', 'x64', 'x86_64']
            ].map(async (cpu) => await buildPlatform(os, cpu))
        )
    )),
])


