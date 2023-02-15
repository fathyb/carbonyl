import { commit } from "refloat";
import docker from "github.com/refloat-plugins/docker";

import pkg from "../package.json";

const { version } = JSON.parse(pkg);
const triple = (platform, arch) => `${archs[arch]}-${platforms[platform]}`;
const lib = (platform, arch) =>
  `build/${triple(platform, arch)}/release/libcarbonyl.${sharedLib[platform]}`;
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

export const jobs = ["macos", "linux"].flatMap((platform) => {
  return [
    {
      name: `Build runtime (${platform})`,
      agent: { tags: ["chromium-src", platform] },
      steps: [
        ...["arm64", "amd64"].map((arch) => ({
          import: { workspace: `core-${triple(platform, arch)}` },
        })),
        {
          parallel: ["arm64", "amd64"].map((arch) => ({
            name: `Fetch pre-built runtime for ${arch}`,
            command: `
              if scripts/runtime-pull.sh ${arch}; then
                  touch skip-build-${arch}
                  cp \\
                    ${lib(platform, arch)} \\
                    build/pre-built/${triple(platform, arch)}
              fi
            `,
          })),
        },
        {
          name: "Fetch Chromium",
          command: `
            if [ -z "$CHROMIUM_ROOT" ]; then
                echo "Chromium build environment not setup"

                exit 2
            fi

            if [ ! -f skip-build-arm64 ] || [ ! -f skip-build-amd64 ]; then
                cp chromium/.gclient "$CHROMIUM_ROOT"

                scripts/gclient.sh sync
                scripts/patches.sh apply

                rm -rf "$CHROMIUM_ROOT/src/carbonyl"
                mkdir "$CHROMIUM_ROOT/src/carbonyl"
                ln -s "$(pwd)/src" "$CHROMIUM_ROOT/src/carbonyl/src"
                ln -s "$(pwd)/build" "$CHROMIUM_ROOT/src/carbonyl/build"
            fi
          `,
        },
        {
          parallel: ["arm64", "amd64"].map((arch) => {
            const target =
              platform === "linux" && arch === "amd64" ? "Default" : arch;

            return {
              serial: [
                {
                  name: `Build Chromium (${arch})`,
                  command: `
                    if [ ! -f skip-build-${arch} ]; then
                        scripts/build.sh ${target} ${arch}
                        scripts/copy-binaries.sh ${target} ${arch}
                    fi
                  `,
                  env: {
                    MACOSX_DEPLOYMENT_TARGET: "10.13",
                    CARBONYL_SKIP_CARGO_BUILD: "true",
                    AR_AARCH64_UNKNOWN_LINUX_GNU: "aarch64-linux-gnu-ar",
                    CC_AARCH64_UNKNOWN_LINUX_GNU: "aarch64-linux-gnu-gcc",
                    CXX_AARCH64_UNKNOWN_LINUX_GNU: "aarch64-linux-gnu-g++",
                    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER:
                      "aarch64-linux-gnu-gcc",
                    AR_X86_64_UNKNOWN_LINUX_GNU: "x86_64-linux-gnu-ar",
                    CC_X86_64_UNKNOWN_LINUX_GNU: "x86_64-linux-gnu-gcc",
                    CXX_X86_64_UNKNOWN_LINUX_GNU: "x86_64-linux-gnu-g++",
                    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER:
                      "x86_64-linux-gnu-gcc",
                  },
                },

                {
                  parallel: [
                    {
                      name: `Push binaries to CDN (${arch})`,
                      command: `
                        if [ ! -f skip-build-${arch} ]; then
                            scripts/runtime-push.sh ${arch}
                        fi
                      `,
                      env: {
                        CDN_ACCESS_KEY_ID: { secret: true },
                        CDN_SECRET_ACCESS_KEY: { secret: true },
                      },
                    },
                    {
                      export: {
                        workspace: `runtime-${triple(platform, arch)}`,
                        path: `build/pre-built/${triple(platform, arch)}`,
                      },
                    },
                  ],
                },
              ],
            };
          }),
        },
      ],
    },
    ...["arm64", "amd64"].flatMap((arch) => {
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
      ];
    }),
  ];
});

if (commit.defaultBranch) {
  jobs.push(
    {
      name: "Publish to Docker",
      agent: { tags: ["carbonyl-publish"] },
      docker: "fathyb/rust-cross",
      steps: [
        {
          serial: ["arm64", "amd64"].map((arch) => ({
            import: { workspace: `runtime-${triple("linux", arch)}` },
          })),
        },
        {
          parallel: ["arm64", "amd64"].map((arch) => ({
            serial: [
              {
                name: `Build ${arch} image`,
                command: `scripts/docker-build.sh ${arch}`,
              },
            ],
          })),
        },
        {
          name: "Publish images to DockerHub",
          command: "scripts/docker-push.sh next",
          using: docker.login({
            username: { secret: "DOCKER_PUBLISH_USERNAME" },
            password: { secret: "DOCKER_PUBLISH_TOKEN" },
          }),
        },
      ],
    },
    {
      name: "Publish to npm",
      agent: { tags: ["carbonyl-publish"] },
      docker: "node:18",
      steps: [
        ...["macos", "linux"].flatMap((platform) =>
          ["arm64", "amd64"].map((arch) => ({
            import: { workspace: `runtime-${triple(platform, arch)}` },
          }))
        ),
        {
          name: "Package",
          command: "scripts/npm-package.sh",
        },
        {
          name: "Write npm token",
          env: { CARBONYL_NPM_PUBLISH_TOKEN: { secret: true } },
          command:
            'echo "//registry.npmjs.org/:_authToken=${CARBONYL_NPM_PUBLISH_TOKEN}" > ~/.npmrc',
        },
        {
          parallel: ["amd64", "arm64"].flatMap((arch) =>
            ["linux", "macos"].map((platform) => ({
              name: `Publish ${platform}/${arch} package`,
              command: "scripts/npm-publish.sh --tag next",
              env: {
                CARBONYL_PUBLISH_ARCH: arch,
                CARBONYL_PUBLISH_PLATFORM: platform,
              },
            }))
          ),
        },
        {
          name: "Publish main package",
          command: "scripts/npm-publish.sh --tag next",
        },
      ],
    }
  );
}
