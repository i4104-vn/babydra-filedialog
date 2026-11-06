# babydra-filedialog

Native file chooser dialog, XDG Desktop Portal FileChooser service, and `xdg-open` drop-in replacement for the BabyDra desktop ecosystem.

## Features

- **Modern File Chooser Dialog**:
  - Consistent BabyDra dark/light design theme matching `babydra-explore` and `babydra-settings`.
  - Places sidebar (Home, Desktop, Documents, Downloads, Music, Pictures, Videos, Trash, File System).
  - Breadcrumb navigation, path entry, search, hidden file toggle (`Ctrl+H`), new folder creation.
  - Multi-file selection, open folder, save file modes with dynamic filter extensions.
  - Comprehensive internationalization (`en`, `vi`) powered by `babydra-core::i18n`.
- **XDG Desktop Portal Provider**:
  - Implements `org.freedesktop.impl.portal.FileChooser` on D-Bus session bus.
  - Intercepts all file open/save requests from browsers (Chrome, Firefox), office apps, and GTK apps when `GTK_USE_PORTAL=1` is set.
- **`xdg-open` Drop-in Replacement**:
  - When invoked as `xdg-open` (via symlink in `~/.local/bin/xdg-open`), opens URLs, files, or folders using default desktop applications and `babydra-explore`.

## Usage

```bash
# Open file dialog
babydra-filedialog

# Open folder dialog
babydra-filedialog -d

# Save file dialog with filter
babydra-filedialog -s --name "notes.txt" --filter "Text files:*.txt,*.md"

# Run as XDG Desktop Portal daemon
babydra-filedialog --portal

# xdg-open replacement
babydra-filedialog --open https://github.com
# or directly via symlink
xdg-open ~/Documents
```
