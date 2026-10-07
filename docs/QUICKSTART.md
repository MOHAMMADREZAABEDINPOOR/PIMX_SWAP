# PIMXSWAP 1.0.0

Run `PIMXSWAP-Setup.exe` to install for the current Windows user, or run `PIMXSWAP.exe` directly. The supplied build targets Windows x64 and requires Microsoft Edge WebView2 Runtime.

At first launch choose the interface language, source keyboard layout and target keyboard layout. Interface languages are English and Persian. The picker exposes the Windows keyboard layout catalog; automatic language detection has profiles for English, Persian, Arabic, Russian, German, French and Spanish. Use Direct convert when you need an explicit layout pair. IME composition and arbitrary custom keyboards cannot always be reconstructed from typed text.

Select wrong-layout text in another application and press `Ctrl+Shift+Space`. For example, with English and Persian configured, `اثممخ` becomes `hello`, and `sghl lk o,` becomes `سلام من خو`. Without a selection the shortcut tries the current line before the caret, then the previous word; select several lines to convert the whole passage. In the internal editor use `Ctrl+Enter`.

Use the compact workspace button to keep only the conversion controls visible. Change hotkeys, theme, motion and startup behavior in Settings. Closing the window can leave the app running in the tray; use Exit in the tray menu to stop it.

If a shortcut does nothing, release modifier keys and keep the intended application focused. Protected/password fields, applications running with higher privileges and applications with unusual clipboard handlers may reject automatic replacement. Use the internal editor and manual paste in those cases. Review the result when text is ambiguous. Universal compatibility is not guaranteed.

The release binaries are currently unsigned. Compare the SHA-256 checksum with the trusted download source. No automatic update service is configured; install a newer release manually. Keep your existing settings when updating.
