# HOREGDAW (Android Edition)

**HOREGDAW** is a native Android Digital Audio Workstation (`.apk`) built in pure Rust with low-latency Android AAudio playback/recording (`cpal`/`ndk`) and hardware-accelerated Vulkan/OpenGL ES rendering (`egui` + `eframe` + `wgpu`), ported and rebranded from [SoundCraft](https://github.com/storytold/soundcraft).

---

## Key Features on Android
- **True Native ARM64 Android Port (`libhoregdaw.so`)**: Runs natively via Android's `NativeActivity` (`android_main`) with zero WebView overhead.
- **Phone & Tablet Landscape Layout**:
  - Immersive fullscreen `sensorLandscape` orientation with edge-swipe system bars.
  - Phone-optimized defaults (`85%` UI scale, collapsed Track/Clip sidebars by default to maximize timeline space, and compact **Narrow Mix** channel strips).
- **Full DAW Engine & Built-in DSP Suite**:
  - Multitrack audio & MIDI editing, playlists, clip gain, fades, automation lanes, and real-time metering.
  - Built-in pure-Rust DSP plugins: Channel Strip, 1-Band & 7-Band Parametric EQ, Compressor/Limiter, De-Esser, Expander/Gate, Maximizer, Room Reverb, Mod Delay, Flanger, Chorus, Phaser, Saturator, Lo-Fi, Pitch Shift, Time Stretch, Signal Generator, and Sampler/Synth instruments.
- **Native Android Storage**:
  - Preferences (`ui.json`), crash-recovery `Autosave/`, `Presets/`, `Sessions/`, and `Bounces/` are persisted in the app's Android data directory (`/sdcard/Android/data/com.horegdaw.app/files/`).

---

## Project Structure

| Path | Description |
|---|---|
| [`apps/horegdaw/`](apps/horegdaw/) | Native Android entry point (`libhoregdaw.so`, `android_main`) and mobile configuration |
| [`packaging/android/`](packaging/android/) | Android `NativeActivity` host, `AndroidManifest.xml`, resources, and APK build script (`package.sh`) |
| [`crates/time/`](crates/time/) | Timebases, tempo map, and grid (L0) |
| [`crates/audio-io/`](crates/audio-io/) | Audio decoding/encoding and peak cache (L0) |
| [`crates/midi/`](crates/midi/) | Standard MIDI Files and MIDI operations (L0) |
| [`crates/dsp/`](crates/dsp/) | Built-in EQ, dynamics, reverb, delay, modulation, pitch, and synth/sampler DSP (L1) |
| [`crates/model/`](crates/model/) | Session document model: tracks, clips, mixer, routing, automation (L2) |
| [`crates/mix/`](crates/mix/) | Real-time and offline multitrack mix engine (L3) |
| [`crates/engine/`](crates/engine/) | Command registry, undo/redo, editing, import/export, offline bounce (L4) |
| [`crates/playback/`](crates/playback/) | Android AAudio real-time audio I/O via `cpal` (L4) |
| [`crates/ui-egui/`](crates/ui-egui/) | Edit window, Mix window, MIDI editor, transport, and plugin editors in `egui` (L6) |

---

## Building the Android APK

### Prerequisites
- Rust nightly toolchain with `aarch64-linux-android` target
- JDK 17+ (`javac`, `keytool`)
- Android SDK (`platforms;android-35`, `build-tools;35.0.0`, `ndk;27.2.12479018`) — automatically installed by `packaging/android/package.sh` if missing

### Build Command
```bash
./packaging/android/package.sh
# or via xtask:
cargo xtask apk
```

This produces:
- `HOREGDAW.apk` (at the repository root)
- `dist/release/HOREGDAW-0.9.0.apk` (16 KB page-aligned and signed with APK Signature Scheme v2/v3)

### Install on a Connected Android Phone
```bash
adb install -r HOREGDAW.apk
```

---

## GitHub Actions Release Workflow
Every push to `main`/`release`, tag push (`v*`), or manual dispatch runs [`.github/workflows/android-release.yml`](.github/workflows/android-release.yml) to compile `libhoregdaw.so`, build and sign `HOREGDAW.apk`, and publish it to **GitHub Releases**.

## License
Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
