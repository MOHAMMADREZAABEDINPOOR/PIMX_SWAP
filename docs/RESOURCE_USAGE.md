# PIMX SWAP resource usage

The background tray is designed to keep the native keyboard engine and global shortcuts available. Closing the editor destroys its WebView; it is created again when the editor opens.

| Measurement | Result |
|:---|:---|
| Build | Windows x64, PIMXSWAP 1.0.0 release executable |
| State | Idle background tray, editor closed |
| Full process-tree working set | 12.85 MiB |
| Minimum / maximum across 10 samples | 12.85 / 12.85 MiB |
| Sample duration | 11.37 seconds after 2 seconds of warm-up |
| Private bytes | 2.23 MiB |
| Processes | 1 native process; no editor WebView active |
| CPU while idle | 0.0000% in this sample |
| Recorded | 2026-10-07T19:45:21.3945036Z |

1 MiB = 1,048,576 bytes. This is a local Windows measurement of this exact release, not a guaranteed maximum. The open editor uses WebView2 processes and consumes more memory; the tray measurement does not represent that state. Hardware, Windows version, runtime and activity change resource usage. Summing working sets includes any shared pages counted per process.

[Raw measurement and executable SHA-256](RESOURCE_USAGE.json). The measurement script launched the release with `--background`, took 10 samples of the complete descendant process tree, then stopped its own instance. It did not convert clipboard text or change app settings.
