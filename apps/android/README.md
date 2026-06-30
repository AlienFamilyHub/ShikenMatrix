# ShikenMatrix Android

Native Kotlin Android client for ShikenMatrix Server.

## Features

1.  Monitors foreground app, media playback, media artwork, app icons, battery, network status, and coarse location.
2.  Uses a Kotlin foreground service for keep-alive and background reporting.
3.  Sends device snapshots only to ShikenMatrix Server.
4.  Requests root with `su` when available and opens Magisk / KernelSU / SukiSU managers when root is not granted.
5.  Builds the UI with Kotlin Compose and Miuix.

## Build

```bash
cd apps/android
./gradlew assembleDebug
```

## Runtime Permissions

The app can use non-root paths first:

1.  Usage Access for foreground app detection.
2.  Notification Listener Access for active media sessions.
3.  Coarse Location for approximate location.
4.  Notification permission for the foreground keep-alive service.
5.  Battery optimization exemption for more reliable background operation.

Root is optional and used as a fallback for stronger foreground app detection.

## Keep Alive

When background reporting is enabled, the app persists only the ShikenMatrix Server URL and report interval locally.

After `BOOT_COMPLETED`, `LOCKED_BOOT_COMPLETED`, or app package replacement, `ShikenMatrixBootReceiver` starts the foreground service again if reporting was previously enabled.
