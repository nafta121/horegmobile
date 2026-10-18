# Attribution

Every asset bundled with or committed to HOREGDAW is listed here with its author, source and
license. `cargo xtask assets` fails CI when a tracked asset file has no row.

## Icons

All UI icons (tools, transport, track buttons) are drawn procedurally in
`crates/ui-egui/src/icons.rs` (MIT OR Apache-2.0). No UI icon image files are used.

## Audio

The demo session's audio (drums, bass, pad) and its MIDI parts are synthesised in code in
`crates/engine/src/demo.rs` (MIT OR Apache-2.0). No sample files are bundled.

## Fonts

HOREGDAW bundles no font files. It uses the default fonts shipped inside the `egui` crate
(Ubuntu-Light: Ubuntu Font Licence 1.0; Hack: MIT; Noto Emoji: OFL-1.1; emoji-icon-font: OFL-1.1/MIT).

## Files

| Path | Author | Source | License |
|---|---|---|---|
| `docs/brand/artcraft-logo.svg` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`) |
| `docs/brand/artcraft-logo.png` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-logo-white.svg` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-logo-white.png` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-mark.svg` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-mark.png` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-mark-black.svg` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `docs/brand/artcraft-mark-black.png` | ArtCraft Team | craftrules `assets/brand/` | ArtCraft brand terms |
| `assets/app-icon/icon.svg` | SoundCraft contributors | original (hand-placed SVG paths) | MIT OR Apache-2.0 |
| `assets/app-icon/soundcraft-1024.png` | SoundCraft contributors | original, rendered from `icon.svg` | MIT OR Apache-2.0 |
| `assets/app-icon/hicolor/` | SoundCraft contributors | original, rendered from `icon.svg` (16–512 px PNGs and the scalable SVG) | MIT OR Apache-2.0 |
| `packaging/android/app/src/main/res/` | SoundCraft contributors | original, Android mipmap icons copied from `assets/app-icon/hicolor/` | MIT OR Apache-2.0 |
| `docs/images/` | SoundCraft contributors | screenshots rendered from the synthesised demo session | MIT OR Apache-2.0 |
