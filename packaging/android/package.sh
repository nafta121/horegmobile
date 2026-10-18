#!/usr/bin/env bash
# Build and package native HOREGDAW for Android: $DIST/HOREGDAW-<version>.apk
#
# Compiles the pure-Rust DAW into native Android shared library `libhoregdaw.so`
# (`aarch64-linux-android` -> `lib/arm64-v8a/libhoregdaw.so`) using the Android NDK,
# then packages, aligns, and signs the native APK.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/android"

export CARGO_HOME="${CARGO_HOME:-/tmp/cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-/tmp/rustup}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/soundcraft-target}"
export PATH="$CARGO_HOME/bin:$PATH"

export ANDROID_HOME="${ANDROID_HOME:-/tmp/android-sdk}"
NDK_VER="27.2.12479018"
NDK_HOME="$ANDROID_HOME/ndk/$NDK_VER"
BUILD_TOOLS="$ANDROID_HOME/build-tools/35.0.0"
ANDROID_JAR="$ANDROID_HOME/platforms/android-35/android.jar"

if [ ! -d "$NDK_HOME" ]; then
  echo "==> Installing Android NDK $NDK_VER..."
  yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses >/dev/null || true
  "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" "ndk;$NDK_VER"
fi

rustup target add aarch64-linux-android

TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export CC_aarch64_linux_android="$TOOLCHAIN/aarch64-linux-android26-clang"
export CXX_aarch64_linux_android="$TOOLCHAIN/aarch64-linux-android26-clang++"
export AR_aarch64_linux_android="$TOOLCHAIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/aarch64-linux-android26-clang"

echo "==> Building native Rust library libhoregdaw.so (aarch64-linux-android)..."
cargo build --release --target aarch64-linux-android -p soundcraft --lib

SO_PATH="$CARGO_TARGET_DIR/aarch64-linux-android/release/libhoregdaw.so"
"$TOOLCHAIN/llvm-strip" --strip-unneeded "$SO_PATH"
ls -lh "$SO_PATH"

# Populate Android mipmap icons from assets/app-icon/hicolor
ICON_SRC="$ROOT/assets/app-icon/hicolor"
for pair in "mdpi:48x48" "hdpi:64x64" "xhdpi:128x128" "xxhdpi:256x256" "xxxhdpi:512x512"; do
  density="${pair%%:*}"
  size="${pair##*:}"
  dir="$HERE/app/src/main/res/mipmap-$density"
  mkdir -p "$dir"
  /bin/cp -f "$ICON_SRC/$size/apps/ai.storyteller.soundcraft.png" "$dir/ic_launcher.png"
done

KEYSTORE="$HERE/horegdaw-release.keystore"
if [ ! -f "$KEYSTORE" ]; then
  keytool -genkeypair -v \
    -keystore "$KEYSTORE" \
    -storepass horegdaw \
    -keypass horegdaw \
    -alias horegdaw \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000 \
    -dname "CN=HOREGDAW, OU=Audio, O=HOREGDAW, L=Jakarta, S=ID, C=ID"
fi

WORK="$HERE/build/apk-work"
rm -rf "$WORK"
mkdir -p "$WORK/compiled_res" "$WORK/gen" "$WORK/classes" "$WORK/dex" "$WORK/lib/arm64-v8a"
/bin/cp -f "$SO_PATH" "$WORK/lib/arm64-v8a/libhoregdaw.so"

echo "==> Compiling Android resources with aapt2..."
"$BUILD_TOOLS/aapt2" compile --dir "$HERE/app/src/main/res" -o "$WORK/compiled_res/res.zip"

echo "==> Linking resources into base APK..."
"$BUILD_TOOLS/aapt2" link \
  -I "$ANDROID_JAR" \
  --manifest "$HERE/app/src/main/AndroidManifest.xml" \
  --java "$WORK/gen" \
  --min-sdk-version 26 \
  --target-sdk-version 35 \
  --version-code 900 \
  --version-name "$VERSION" \
  -o "$WORK/unaligned.apk" \
  "$WORK/compiled_res/res.zip"

echo "==> Compiling Java NativeActivity host..."
find "$HERE/app/src/main/java" "$WORK/gen" -name "*.java" > "$WORK/sources.txt"
javac --release 17 -cp "$ANDROID_JAR" -d "$WORK/classes" @"$WORK/sources.txt"

echo "==> Compiling bytecode to DEX with d8..."
find "$WORK/classes" -name "*.class" > "$WORK/classes.txt"
"$BUILD_TOOLS/d8" --release --lib "$ANDROID_JAR" --min-api 26 --output "$WORK/dex" @"$WORK/classes.txt"

echo "==> Packaging classes.dex and native lib/arm64-v8a/libhoregdaw.so into APK..."
(cd "$WORK/dex" && zip -q "$WORK/unaligned.apk" classes.dex)
(cd "$WORK" && zip -q -0 "$WORK/unaligned.apk" lib/arm64-v8a/libhoregdaw.so)

echo "==> Aligning (16 KB page-aligned) and signing APK..."
"$BUILD_TOOLS/zipalign" -f -P 16 -v 4 "$WORK/unaligned.apk" "$WORK/aligned.apk" >/dev/null
OUT_APK="$DIST/HOREGDAW-$VERSION.apk"
"$BUILD_TOOLS/apksigner" sign \
  --ks "$KEYSTORE" \
  --ks-pass pass:horegdaw \
  --key-pass pass:horegdaw \
  --ks-key-alias horegdaw \
  --out "$OUT_APK" \
  "$WORK/aligned.apk"

"$BUILD_TOOLS/apksigner" verify --verbose "$OUT_APK"
/bin/cp -f "$OUT_APK" "$ROOT/HOREGDAW.apk"
echo "==> Wrote $OUT_APK and $ROOT/HOREGDAW.apk"
ls -lh "$OUT_APK" "$ROOT/HOREGDAW.apk"
