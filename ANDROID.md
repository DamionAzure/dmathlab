# Android

The release workflow builds and **signs** the APK automatically (`.github/workflows/release.yml`).
An unsigned APK is the usual reason Android says "App not installed", so CI always signs it.

## One-time: make a permanent signing key (recommended)

Without these secrets CI signs with a throwaway key — the APK still installs, but a later release
signed with a different key can't update it in place (users would have to uninstall first).

```bash
keytool -genkeypair -v -keystore dmath-release.keystore -alias dmath \
  -keyalg RSA -keysize 2048 -validity 10000
base64 -w0 dmath-release.keystore      # macOS: base64 -i dmath-release.keystore
```

Then in GitHub: **Settings → Secrets and variables → Actions → New repository secret**:

| Secret | Value |
|---|---|
| `ANDROID_KEYSTORE_BASE64` | output of the `base64` command |
| `ANDROID_KEY_ALIAS` | `dmath` |
| `ANDROID_KEYSTORE_PASSWORD` | the password you chose (store and key password must match) |

Back up the keystore file somewhere safe. If you lose it you can never update installed copies.

## Installing

Enable "install unknown apps" for your browser/file manager, open the APK. Or: `adb install -r DMath-Lab_x.y.z_android.apk`.
If an older copy signed with a different key is installed, uninstall it first.

## Local build (optional)

Needs Android Studio (SDK + NDK), JDK 17 and `NDK_HOME` set:
`npx tauri android init && npx tauri android build --apk --target aarch64`

## Launch screen

`scripts/android-brand.sh` runs in CI right after `tauri android init`. It gives the app a branded launch
screen: the Kaylei icon centred on the app's light (`#F4F4F5`) or dark (`#0B0B0C`) background on Android 12+,
and a plain matching window colour on Android 7–11. Run it yourself after a local `tauri android init`.
