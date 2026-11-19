#!/usr/bin/env bash
# Branded launch screen for the generated Android project.
# Run after `tauri android init`. Safe to run twice.
#   Android 12+  : the system splash uses our colour and the app icon
#   Android 7-11 : the window background is our colour until the web view is ready
set -euo pipefail

RES="${1:-src-tauri/gen/android/app/src/main/res}"
[ -d "$RES" ] || { echo "android-brand: no such folder: $RES" >&2; exit 1; }

BASE="$RES/values/themes.xml"
[ -f "$BASE" ] || { echo "android-brand: $BASE not found" >&2; exit 1; }
THEME=$(grep -oE '<style name="[^"]+" parent="[^"]+"' "$BASE" | head -n1)
NAME=$(sed -E 's/<style name="([^"]+)".*/\1/' <<<"$THEME")
PARENT=$(sed -E 's/.*parent="([^"]+)".*/\1/' <<<"$THEME")
[ -n "$NAME" ] || { echo "android-brand: could not find the app theme in $BASE" >&2; exit 1; }

mkdir -p "$RES/values" "$RES/values-night" "$RES/values-v31" "$RES/values-night-v31"

# colours: light = zinc-100, dark = near-black (matches the app's two themes)
cat > "$RES/values/kaylei_splash.xml" <<X
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="kaylei_splash">#F4F4F5</color>
</resources>
X
cat > "$RES/values-night/kaylei_splash.xml" <<X
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="kaylei_splash">#0B0B0C</color>
</resources>
X

# API 24-30 (and the base for everything): plain coloured window
python3 - "$BASE" "$RES/values-night/themes.xml" <<'P'
import re, sys, os
for path in sys.argv[1:]:
    if not os.path.exists(path):
        continue
    s = open(path, encoding="utf-8").read()
    if "kaylei_splash" in s:
        continue
    item = '<item name="android:windowBackground">@color/kaylei_splash</item>'
    s = re.sub(r'(<style name="[^"]+" parent="[^"]+"\s*>)', lambda m: m.group(1) + "\n        " + item, s, count=1)
    open(path, "w", encoding="utf-8").write(s)
P

# API 31+: system splash screen (centred app icon on our colour)
for d in values-v31 values-night-v31; do
cat > "$RES/$d/kaylei_splash_theme.xml" <<X
<?xml version="1.0" encoding="utf-8"?>
<resources>
    <style name="$NAME" parent="$PARENT">
        <item name="android:windowBackground">@color/kaylei_splash</item>
        <item name="android:windowSplashScreenBackground">@color/kaylei_splash</item>
        <item name="android:windowSplashScreenAnimatedIcon">@mipmap/ic_launcher_foreground</item>
    </style>
</resources>
X
done
echo "android-brand: launch screen set for theme $NAME"
