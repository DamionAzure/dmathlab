#!/usr/bin/env bash
# Keep the app clear of the Android status bar (clock, battery, notifications), the
# camera cut-out and the gesture/navigation bar. Run after `tauri android init`.
# Safe to run twice. It patches the generated MainActivity so the web view is laid out
# inside the system bars, and it matches the status-bar icon colour to the app theme.
set -euo pipefail

ROOT="${1:-src-tauri/gen/android/app/src/main}"
FILE=$(find "$ROOT/java" -name MainActivity.kt 2>/dev/null | head -n1 || true)
[ -n "$FILE" ] || { echo "android-insets: MainActivity.kt not found under $ROOT/java" >&2; exit 1; }

python3 - "$FILE" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p).read()
if "kaylei-insets" in s:
    print("android-insets: already patched")
    sys.exit(0)

body = '''
    // kaylei-insets: lay the web view out inside the status bar, cut-out and navigation bar.
    val content = findViewById<android.view.View>(android.R.id.content)
    androidx.core.view.ViewCompat.setOnApplyWindowInsetsListener(content) { v, insets ->
      val bars = insets.getInsets(
        androidx.core.view.WindowInsetsCompat.Type.systemBars() or
          androidx.core.view.WindowInsetsCompat.Type.displayCutout()
      )
      v.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      androidx.core.view.WindowInsetsCompat.CONSUMED
    }
    val night = (resources.configuration.uiMode and
      android.content.res.Configuration.UI_MODE_NIGHT_MASK) == android.content.res.Configuration.UI_MODE_NIGHT_YES
    androidx.core.view.WindowCompat.getInsetsController(window, content).isAppearanceLightStatusBars = !night
    androidx.core.view.WindowCompat.getInsetsController(window, content).isAppearanceLightNavigationBars = !night
'''

m = re.search(r'super\.onCreate\(savedInstanceState\)\s*\n', s)
if m:
    s = s[:m.end()] + body + s[m.end():]
else:
    cls = re.search(r'class MainActivity\s*:\s*TauriActivity\(\)\s*\{?', s)
    if not cls:
        sys.exit("android-insets: unexpected MainActivity.kt shape")
    if not cls.group(0).endswith("{"):
        s = s.replace(cls.group(0), "class MainActivity : TauriActivity() {", 1).rstrip() + "\n}\n"
    inject = "\n  override fun onCreate(savedInstanceState: android.os.Bundle?) {\n    super.onCreate(savedInstanceState)\n" + body + "  }\n"
    i = s.index("class MainActivity")
    j = s.index("{", i) + 1
    s = s[:j] + inject + s[j:]
open(p, "w").write(s)
print("android-insets: patched", p)
PY
