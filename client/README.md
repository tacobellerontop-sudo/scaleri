# Scaleri Client

A casual PvP client for Minecraft **1.21.1** (Fabric). It works like Lunar or Badlion:
a drag-and-drop HUD and quality-of-life PvP features, with a softer, friendlier look.
Everything is client-side and cosmetic. There is no reach, autoclicker or anything
else that gives an unfair advantage.

## Controls

| Key | Action |
| --- | --- |
| `Right Shift` | Open the Scaleri menu (or use the **Scaleri** button on the title and pause screens) |
| `C` (hold) | Zoom. Scroll while zooming to zoom further in or out |
| `Left Alt` (hold) | Freelook (off by default, turn it on in the menu) |
| Sprint key (tap) | Toggle sprint |

You can rebind all of these under *Options → Controls → Key Binds → Scaleri Client*.

In the menu, **click a card** to turn a module on or off, and **right-click it or click ⚙**
to open its settings. **Edit HUD** lets you drag modules anywhere, scroll to resize them,
and right-click one to open its settings. Modules snap to the screen edges and center.

## Modules

**HUD:** FPS, CPS, Ping, Coordinates (with direction and biome), Keystrokes (WASD,
mouse buttons with CPS, spacebar), Armor Status, Potion Effects, Reach Display,
Combo Counter, Clock, Speed, Memory, Server Address

**PvP:** Toggle Sprint / Sneak, Custom Crosshair (6 styles, target highlight,
attack cooldown bar), No Hurt Cam, Low Fire, Block Outline (color and fill),
Hit Particles

**Visual:** Zoom (smooth, scroll to adjust), Freelook, Fullbright, Time Changer,
Clear Weather

**Misc:** Chat Timestamps, Auto GG, Death Coordinates, Theme (accent color and branding)

Text modules all share background, shadow, brackets and color options, and every
color can be a swatch, any opacity, or animated chroma.

Settings save to `.minecraft/config/scaleri-client.json`.

## Install

1. Install [Fabric Loader](https://fabricmc.net/use/) for Minecraft 1.21.1
   (in Scaleri/Prism: *Add Instance → 1.21.1 → Fabric*).
2. Put `scaleri-client-<version>.jar` and [Fabric API](https://modrinth.com/mod/fabric-api)
   in the instance's `mods` folder.
3. For Lunar-style FPS, add [Sodium](https://modrinth.com/mod/sodium) and
   [Lithium](https://modrinth.com/mod/lithium). Scaleri works fine alongside them.

## Building

Requires JDK 21.

```sh
./gradlew build        # jar ends up in build/libs/
./gradlew runClient    # launch a dev copy of the game with the client loaded
```

CI (`.github/workflows/client.yml`) builds the jar and runs `./gradlew runClient -PsmokeTest`
under a virtual display. That boots the real game, makes sure every mixin applies, opens
every Scaleri screen, and then quits.

## Server rules

Most servers are fine with everything here. Some servers don't allow **Freelook**,
so it is off by default. Check a server's rules before turning it on.
