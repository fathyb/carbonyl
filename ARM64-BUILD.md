# Building Carbonyl on arm64 Linux + File Download Support

This document covers:

1. **Building Carbonyl from source on Ubuntu 24.04 arm64** (tested in a Multipass VM on Apple M-series Mac)
2. **Adding file download support** — Carbonyl has no `DownloadManagerDelegate`, so downloads silently fail; these patches wire one up

All paths are relative to the Carbonyl source root (`~/carbonyl/carbonyl-src/`) unless prefixed with `chromium/src/`.

---

## Corrections from the verified 2026-07 rebuild

> This document was written against an earlier depot_tools/toolchain state. A full clean
> rebuild on 2026-07-18/19 (Ubuntu 24.04 arm64 Multipass VM, 8 CPU / 12 GB RAM) surfaced
> several deviations. With the fixes below the build completes with **zero errors** and is
> verified end-to-end: `headless_shell --version` prints `Carbonyl 0.0.3`, it renders live
> pages to the terminal, and file downloads save to `~/Downloads/`. The critical fixes,
> each detailed inline in the section noted:
>
> 1. **Pin depot_tools to a Chromium-111-era commit** — latest depot_tools requires
>    Python 3.11+ (`StrEnum`), which conflicts with the Python 3.10 that Chromium 111
>    needs. (§1)
> 2. **Use a Python 3.10 PATH shim, not `update-alternatives`** — repointing the system
>    `python3` breaks `apt`/`add-apt-repository` on Ubuntu 24.04. (Environment)
> 3. **Sync before applying patches** — `patches.sh apply` needs `chromium/src` to exist,
>    so the old "apply patches before syncing" order fails. (§1)
> 4. **Neutralize reclient in `chromium/src/DEPS`** — the `.gclient` `custom_deps: None`
>    does *not* suppress the arm64 cipd fetch of `infra/rbe/client/linux-arm64`. (§1)
> 5. **`gn` and `esbuild` arrive automatically** as arm64 cipd binaries from the sync — the
>    manual build/copy steps in §2f/§2h are normally unnecessary. (§2f/§2h)
> 6. **`LASTCHANGE.committime` must be a bare integer** — the `lastchange.py` fallback
>    writes a `key=value` blob that breaks `compute_build_timestamp.py`. (§2g)
> 7. **Apply `third_party/node/typescript.patch` after `npm install`** — without it the
>    WebUI `tsc` step fails with `TrustedHTML not assignable to string`. (§2i)
> 8. **`DAWN_VERSION` must have no trailing newline** — a newline becomes an unterminated
>    C++ string literal that cascades into bogus Dawn compile errors. (§2k)
> 9. **Running the binary** needs `libcarbonyl.so` beside it, extra runtime apt packages,
>    and `--no-sandbox` inside a VM. (§6)

---

## Environment

- Ubuntu 24.04 arm64 (Multipass VM on Apple M4, 16 GB RAM allocated)
- Chromium 111.0.5511.1 (vendored by Carbonyl)
- System clang-18 (Google's pre-built clang is x86_64-only)
- Python 3.10 (Chromium 111 build scripts are not compatible with Python 3.12)

### System packages

```bash
sudo apt-get install -y \
  clang-18 lld-18 lld \
  gperf \
  python3-html5lib \
  nodejs \
  curl git
```

### Python 3.10

Ubuntu 24.04 ships Python 3.12, which breaks Chromium 111's build scripts (the `imp`
module was removed in 3.12 and bundled `six` 1.14 is broken). Install Python 3.10
from the deadsnakes PPA:

```bash
sudo add-apt-repository ppa:deadsnakes/ppa -y
sudo apt-get install -y python3.10 python3.10-distutils python3.10-dev
```

> **Use a PATH shim, not `update-alternatives` (corrected 2026-07).** Repointing the
> system `/usr/bin/python3` to 3.10 breaks Ubuntu 24.04's `apt`, `add-apt-repository`,
> and other system tools that require 3.12. Instead, put a 3.10 `python3` earlier on
> `PATH` only for the build. This keeps the system default at 3.12 while the build sees
> 3.10:
>
> ```bash
> mkdir -p ~/py310
> ln -sf /usr/bin/python3.10 ~/py310/python3
> ln -sf /usr/bin/python3.10 ~/py310/python
> ```
>
> Then source a small build-env file in every build shell (also used by later steps):
>
> ```bash
> cat > ~/buildenv.sh << 'EOF'
> export PATH="$HOME/py310:$HOME/carbonyl/carbonyl-src/chromium/depot_tools:$HOME/.cargo/bin:$PATH"
> export DEPOT_TOOLS_UPDATE=0
> export VPYTHON_BYPASS="manually managed python not supported by chrome operations"
> EOF
> source ~/buildenv.sh
> python3 --version   # should show 3.10.x here, still 3.12 in a normal shell
> ```

---

## 1. Checkout

```bash
# Use this fork (carylewis/carbonyl) — it already contains the file-download support
# committed under src/browser/ and chromium/patches/chromium/0015-*, so §5 is baked in.
git clone https://github.com/carylewis/carbonyl ~/carbonyl/carbonyl-src
cd ~/carbonyl/carbonyl-src

# Fetch depot_tools
git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git chromium/depot_tools
```

> **Pin depot_tools to the Chromium-111 era (corrected 2026-07).** The latest depot_tools
> requires Python 3.11+ (`from enum import StrEnum`), which is incompatible with the
> Python 3.10 that Chromium 111 needs — `gclient` crashes on import. Pin it to a
> contemporaneous commit and disable auto-update (`DEPOT_TOOLS_UPDATE=0`, already set in
> `~/buildenv.sh`):
>
> ```bash
> ( cd chromium/depot_tools && \
>   git checkout -q "$(git rev-list -1 --before=2023-03-15 origin/main)" )
> ```

```bash
source ~/buildenv.sh   # puts py310 shim + depot_tools on PATH, DEPOT_TOOLS_UPDATE=0

cd chromium
gclient sync --no-history --shallow --nohooks   # Chromium 111 + sub-repos, ~14 GB, ~5 min
```

> **Sync *before* applying patches (corrected 2026-07).** `scripts/patches.sh apply`
> `cd`s into `chromium/src` and `git checkout`s the upstream base commits, so `chromium/src`
> must already exist. The old "apply patches before syncing" order fails with
> `cd: chromium/src: No such file or directory`. With `--shallow` the patch-base commits
> (`92da8189…`, Skia `486deb23…`, WebRTC `727080cb…`) are exactly the synced HEADs, so the
> checkouts in `patches.sh` are no-ops and succeed.

> **Neutralize reclient in `chromium/src/DEPS` (corrected 2026-07).** `reclient`
> (`infra/rbe/client/${platform}`) has no `linux-arm64` cipd package, and the `.gclient`
> `custom_deps: {"src/buildtools/reclient": None}` does **not** suppress the fetch — the
> sync dies with `no such package: infra/rbe/client/linux-arm64`. Give the reclient block
> in `chromium/src/DEPS` an impossible condition, then re-run the sync:
>
> ```bash
> # In chromium/src/DEPS, inside the 'src/buildtools/reclient': { ... } block, add:
> #   'condition': 'host_os == "win" and host_os == "linux"',   # always false → skipped
> cd ~/carbonyl/carbonyl-src/chromium && gclient sync --no-history --shallow --nohooks
> ```
>
> Set a git identity first (`git config --global user.email/user.name`) so `patches.sh`'s
> `git am` can commit. Then apply patches:
>
> ```bash
> cd ~/carbonyl/carbonyl-src && ./scripts/patches.sh apply
> ```

> **Note on hooks:** we sync with `--nohooks` on purpose. The arm64-incompatible hooks are
> exactly the ones replaced by the manual steps in §2 (sysroot, gn, LASTCHANGE, esbuild,
> node_modules, generated headers). `gn` and `esbuild` still arrive as arm64 cipd binaries
> during the dep sync, so §2f/§2h are usually already satisfied — check before doing them
> manually.

---

## 2. Pre-build fixes

### 2a. arm64 Node.js detection in `node.py`

The bundled node helper doesn't recognize `aarch64` Linux. Patch it to use the
`node-linux-arm64` binary:

**File:** `chromium/src/third_party/node/node.py`

```python
# Replace the 'Linux' line in GetBinaryPath():

# before
'Linux': ('linux', 'node-linux-x64', 'bin', 'node'),

# after
'Linux': ('linux', 'node-linux-arm64' if platform.machine() == 'aarch64' else 'node-linux-x64', 'bin', 'node'),
```

Then create a symlink so the expected binary path resolves:

```bash
mkdir -p chromium/src/third_party/node/linux/node-linux-arm64/bin
ln -sf /usr/bin/node chromium/src/third_party/node/linux/node-linux-arm64/bin/node
```

---

### 2b. Clang-18 narrowing errors

Clang 18 is stricter than the Clang 16 Chromium 111 was written for. Two files need
`-Wno-c++11-narrowing-const-reference` added.

**File:** `chromium/src/build/config/compiler/BUILD.gn`

Find the `if (is_clang)` block that sets compiler warning flags (search for
`Wno-unneeded-internal-declaration`). Add inside that block:

```gn
"-Wno-c++11-narrowing-const-reference",
```

**File:** `chromium/src/third_party/webrtc/BUILD.gn`

Find the block `cflags += [ "-Wc++11-narrowing" ]` and extend it:

```gn
cflags += [ "-Wc++11-narrowing", "-Wno-c++11-narrowing-const-reference" ]
```

---

### 2c. `libcarbonyl.so` setup

The build links against `libcarbonyl.so`. In the repository the `carbonyl/build`
directory is a git placeholder (empty file). Set it up to point to the pre-built `.so`
from the Carbonyl release:

```bash
cd chromium/src

# The repo has 'build' as an empty placeholder file — replace with a real directory
rm ../build  # relative to carbonyl-src root: carbonyl-src/build
mkdir -p ../build/release

# Copy or download libcarbonyl.so into build/release/
# (copy from a working build, or build it with: cd carbonyl-src && cargo build --release)
cp /path/to/libcarbonyl.so ../build/release/

# The build system also looks for the cross-compile triple path; symlink it
mkdir -p ../build/aarch64-unknown-linux-gnu/release
ln -sf "$(pwd)/../build/release/libcarbonyl.so" \
       ../build/aarch64-unknown-linux-gnu/release/libcarbonyl.so
```

Note: `chromium/src/carbonyl/build` is a symlink to `../../../build` (the
`carbonyl-src/build` directory), so setting up `carbonyl-src/build` is what matters.

---

### 2d. glibc version mismatch — `--allow-shlib-undefined`

`libcarbonyl.so` is built against the host Ubuntu 24.04 glibc (2.38), but the build
uses a Debian Bullseye arm64 sysroot that only has glibc 2.31. lld rejects the link
by default. Since the binary runs on the host (glibc 2.38 present at runtime) this
check can safely be suppressed.

**File:** `chromium/src/headless/BUILD.gn`

In the `executable("headless_shell")` target, find the `} else if (is_linux)` ldflags
block and add the flag:

```gn
} else if (is_linux) {
  ldflags = [
    "-Wl,-rpath=\$ORIGIN/.",
    "-Wl,-rpath-link=.",
    "-Wl,--allow-shlib-undefined",
  ]
}
```

---

### 2e. arm64 sysroot

The Chromium build needs an arm64 Debian Bullseye sysroot. Install it:

```bash
cd chromium/src
python3 build/linux/sysroot_scripts/install-sysroot.py --arch=arm64
```

---

### 2f. `gn` binary

The `gn` build-file generator binary is not included in the repository and is not
fetched by `gclient sync` for arm64 hosts (it's only pre-built for x86_64). Either
build it from source or copy it from an existing build:

```bash
# Option A: copy from another build
cp /path/to/existing/buildtools/linux64/gn \
   chromium/src/buildtools/linux64/gn
chmod +x chromium/src/buildtools/linux64/gn

# Option B: build from source (requires Go)
# git clone https://gn.googlesource.com/gn && cd gn && python3 build/gen.py && ninja -C out
```

---

### 2g. LASTCHANGE files

These files record the last git commit timestamp and are required by `gn gen`. They
are not committed to the repository; generate them or copy from an existing build:

```bash
# Generate from the current git state:
cd chromium/src
python3 build/util/lastchange.py -o build/util/LASTCHANGE
python3 build/util/lastchange.py --revision-id-only -o build/util/LASTCHANGE.committime
: > build/util/LASTCHANGE.dummy

# Or copy from an existing build:
cp /path/to/existing/build/util/LASTCHANGE     chromium/src/build/util/
cp /path/to/existing/build/util/LASTCHANGE.committime chromium/src/build/util/
cp /path/to/existing/build/util/LASTCHANGE.dummy      chromium/src/build/util/
```

> **`LASTCHANGE.committime` must be a bare unix integer (corrected 2026-07).** After
> `patches.sh`, the patched HEAD has no Gerrit `Change-Id`, so `lastchange.py` falls back
> and writes `LASTCHANGE=0\nLASTCHANGE_YEAR=1970` into *both* files. `gn gen` then dies in
> `compute_build_timestamp.py` with `ValueError: invalid literal for int()`. Overwrite the
> committime file with the actual HEAD commit time (a single integer, no newline):
>
> ```bash
> printf '%s' "$(git -C chromium/src log -1 --format=%ct HEAD)" \
>   > chromium/src/build/util/LASTCHANGE.committime
> ```

---

### 2h. `esbuild` binary

The DevTools frontend build requires an `esbuild` binary that is not checked in.

> **Usually already present (2026-07).** The dep sync fetches an arm64 `esbuild` cipd
> binary automatically. Check first — `file third_party/devtools-frontend/src/third_party/esbuild/esbuild`
> should report `ELF ... ARM aarch64`. Same for `gn` in §2f
> (`buildtools/linux64/gn` arrives as an arm64 ELF). Only do the manual steps if they're
> actually missing.

If it is missing, copy from an existing build or download it:

```bash
mkdir -p chromium/src/third_party/devtools-frontend/src/third_party/esbuild
cp /path/to/existing/third_party/devtools-frontend/src/third_party/esbuild/esbuild \
   chromium/src/third_party/devtools-frontend/src/third_party/esbuild/esbuild
chmod +x chromium/src/third_party/devtools-frontend/src/third_party/esbuild/esbuild
```

---

### 2i. Node.js `node_modules`

The build requires a `node_modules` tree (terser, typescript, etc.) under
`chromium/src/third_party/node/`. This is not fetched by `gclient sync` for arm64
hosts. Copy from an existing build (63 MB):

```bash
cp -r /path/to/existing/chromium/src/third_party/node/node_modules \
      chromium/src/third_party/node/
```

Alternatively, install them with npm (requires the versions matching Chromium 111):

```bash
cd chromium/src/third_party/node
npm install  # uses the package.json + package-lock.json already present
```

> **After `npm install`, apply `typescript.patch` (corrected 2026-07).** Chromium ships
> `third_party/node/typescript.patch`, which rewrites TypeScript's `lib.dom.d.ts`
> `innerHTML: string` into a `set innerHTML(html: string|TrustedHTML)` setter. A fresh
> `npm install` gives the *unpatched* TypeScript, so the WebUI `tsc` step
> (`//ui/webui/resources:library`) fails ~40 % into the build with
> `TS2322: Type 'TrustedHTML' is not assignable to type 'string'`. This is why copying
> `node_modules` from an existing build (above) is preferred — that copy already has the
> patch applied. If you used `npm install`, apply it manually:
>
> ```bash
> cd chromium/src/third_party/node
> patch -p1 --forward < typescript.patch
> grep -n 'set innerHTML' node_modules/typescript/lib/lib.dom.d.ts   # verify
> ```

---

### 2j. Generated source-tree headers

Several small headers are generated by `lastchange.py` and written directly into the
source tree (not `out/gen/`). They are not committed to the repository. Copy from an
existing build or create them manually:

```bash
# gpu/config/gpu_lists_version.h
cat > chromium/src/gpu/config/gpu_lists_version.h << 'EOF'
/* Generated by lastchange.py, do not edit.*/

#ifndef GPU_CONFIG_GPU_LISTS_VERSION_H_
#define GPU_CONFIG_GPU_LISTS_VERSION_H_

#define GPU_LISTS_VERSION "0"

#endif  // GPU_CONFIG_GPU_LISTS_VERSION_H_
EOF

# skia/ext/skia_commit_hash.h  (hash from chromium/src/DEPS skia_revision entry)
cat > chromium/src/skia/ext/skia_commit_hash.h << 'EOF'
/* Generated by lastchange.py, do not edit.*/

#ifndef SKIA_EXT_SKIA_COMMIT_HASH_H_
#define SKIA_EXT_SKIA_COMMIT_HASH_H_

#define SKIA_COMMIT_HASH "486deb23bc2a4d3d09c66fef52c2ad64d8b4f761"

#endif  // SKIA_EXT_SKIA_COMMIT_HASH_H_
EOF
```

To get the correct Skia hash, look up `skia_revision` in `chromium/src/DEPS`.

---

### 2k. `DAWN_VERSION` file

The Dawn (WebGPU) code generator requires `gpu/webgpu/DAWN_VERSION` — a single-line
file containing the Dawn git hash. It is not generated by `gclient sync` for this
checkout. Copy from an existing build:

```bash
cp /path/to/existing/gpu/webgpu/DAWN_VERSION chromium/src/gpu/webgpu/DAWN_VERSION
```

Or create it manually with the Dawn commit hash from `chromium/src/DEPS`:

```bash
grep -A2 '"dawn_revision"' chromium/src/DEPS
# e.g.: "cb8ce4332c5d6102b8b489186f342ae01d5054be"
# NOTE: printf, not echo — the file must have NO trailing newline (see below).
printf '%s' "cb8ce4332c5d6102b8b489186f342ae01d5054be" > chromium/src/gpu/webgpu/DAWN_VERSION
```

> **`DAWN_VERSION` must have no trailing newline (corrected 2026-07).** Dawn's
> `dawn_version_gen` action embeds the file's bytes *verbatim* into a C++ string literal
> (`static constexpr std::string_view kDawnVersion("…")`). `echo` appends `\n`, which
> lands *inside* the literal and makes it unterminated — the tokenizer then desyncs and
> emits misleading cascade errors in unrelated Dawn headers (`no template named 'RefBase'`,
> `unknown type name 'FeatureInfo'`), failing the build at ~target 14,700/37,107. Write it
> with `printf` (the file should be exactly 40 bytes), and if you already built, remove the
> stale generated header so it regenerates:
>
> ```bash
> printf '%s' "$(grep -oE 'cb8ce[0-9a-f]{35}' chromium/src/DEPS | head -1)" \
>   > chromium/src/gpu/webgpu/DAWN_VERSION
> wc -c chromium/src/gpu/webgpu/DAWN_VERSION   # -> 40
> rm -f chromium/src/gen/third_party/dawn/src/dawn/common/Version_autogen.h
> ```

---

## 3. Build configuration

Create **`chromium/src/out/Default/args.gn`**:

```gn
import("//carbonyl/src/browser/args.gn")

# arm64 native build
target_cpu = "arm64"
host_cpu = "arm64"

# Use system clang (arm64-native; Google's pre-built clang is x86_64 only)
clang_base_path = "/usr/lib/llvm-18"
clang_use_chrome_plugins = false

use_lld = true

# Clang 18 is stricter than Chromium 111's bundled clang 16
treat_warnings_as_errors = false

is_debug = false
symbol_level = 0
is_official_build = false

use_sysroot = true
```

The `import("//carbonyl/src/browser/args.gn")` line pulls in all the Carbonyl-specific
feature flags (disabled PDF, printing, plugins, X11, etc.) from the single canonical
location.

---

## 4. Build

```bash
cd chromium/src
export PATH="$PWD/../depot_tools:$PATH"

# Generate build files
gn gen out/Default

# Compile
ninja -C out/Default headless:headless_shell -j4
```

Use `-j4` on a 16 GB Multipass VM to avoid swap pressure. `-j8` is viable if the host
machine has headroom. ninja resumes from where it stopped after a kill/restart — it
does **not** start from scratch.

Expect ~2–3 hours for a full first build on an M4 Mac with a 16 GB VM at `-j4`.

---

## 5. File download support

Carbonyl has no `DownloadManagerDelegate`, so any URL that would trigger a browser
download (e.g. `Content-Disposition: attachment`) silently does nothing. The following
patches implement one that saves files to `~/Downloads/` and reports progress in the
terminal title bar.

---

### 5a. New file: `carbonyl/src/browser/download_manager_delegate.h`

```cpp
#ifndef CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_
#define CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_

#include "content/public/browser/download_manager_delegate.h"
#include "content/public/browser/web_contents.h"

namespace carbonyl {

class DownloadManagerDelegate : public content::DownloadManagerDelegate {
 public:
  DownloadManagerDelegate();
  ~DownloadManagerDelegate() override;

  void GetNextId(content::DownloadIdCallback callback) override;
  bool DetermineDownloadTarget(
      download::DownloadItem* item,
      content::DownloadTargetCallback* callback) override;
  bool ShouldAutomaticallyOpenFile(const GURL& url,
                                   const base::FilePath& path) override;
  bool ShouldCompleteDownload(download::DownloadItem* item,
                              base::OnceClosure normal_complete) override;
  void CheckDownloadAllowed(
      const content::WebContents::Getter& web_contents_getter,
      const GURL& url,
      const std::string& request_method,
      absl::optional<url::Origin> request_initiator,
      bool from_download_cross_origin_redirect,
      bool content_initiated,
      content::CheckDownloadAllowedCallback check_download_allowed_cb) override;

 private:
  uint32_t next_id_ = 1;
};

}  // namespace carbonyl

#endif  // CARBONYL_SRC_BROWSER_DOWNLOAD_MANAGER_DELEGATE_H_
```

---

### 5b. New file: `carbonyl/src/browser/download_manager_delegate.cc`

```cpp
#include "carbonyl/src/browser/download_manager_delegate.h"

#include <string>

#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/path_service.h"
#include "base/threading/thread_restrictions.h"
#include "carbonyl/src/browser/renderer.h"
#include "components/download/public/common/download_danger_type.h"
#include "components/download/public/common/download_interrupt_reasons.h"
#include "components/download/public/common/download_item.h"
#include "content/public/browser/download_item_utils.h"
#include "content/public/browser/web_contents.h"
#include "net/base/filename_util.h"
#include "url/origin.h"

namespace carbonyl {

DownloadManagerDelegate::DownloadManagerDelegate() = default;
DownloadManagerDelegate::~DownloadManagerDelegate() = default;

void DownloadManagerDelegate::GetNextId(content::DownloadIdCallback callback) {
  std::move(callback).Run(next_id_++);
}

bool DownloadManagerDelegate::DetermineDownloadTarget(
    download::DownloadItem* item,
    content::DownloadTargetCallback* callback) {
  base::ScopedAllowBlocking allow_blocking;

  base::FilePath downloads_dir;
  base::PathService::Get(base::DIR_HOME, &downloads_dir);
  downloads_dir = downloads_dir.AppendASCII("Downloads");
  base::CreateDirectory(downloads_dir);

  std::string suggested = item->GetSuggestedFilename();
  base::FilePath filename;

  if (!suggested.empty()) {
    filename = base::FilePath::FromUTF8Unsafe(suggested);
  } else {
    filename = net::GenerateFileName(
        item->GetURL(),
        item->GetContentDisposition(),
        std::string(),
        std::string(),
        item->GetMimeType(),
        "download");
  }

  filename = filename.BaseName();
  if (filename.empty() || filename.value() == ".") {
    filename = base::FilePath::FromUTF8Unsafe("download");
  }

  base::FilePath target_path = downloads_dir.Append(filename);

  // Collision avoidance: append (1), (2), etc.
  if (base::PathExists(target_path)) {
    std::string stem = filename.RemoveExtension().value();
    std::string ext = filename.Extension();
    for (int i = 1; i < 1000; ++i) {
      base::FilePath candidate = downloads_dir.AppendASCII(
          stem + " (" + std::to_string(i) + ")" + ext);
      if (!base::PathExists(candidate)) {
        target_path = candidate;
        break;
      }
    }
  }

  auto* renderer = Renderer::GetCurrent();
  if (renderer) {
    renderer->NotifyDownload(target_path.value(), -1LL);
  }

  base::FilePath intermediate_path =
      target_path.AddExtension(FILE_PATH_LITERAL(".crdownload"));

  std::move(*callback).Run(
      target_path,
      download::DownloadItem::TARGET_DISPOSITION_OVERWRITE,
      download::DOWNLOAD_DANGER_TYPE_NOT_DANGEROUS,
      download::DownloadItem::InsecureDownloadStatus::SAFE,
      intermediate_path,
      base::FilePath(),
      item->GetMimeType(),
      download::DOWNLOAD_INTERRUPT_REASON_NONE);

  return true;
}

bool DownloadManagerDelegate::ShouldAutomaticallyOpenFile(
    const GURL& url,
    const base::FilePath& path) {
  return false;
}

bool DownloadManagerDelegate::ShouldCompleteDownload(
    download::DownloadItem* item,
    base::OnceClosure normal_complete) {
  auto* renderer = Renderer::GetCurrent();
  if (renderer) {
    renderer->NotifyDownload(item->GetTargetFilePath().value(),
                             item->GetReceivedBytes());
  }
  return true;
}

// Bypass the default async WebContents-validity check, which can race against
// the navigation-to-download interception and silently drop downloads.
void DownloadManagerDelegate::CheckDownloadAllowed(
    const content::WebContents::Getter& web_contents_getter,
    const GURL& url,
    const std::string& request_method,
    absl::optional<url::Origin> request_initiator,
    bool from_download_cross_origin_redirect,
    bool content_initiated,
    content::CheckDownloadAllowedCallback check_download_allowed_cb) {
  std::move(check_download_allowed_cb).Run(true);
}

}  // namespace carbonyl
```

---

### 5c. Modified: `carbonyl/src/browser/BUILD.gn`

Add a new `downloads` component at the end of the existing file:

```gn
component("downloads") {
  output_name = "carbonyl_downloads"
  sources = [
    "download_manager_delegate.cc",
    "download_manager_delegate.h",
  ]
  deps = [
    ":renderer",
    "//base",
    "//components/download/public/common:public",
    "//content/public/browser",
    "//net",
  ]
}
```

---

### 5d. Modified: `carbonyl/src/browser/renderer.h`

Add the `NotifyDownload` method declaration to the `Renderer` class (public section):

```cpp
void NotifyDownload(const std::string& path, long long bytes);
```

---

### 5e. Modified: `carbonyl/src/browser/renderer.cc`

Add the extern C declaration for the Rust FFI function (in the `extern "C"` block
alongside the other `carbonyl_renderer_*` declarations):

```cpp
void carbonyl_renderer_notify_download(
    struct carbonyl_renderer* renderer,
    const char* path,
    long long bytes
);
```

Add the method implementation (in the `Renderer` class body):

```cpp
void Renderer::NotifyDownload(const std::string& path, long long bytes) {
    if (!path.size()) {
        return;
    }
    carbonyl_renderer_notify_download(ptr_, path.c_str(), bytes);
}
```

---

### 5f. Modified: `src/browser/bridge.rs`

Add the `carbonyl_renderer_notify_download` export to the Rust library, after the
existing `carbonyl_renderer_set_title` function:

```rust
#[no_mangle]
pub extern "C" fn carbonyl_renderer_notify_download(
    bridge: RendererPtr,
    path: *const c_char,
    bytes: i64,
) {
    let (bridge, path) = unsafe { (bridge.as_ref(), CStr::from_ptr(path)) };
    let (mut bridge, path) = (bridge.unwrap().lock().unwrap(), path.to_owned());

    bridge.renderer.render(move |renderer| {
        let filename = path
            .to_str()
            .unwrap_or("unknown")
            .rsplit('/')
            .next()
            .unwrap_or("download");

        let msg = if bytes >= 0 {
            format!("Downloaded: {} ({} bytes)", filename, bytes)
        } else {
            format!("Downloading: {}", filename)
        };

        renderer.set_title(&msg).unwrap_or(());
    });
}
```

---

### 5g. Modified: `chromium/src/headless/BUILD.gn`

Add the downloads dependency to the `headless_shell_lib` deps list:

```gn
"//carbonyl/src/browser:downloads",
```

---

### 5h. Modified: `chromium/src/headless/lib/browser/headless_browser_context_impl.h`

Add the include alongside other headless includes:

```cpp
#include "carbonyl/src/browser/download_manager_delegate.h"
```

In the private section, add the delegate member as a **raw pointer** (not `unique_ptr`):

```cpp
carbonyl::DownloadManagerDelegate* download_manager_delegate_ = nullptr;
```

**Why raw pointer?** C++ destroys derived-class members before base-class destructors
run. `HeadlessBrowserContextImpl`'s `download_manager_delegate_` member would be
destroyed before `BrowserContextImpl::~BrowserContextImpl()` runs. That destructor
calls `DownloadManagerImpl::Shutdown()`, which calls `delegate_->Shutdown()` — a
use-after-free through a corrupted vtable (SEGV at a garbage address like
`0x303c0044000000`). Using a raw pointer (intentional leak — the delegate is a tiny
object and the process is about to exit) keeps it alive through the full shutdown
sequence.

---

### 5i. Modified: `chromium/src/headless/lib/browser/headless_browser_context_impl.cc`

Add the include at the top:

```cpp
#include "carbonyl/src/browser/download_manager_delegate.h"
```

Add the override method:

```cpp
content::DownloadManagerDelegate*
HeadlessBrowserContextImpl::GetDownloadManagerDelegate() {
  if (!download_manager_delegate_) {
    download_manager_delegate_ = new carbonyl::DownloadManagerDelegate();
  }
  return download_manager_delegate_;
}
```

Also add the declaration to the class in the header (`headless_browser_context_impl.h`
public section):

```cpp
content::DownloadManagerDelegate* GetDownloadManagerDelegate() override;
```

---

### 5j. Modified: `chromium/src/base/threading/thread_restrictions.h`

`DetermineDownloadTarget` calls `base::CreateDirectory` and `base::PathExists` — both
are blocking filesystem operations. Chromium's browser UI thread disallows blocking
by default; calling them triggers a `FATAL` CHECK. `ScopedAllowBlocking` is the
standard mechanism to temporarily permit this, but its constructor is private —
only explicitly-named `friend` classes may instantiate it.

Add a forward declaration near line 137, alongside the other forward declarations:

```cpp
namespace carbonyl {
class DownloadManagerDelegate;
}  // namespace carbonyl
```

Add a friend declaration inside `class ScopedAllowBlocking`'s private section,
alongside the other `friend class` entries:

```cpp
friend class carbonyl::DownloadManagerDelegate;
```

> **Warning:** `thread_restrictions.h` is included by virtually every Chromium
> translation unit. Touching it invalidates ~3,800 compiled objects and triggers a
> full recompile — expect 2–3 hours on this hardware.

---

## 6. Running

Two runtime prerequisites (added 2026-07), both one-time:

**a. Put `libcarbonyl.so` next to the binary.** `headless_shell` has `RPATH=$ORIGIN/.` and
`NEEDED libcarbonyl.so`, so the `.so` (built by `cargo build --release` into
`carbonyl-src/build/release/`) must sit in the same dir, or you get
`error while loading shared libraries: libcarbonyl.so: cannot open shared object file`:

```bash
cp ~/carbonyl/carbonyl-src/build/release/libcarbonyl.so \
   ~/carbonyl/carbonyl-src/chromium/src/out/Default/
```

**b. Install runtime shared libraries.** The binary is built against the Bullseye sysroot
but runs on the host, which lacks several libs (first missing one is usually
`libasound.so.2`):

```bash
sudo apt-get install -y \
  libasound2t64 libnss3 libnspr4 libatk1.0-0t64 libatk-bridge2.0-0t64 \
  libcups2t64 libdrm2 libgbm1 libxkbcommon0 libxcomposite1 libxdamage1 \
  libxrandr2 libxfixes3 libxext6 libx11-6 libpangocairo-1.0-0 libcairo2 \
  libatspi2.0-0t64 libxcb1 libxrender1
```

Then run:

```bash
cd ~/carbonyl/carbonyl-src/chromium/src/out/Default
./headless_shell --version                                    # -> Carbonyl 0.0.3
./headless_shell --no-sandbox [--ignore-certificate-errors] <url>
```

`--no-sandbox` is required inside a container/VM without a usable sandbox (otherwise
`FATAL:zygote_host_impl_linux.cc … No usable sandbox!`). A convenience launcher on `PATH`
keeps this hands-off:

```bash
sudo tee /usr/local/bin/carbonyl > /dev/null << 'EOF'
#!/usr/bin/env bash
BIN="$HOME/carbonyl/carbonyl-src/chromium/src/out/Default"
[ -e "$BIN/libcarbonyl.so" ] || cp "$HOME/carbonyl/carbonyl-src/build/release/libcarbonyl.so" "$BIN/"
exec "$BIN/headless_shell" --no-sandbox "$@"
EOF
sudo chmod +x /usr/local/bin/carbonyl
# now just: carbonyl https://example.com
```

Downloaded files appear in `~/Downloads/`. During download the terminal title bar
shows `Downloading: <filename>`; on completion it shows `Downloaded: <filename> (N bytes)`.

To trigger a download, navigate to a URL that serves `Content-Disposition: attachment`,
or use curl to verify the URL works first:

```bash
curl -OJ 'https://example.com/file.tar.gz'
```

---

## 7. Troubleshooting

### `ninja: error: loading 'build.ninja': No such file or directory`

Run `gn gen out/Default` first. `build.ninja` is generated by `gn`, not checked in.

### `Missing sysroot (//build/linux/debian_bullseye_arm64-sysroot)`

Run:
```bash
python3 build/linux/sysroot_scripts/install-sysroot.py --arch=arm64
```

### `FileNotFoundError: build/util/LASTCHANGE`

Generate or copy the LASTCHANGE files (see section 2g).

### `ninja: error: '...esbuild', needed by '...', missing and no known rule to make it`

Copy or download the `esbuild` binary (see section 2h).

### `gn.py: Could not find gn executable`

Copy or build the `gn` binary (see section 2f).

### SEGV in `DownloadManagerImpl::Shutdown()` (vtable corruption)

The delegate member must be a **raw pointer**, not `unique_ptr`. See section 5h.

### `Cannot find module '...terser/bin/terser'`

The `node_modules` tree is missing. Copy it from an existing build or `npm install`
inside `chromium/src/third_party/node/` (see section 2i).

### `fatal error: 'gpu/config/gpu_lists_version.h' file not found`

Create the file manually (see section 2j) — it is a generated header not checked in.

### `fatal error: 'skia/ext/skia_commit_hash.h' file not found`

Copy from an existing build or create manually (see section 2j for the pattern — same
approach, different path and hash constant).

### `FileNotFoundError: gpu/webgpu/DAWN_VERSION`

Copy the file from an existing build, or extract the Dawn commit hash from `DEPS` and
write it manually (see section 2k).

### `--allow-shlib-undefined` link error

`libcarbonyl.so` references glibc symbols newer than the Bullseye sysroot. Add
`"-Wl,--allow-shlib-undefined"` to the headless_shell ldflags (see section 2d).

### `ImportError: cannot import name 'StrEnum' from 'enum'` (gclient)

depot_tools is too new for Python 3.10. Pin it to a Chromium-111-era commit and set
`DEPOT_TOOLS_UPDATE=0` (see section 1).

### `no such package: infra/rbe/client/linux-arm64` (gclient sync)

reclient has no arm64 cipd package and `custom_deps: None` doesn't suppress it. Give the
`src/buildtools/reclient` block in `chromium/src/DEPS` an always-false `condition` and
re-sync with `--nohooks` (see section 1).

### `cd: chromium/src: No such file or directory` (patches.sh)

You ran `patches.sh apply` before `gclient sync`. Sync first (the source tree must exist),
then apply patches (see section 1).

### `ValueError: invalid literal for int()` in `compute_build_timestamp.py` (gn gen)

`build/util/LASTCHANGE.committime` holds the `LASTCHANGE=0…` fallback blob instead of a
bare integer. Overwrite it with `git log -1 --format=%ct HEAD` (see section 2g).

### `TS2322: Type 'TrustedHTML' is not assignable to type 'string'` (tsc / ui:library)

You used `npm install` and didn't apply `third_party/node/typescript.patch`. Apply it, then
resume the build (see section 2i).

### Dawn: `no template named 'RefBase'` / `unknown type name 'FeatureInfo'`

Misleading cascade from a malformed `gpu/webgpu/DAWN_VERSION` — it has a trailing newline
that breaks the generated `kDawnVersion` string literal. Rewrite it with `printf` (no
newline), delete the stale `Version_autogen.h`, and resume (see section 2k).

### `error while loading shared libraries: libcarbonyl.so` (at runtime)

Copy `build/release/libcarbonyl.so` next to `headless_shell` — its RPATH is `$ORIGIN/.`
(see section 6a).

### `error while loading shared libraries: libasound.so.2` (or other lib) at runtime

Install the runtime shared libraries the host is missing (see section 6b).
