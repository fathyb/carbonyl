<table align="center">
  <tbody>
    <tr>
      <td>
        <p></p>
        <pre>
   O    O
    \  /
O —— Cr —— O
    /  \
   O    O</pre>
      </td>
      <td><h1>Carbonyl</h1></td>
    </tr>
  </tbody>
</table>

Carbonyl is a Chromium based browser built to run in a terminal. [Read the blog post](https://fathy.fr/carbonyl).

It supports pretty much all Web APIs including WebGL, WebGPU, audio and video playback, animations, etc..

It's snappy, starts in less than a second, runs at 60 FPS, and idles at 0% CPU usage. It does not require a window server (i.e. works in a safe-mode console), and even runs through SSH.

Carbonyl originally started as [`html2svg`](https://github.com/fathyb/html2svg) and is now the runtime behind it.

## Usage

> Carbonyl on Linux without Docker requires the same dependencies as Chromium.

### Docker

```shell
$ docker run --rm -ti fathyb/carbonyl https://youtube.com
```

### npm

```console
$ npm install --global carbonyl
$ carbonyl https://github.com
```

### Binaries

- [macOS amd64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.macos-amd64.zip)
- [macOS arm64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.macos-arm64.zip)
- [Linux amd64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.linux-amd64.zip)
- [Linux arm64](https://github.com/fathyb/carbonyl/releases/download/v0.0.3/carbonyl.linux-arm64.zip)

## Demo

<table>
  <tbody>
    <tr>
      <td>
        <video src="https://user-images.githubusercontent.com/5746414/213682926-f1cc2de7-a38c-4125-9257-92faecfc7e24.mp4">
      </td>
      <td>
        <video src="https://user-images.githubusercontent.com/5746414/213682913-398d3d11-1af8-4ae6-a0cd-a7f878efd88b.mp4">
      </td>
    </tr>
    <tr>
      <td colspan="2">
        <video src="https://user-images.githubusercontent.com/5746414/213682918-d6396a4f-ee23-431d-828e-4ad6a00e690e.mp4">
      </td>
    </tr>
  </tbody>
</table>

## Known issues

- Fullscreen mode not supported yet

## Comparisons

### Lynx

Lynx is the original terminal web browser, and the oldest one still maintained.

#### Pros

- When it understands a page, Lynx has the best layout, fully optimized for the terminal

#### Cons

> Some might sound like pluses, but Browsh and Carbonyl let you disable most of those if you'd like

- Does not support a lot of modern web standards
- Cannot run JavaScript/WebAssembly
- Cannot view or play media (audio, video, DOOM)

### Browsh

Browsh is the original "normal browser into a terminal" project. It starts Firefox in headless mode and connects to it through an automation protocol.

#### Pro

- It's easier to update the underlying browser: just update Firefox
- This makes development easier: just install Firefox and compile the Go code in a few seconds
- As of today, Browsh supports extensions while Carbonyl doesn't, although it's on our roadmap

#### Cons

- It runs slower and requires more resources than Carbonyl. 50x more CPU power is needed for the same content in average, that's because Carbonyl does not downscale or copy the window framebuffer, it natively renders to the terminal resolution.
- It uses custom stylesheets to fix the layout, which is less reliable than Carbonyl's changes to its HTML engine (Blink).

## Operating System Support

As far as tested, the operating systems under are supported:

- Linux (Debian, Ubuntu and Arch tested)
- MacOS
- Windows 11 and WSL

## Contributing

Carbonyl is split in two parts: the "core" which is built into a shared library (`libcarbonyl`), and the "runtime" which dynamically loads the core (`carbonyl` executable).

The core is written in Rust and takes a few seconds to build from scratch. The runtime is a modified version of the Chromium headless shell and takes more than an hour to build from scratch.

If you're just making changes to the Rust code, build `libcarbonyl` and replace it in a release version of Carbonyl.

### Core

```console
$ cargo build
```

### Runtime

Few notes:

- Building the runtime is almost the same as building Chromium with extra steps to patch and bundle the Rust library. Scripts in the `scripts/` directory are simple wrappers around `gn`, `ninja`, etc..
- Building Chromium for arm64 on Linux requires an amd64 processor
- Carbonyl is only tested on Linux and macOS, other platforms likely require code changes to Chromium
- Chromium is huge and takes a long time to build, making your computer mostly unresponsive. An 8-core CPU such as an M1 Max or an i9 9900k with 10 Gbps fiber takes around ~1 hour to fetch and build. It requires around 100 GB of disk space.

#### Fetch

> Fetch Chromium's code.

```console
$ ./scripts/gclient.sh sync
```

#### Apply patches

> Any changes made to Chromium will be reverted, make sure to save any changes you made.

```console
$ ./scripts/patches.sh apply
```

#### Configure

```console
$ ./scripts/gn.sh args out/Default
```

> `Default` is the target name, you can use multiple ones and pick any name you'd like, i.e.:
>
> ```console
> $ ./scripts/gn.sh args out/release
> $ ./scripts/gn.sh args out/debug
> # or if you'd like to build a multi-platform image
> $ ./scripts/gn.sh args out/arm64
> $ ./scripts/gn.sh args out/amd64
> ```

When prompted, enter the following arguments:

```gn
import("//carbonyl/src/browser/args.gn")

# uncomment this to build for arm64
# target_cpu = "arm64"

# comment this to disable ccache
cc_wrapper = "env CCACHE_SLOPPINESS=time_macros ccache"

# comment this for a debug build
is_debug = false
symbol_level = 0
is_official_build = true
```

#### Build binaries

```console
$ ./scripts/build.sh Default
```

This should produce the following outputs:

- `out/Default/headless_shell`: browser binary
- `out/Default/icudtl.dat`
- `out/Default/libEGL.so`
- `out/Default/libGLESv2.so`
- `out/Default/v8_context_snapshot.bin`

#### Build Docker image

```console
# Build arm64 Docker image using binaries from the Default target
$ ./scripts/docker-build.sh Default arm64
# Build amd64 Docker image using binaries from the Default target
$ ./scripts/docker-build.sh Default amd64
```

#### Run

```
$ ./scripts/run.sh Default https://wikipedia.org
```


## 🌐 Web Resources & Interactive Index
- [ABOUT A FROG](https://quizzesarena.onrender.com/about-a-frog.html)
- [CATEGORY INTERSTELLAR](https://eduquestkr.pages.dev/category-interstellar.html)
- [INDEX19](https://brainquests.pages.dev/index19.html)
- [CATEGORY CASUAL969](https://eduquests.onrender.com/category-casual969.html)
- [ENERGY CLICKER](https://eduquestspt.pages.dev/energy-clicker.html)
- [CATEGORY SHOOTER](https://eduquests.github.io/category-shooter.html)
- [KNIT BEARS](https://eduquests.pages.dev/knit-bears.html)
- [CATEGORY RACING DRIVING 2](https://eduquestses.pages.dev/category-racing-driving-2.html)
- [PLANET MERGE](https://eduquestspt.pages.dev/planet-merge.html)
- [CATEGORY SCRATCH17](https://eduquests.onrender.com/category-scratch17.html)
- [HEAD JUMP](https://brainquests.pages.dev/head-jump.html)
- [MAHJONG MAGIC ISLANDS](https://eduquestspt.pages.dev/mahjong-magic-islands.html)
- [JENNYS MATH PUZZLE](https://eduquestspt.pages.dev/jennys-math-puzzle.html)
- [CATEGORY WEBGAME](https://eduquests.github.io/category-webgame.html)
- [ANIMAL BUS TRAFFIC JAM](https://eduquests.github.io/animal-bus-traffic-jam.html)
- [CATEGORY FPS](https://eduquests.onrender.com/category-fps.html)
- [HEAD JUMP](https://eduquests.github.io/head-jump.html)
- [M5 CITY DRIVER](https://eduquestspt.pages.dev/m5-city-driver.html)
- [CATEGORY FPS175](https://eduquests.onrender.com/category-fps175.html)
- [FARMING LIFE](https://welearnaction.onrender.com/farming-life.html)
- [2048 BLOCKS DESTRUCTION](https://eduquests.github.io/2048-blocks-destruction.html)
- [PUMPKING VS MUMMY](https://eduquestsfr.pages.dev/pumpking-vs-mummy.html)
- [TOCO TEENS HALLOWEEN PARTY](https://brainquests.pages.dev/toco-teens-halloween-party.html)
- [CATEGORY BALL175](https://welearnaction.onrender.com/category-ball175.html)
- [DESTINATION BRAIN TEST](https://eduquestspt.pages.dev/destination-brain-test.html)
- [INDEX28](https://eduquestsfr.pages.dev/index28.html)
- [MAKEUP STUDIO GLAM DIVA](https://eduquestsfr.pages.dev/makeup-studio-glam-diva.html)
- [ROPE KING](https://eduquests.netlify.app/rope-king.html)
- [CATEGORY MERGE224](https://eduquestsfr.pages.dev/category-merge224.html)
- [MIGHTY RUN](https://eduquestsfr.pages.dev/mighty-run.html)
- [FOOTBALL FUN](https://learnaction.netlify.app/football-fun.html)
- [BUBBLE SHOOTER LEGEND](https://eduquests.netlify.app/bubble-shooter-legend.html)
- [STALKER STRIKE](https://eduquestsfr.pages.dev/stalker-strike.html)
- [CATEGORY TURN BASED30](https://learnaction.netlify.app/category-turn-based30.html)
- [FLIGHT PILOT AIRPLANE GAMES 24](https://eduquests.pages.dev/flight-pilot-airplane-games-24.html)
- [AUTUMN GLAM GALA](https://eduquests.onrender.com/autumn-glam-gala.html)
- [GLOVES GROW RUSH](https://brainquests.pages.dev/gloves-grow-rush.html)
- [MUSIC RUSH](https://eduquests.github.io/music-rush.html)
- [CATEGORY OBSTACLE299](https://eduquests.github.io/category-obstacle299.html)
- [DREAMY JEWEL](https://eduquests.pages.dev/dreamy-jewel.html)
- [CATEGORY MOBILE2 097](https://brainquests.pages.dev/category-mobile2-097.html)
- [DAZZLING DIVAS MAKEUP](https://brainquests.pages.dev/dazzling-divas-makeup.html)
- [CATEGORY MERGE](https://eduquests.onrender.com/category-merge.html)
- [CATEGORY COOKING46](https://eduquestses.pages.dev/category-cooking46.html)
- [CATEGORY MAKEUP51](https://eduquestsfr.pages.dev/category-makeup51.html)
- [FIND IT FIND THE DIFFERENCES](https://eduquests.github.io/find-it-find-the-differences.html)
- [ELLIE AND BEN CHRISTMAS EVE](https://learnaction.netlify.app/ellie-and-ben-christmas-eve.html)
- [ADDICTION MINI SOLITAIRE](https://eduquests.pages.dev/addiction-mini-solitaire.html)
- [ZOMBIE RUSH](https://eduquests.pages.dev/zombie-rush.html)
- [CATEGORY MINECRAFT 2](https://eduquestsfr.pages.dev/category-minecraft-2.html)
- [CATEGORY PHYSICS371](https://eduquestsfr.pages.dev/category-physics371.html)
- [INDEX34](https://eduquestsfr.pages.dev/index34.html)
- [HERO STORY MONSTERS CROSSING](https://welearnaction.onrender.com/hero-story-monsters-crossing.html)
- [DRAW TO FISH FIGHT](https://eduquestsfr.pages.dev/draw-to-fish-fight.html)
- [FIND 6 DIFFERENCES SPOT THE HIDDEN CHANGES](https://eduquests.onrender.com/find-6-differences-spot-the-hidden-changes.html)
- [CATEGORY MINECRAFT 3](https://eduquestsfr.pages.dev/category-minecraft-3.html)
- [ROBBY BOMBERMAN](https://eduquestspt.pages.dev/robby-bomberman.html)
- [SOLITAIRE KLONDIKE ETERNAL RUSSIAN CLASSIC](https://brainquests.pages.dev/solitaire-klondike-eternal-russian-classic.html)
- [DRIVERZ ED](https://eduquests.github.io/driverz-ed.html)
- [GREATSWORD V3](https://eduquestsfr.pages.dev/greatsword-v3.html)
- [ISLAND EXPANDER](https://learnaction.netlify.app/island-expander.html)
- [FOOD TRUCK CHEF](https://eduquestsfr.pages.dev/food-truck-chef.html)
- [CATEGORY FLASH](https://learnaction.netlify.app/category-flash.html)
- [CHAIN PUZZLE](https://learnaction.netlify.app/chain-puzzle.html)
- [CATEGORY BLOODY29](https://eduquests.github.io/category-bloody29.html)
- [ITALIAN BRAINROT SURVIVE PARKOUR](https://eduquests.github.io/italian-brainrot-survive-parkour.html)
- [DRIVE AHEAD SPORTS](https://eduquests.pages.dev/drive-ahead-sports.html)
- [HIDDEN OBJECT STREET OF SECRETS](https://welearnaction.onrender.com/hidden-object-street-of-secrets.html)
- [TIC TAC TOE MATCH THREE](https://learnaction.netlify.app/tic-tac-toe-match-three.html)
- [ROBBY BOMBERMAN](https://brainquests.pages.dev/robby-bomberman.html)
- [CATEGORY FPS175](https://brainquests.pages.dev/category-fps175.html)
- [BEGGAR CLICKER](https://eduquestsfr.pages.dev/beggar-clicker.html)
- [INDEX37](https://eduquestkr.pages.dev/index37.html)
- [FINGER SOCCER TOURNAMENT](https://eduquestspt.pages.dev/finger-soccer-tournament.html)
- [CRAZY GOOSE SIMULATOR](https://eduquests.pages.dev/crazy-goose-simulator.html)
- [OBBY TOWER](https://eduquests.github.io/obby-tower.html)
- [POLYGON SPACE](https://eduquestsfr.pages.dev/polygon-space.html)
- [SUPER RACING](https://eduquests.netlify.app/super-racing.html)
- [CONSTRUCTION TRUCK BUILDING GAMES FOR KIDS](https://eduquestses.pages.dev/construction-truck-building-games-for-kids.html)
- [COLOR RING SORTING MATCH](https://eduquestspt.pages.dev/color-ring-sorting-match.html)
- [HUNT AND SEEK](https://eduquestses.pages.dev/hunt-and-seek.html)
- [PET ME MAZE](https://eduquests.github.io/pet-me-maze.html)
- [CATEGORY LOGIC538](https://eduquests.github.io/category-logic538.html)
- [DINO GAME](https://eduquestsfr.pages.dev/dino-game.html)
- [CRAB GUARDS](https://eduquests.github.io/crab-guards.html)
- [INDEX36](https://eduquestspt.pages.dev/index36.html)
- [COLOR BRAIN TEST GAMES](https://learnaction.netlify.app/color-brain-test-games.html)
- [9 BLOCKS](https://eduquests.github.io/9-blocks.html)
- [CATEGORY INCREMENTAL388](https://eduquestkr.pages.dev/category-incremental388.html)
- [CATEGORY FLASH 2](https://eduquestkr.pages.dev/category-flash-2.html)
- [POLICE TRAFFIC RACER](https://eduquestkr.pages.dev/police-traffic-racer.html)
- [WORD HUNT](https://eduquests.netlify.app/word-hunt.html)
- [GOLD MINER CLASSIC](https://eduquests.github.io/gold-miner-classic.html)
- [CATEGORY MINECRAFT81](https://brainquests.pages.dev/category-minecraft81.html)
- [CITY BANANA MAN AGENT](https://eduquestkr.pages.dev/city-banana-man-agent.html)
- [TERMS](https://brainquests.netlify.app/terms.html)
- [JEWEL DRESS UP](https://welearnaction.onrender.com/jewel-dress-up.html)
- [TATTOO MASTER](https://eduquests.netlify.app/tattoo-master.html)
- [MAJESTIC DRAGONS MERGE](https://eduquests.github.io/majestic-dragons-merge.html)
- [HEXA PUZZLE](https://eduquests.onrender.com/hexa-puzzle.html)
- [CHECKERS DELUXE EDITION](https://eduquestspt.pages.dev/checkers-deluxe-edition.html)
- [CATEGORY IDLE445](https://eduquestsfr.pages.dev/category-idle445.html)
- [CATEGORY PARTY23](https://eduquests.onrender.com/category-party23.html)
- [STACK TOWER PRO](https://welearnaction.onrender.com/stack-tower-pro.html)
- [ROUGH BALL](https://brainquests.pages.dev/rough-ball.html)
- [TAILOR STYLIST FASHION DIARY](https://eduquests.onrender.com/tailor-stylist-fashion-diary.html)
- [TWO STUNT RACERS](https://eduquestsfr.pages.dev/two-stunt-racers.html)
- [CATEGORY EDUCATIONAL](https://learnaction.netlify.app/category-educational.html)
- [KING OF CRABS](https://eduquests.netlify.app/king-of-crabs.html)
- [INDEX22](https://brainquests.pages.dev/index22.html)
- [TANGLED SNAKES SORT PUZZLE](https://eduquests.pages.dev/tangled-snakes-sort-puzzle.html)
- [DRAW WAR](https://eduquests.pages.dev/draw-war.html)
- [ANIMAL BASKETBALL](https://brainquests.pages.dev/animal-basketball.html)
- [SHOOT THE BOTTLE](https://ieduquests.web.app/shoot-the-bottle.html)
- [SUMMER TRIPLE MAHJONG](https://welearnaction.onrender.com/summer-triple-mahjong.html)
- [CATEGORY ADVENTURE 3](https://brainquests.pages.dev/category-adventure-3.html)
- [LITTLE LILY HALLOWEEN PREP](https://eduquestsfr.pages.dev/little-lily-halloween-prep.html)
- [MY PERFECT ORGANIZATION](https://learnaction.netlify.app/my-perfect-organization.html)
- [BEAT MUSIC BATTLE](https://eduquests.pages.dev/beat-music-battle.html)
- [CATEGORY MERGE](https://ieduquests.web.app/category-merge.html)
- [CATEGORY SURVIVAL](https://eduquestkr.pages.dev/category-survival.html)
- [UNSCREW WOOD PUZZLE](https://eduquestspt.pages.dev/unscrew-wood-puzzle.html)
- [FRIDAY NIGHT SPRUNKI](https://eduquestsfr.pages.dev/friday-night-sprunki.html)
- [GANG WAR STRIKE SHOOTER](https://eduquests.netlify.app/gang-war-strike-shooter.html)
- [CATEGORY LINKS](https://eduquestkr.pages.dev/category-links.html)
- [CATEGORY CUTE](https://eduquestses.pages.dev/category-cute.html)
- [CATEGORY MAKEUP](https://eduquestkr.pages.dev/category-makeup.html)
- [CATEGORY TOP DOWN251](https://eduquestses.pages.dev/category-top-down251.html)
- [GRID BLAST](https://eduquestspt.pages.dev/grid-blast.html)
- [WACKY WHEELS](https://eduquests.netlify.app/wacky-wheels.html)
