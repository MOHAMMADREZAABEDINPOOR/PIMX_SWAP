# PIMXSWAP privacy information

Text conversion and language detection run locally. PIMXSWAP has no account, analytics, telemetry or cloud translation service. Input text, conversion candidates and clipboard snapshots are held temporarily in memory and are not written to the settings file or application diagnostics.

When you invoke a global shortcut, the app copies selected text from the focused application, converts it and pastes the result. With no selection it tries the current line before the caret, then the previous word. Supported clipboard contents are restored when safe. New clipboard content from another application takes precedence. The app does not continuously record your typing.

Preferences are stored in `%APPDATA%\com.pimxswap.desktop\settings.json`. Windows startup registration, when enabled, is stored in the current user's Run registry key. Uninstall preserves preferences; delete the settings directory after closing the app to reset them. Do not delete it while the app is running.

The installer may contact Microsoft to download WebView2 if that prerequisite is missing. Text conversion works offline after installation. Windows, Office and other applications have their own clipboard history, privacy and crash-reporting behavior, which PIMXSWAP does not control.
