<p align="center">
  <img alt="Scaleri" src="/program_info/io.github.tacobellerontop_sudo.Scaleri.logo.svg" width="50%">
</p>

<p align="center">
  Scaleri is a modern custom launcher for Minecraft that lets you easily manage multiple installations of Minecraft at once.<br />
  <br />Scaleri is built on <a href="https://github.com/PrismLauncher/PrismLauncher">Prism Launcher</a> and is <b>not</b> endorsed by the Prism Launcher or MultiMC projects.
</p>

<p align="center">
  <img alt="Scaleri home screen" src="/docs/screenshots/main-window.png" width="85%">
</p>
<p align="center">
  <img alt="Scaleri settings" src="/docs/screenshots/settings.png" width="42%">
  <img alt="Scaleri instance editor" src="/docs/screenshots/edit-instance.png" width="42%">
</p>

## Scaleri Client

Scaleri comes with **Scaleri Client**, a casual PvP and HUD mod for Minecraft 1.21.1 (Fabric), built in. There is
nothing to download:

- On first start, Scaleri creates a **Scaleri Client 1.21.1** profile (Minecraft 1.21.1 with Fabric Loader) and selects it,
  so the first Play starts the game with the mod.
- Before every launch, Scaleri copies the version of the mod that came with the launcher into the profile's `mods`
  folder if it changed, so updating Scaleri updates the mod. Fabric API, which the mod needs, is downloaded from Modrinth
  the first time. Your Scaleri Client settings (`config/scaleri-client.json`) are never touched.
- Any other Minecraft 1.21.1 Fabric profile can use it too: tick **Scaleri Client** under the Play button on *Home*.
  Unticking it removes the mod from that profile. If you delete the Scaleri Client profile, *Home* offers to add it back.

The mod's source is in [`client/`](client/README.md). Launcher builds bundle it as `jars/scaleri-client.jar` (see
[Building](#building)).

## What's different from Prism Launcher

- **New home screen.** A sidebar for adding instances, settings, folders, help and accounts; a hero banner with a
  pixel-art landscape showing the selected instance and a big green Play button; and a searchable instance library.
- **Redesigned windows.** Settings, Edit Instance, New Instance and the mod browser share a new layout: a dark sidebar
  with a green marker on the current page, and a header bar with the page's icon and title.
- **New look.** A built-in *Scaleri* theme, on by default: deepslate greys, square corners, bevelled block buttons,
  grass-green accents, an XP-bar progress bar and Minecraft-style purple tooltips. Game logs use Minecraft's chat colours.
  The other Prism themes are still available under *Settings → Appearance*.
- **New identity.** Scaleri has its own name, pixel-art logo and app ID (`io.github.tacobellerontop_sudo.Scaleri`),
  and keeps its data in its own folder, so it can be installed side by side with Prism Launcher.
- **Scaleri Client built in.** See [above](#scaleri-client).
- **Your own API keys.** Scaleri does not reuse Prism Launcher's Microsoft, CurseForge or Imgur keys (see below).

Everything else (instances, mod loaders, modpacks, Modrinth/CurseForge browsing, Java management) works exactly as in
Prism Launcher 11.1.1, which Scaleri is based on. Instances exported from Scaleri, Prism Launcher, PolyMC or MultiMC can be
imported into each other.

## Building

Scaleri builds exactly like Prism Launcher; follow the
[Prism Launcher build instructions](https://prismlauncher.org/wiki/development/build-instructions). For example, on Ubuntu 24.04:

```sh
sudo apt install cmake ninja-build extra-cmake-modules openjdk-17-jdk-headless cmark libcmark-dev gamemode-dev \
  qt6-base-dev qt6-base-private-dev qt6-5compat-dev qt6-networkauth-dev qt6-svg-dev qt6-tools-dev qt6-image-formats-plugins \
  libqt6opengl6-dev libgl-dev libqrencode-dev libarchive-dev libtomlplusplus-dev zlib1g-dev scdoc

cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=install \
  -DLauncher_BUILD_SCALERI_CLIENT=ON
cmake --build build
cmake --install build
./install/bin/scaleri
```

`Launcher_BUILD_SCALERI_CLIENT=ON` builds Scaleri Client from `client/` with Gradle, which needs JDK 21 (set `JAVA_HOME`
to it if your default Java is 17) and Internet access. If you already have the mod's jar, pass
`-DLauncher_SCALERI_CLIENT_JAR=/path/to/scaleri-client-0.1.0.jar` instead. Without either, the launcher builds and runs
but Scaleri Client profiles have no mod to install.

Pull requests to this repository are built for Linux, Windows and macOS by GitHub Actions, and the packages are attached
to each run. CI builds Scaleri Client first and bundles it into every package.

### API keys

Prism Launcher's keys were issued to Prism Launcher, and its build files ask rebrands to use their own. Microsoft login
uses Scaleri's own Azure application (`174db889-6412-4846-869e-faafd5a578d0`), which Microsoft approved for the
Minecraft services, so it works out of the box. The other keys are empty, which turns the matching feature off until
you provide one:

| Feature | CMake option | Where to get one |
| --- | --- | --- |
| Microsoft account login (needed to play online) | `Launcher_MSA_CLIENT_ID` | Built in. To use another app, register one in the [Microsoft Entra admin center](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app) as a public client with "Personal Microsoft accounts" enabled, then [request Minecraft API access](https://aka.ms/mce-reviewappid). |
| CurseForge browsing and modpacks | `Launcher_CURSEFORGE_API_KEY` | Apply at the [CurseForge for Studios console](https://console.curseforge.com/). |
| Imgur screenshot uploads | `Launcher_IMGUR_CLIENT_ID` | [Register an Imgur application](https://api.imgur.com/oauth2/addclient). |

You can also paste the Microsoft client ID and CurseForge key into *Settings → APIs* in a running build. For CI builds,
add them as the repository secrets `MSA_CLIENT_ID` and `CURSEFORGE_API_KEY`.

### Changing the branding

The name, app ID and URLs live in `program_info/CMakeLists.txt` and the top-level `CMakeLists.txt`. The logo and every
icon are generated from the pixel art in `program_info/genicons.py`. The theme is `launcher/ui/themes/ScaleriTheme.cpp`
(palette) and `launcher/resources/scaleri_theme/scaleri.qss` (stylesheet).

## Staying up to date with Prism Launcher

The first commit on `main` is an unmodified import of Prism Launcher 11.1.1. To pull in a newer release, create a
branch from that import commit, replace its contents with the new release's source (with `libraries/libnbtplusplus`
copied in), commit, and merge that branch into `main`. Git then only asks you to resolve the files Scaleri changed.

## License

Scaleri is free software under the GNU General Public License v3.0, like Prism Launcher. See [LICENSE](LICENSE) and
[COPYING.md](COPYING.md), which keeps the notices of Prism Launcher, PolyMC, MultiMC and the bundled libraries.
The Scaleri logo and icons are licensed under CC BY-SA 4.0.
