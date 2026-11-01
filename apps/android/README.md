# ShikenMatrix Android

Native Kotlin/Compose Android client for ShikenMatrix Server, using Miuix components.

## Reporting and screen state

- Configure the real Server WebSocket URL and API key, then enable timed reporting. URL, key, interval, and reporting state are persisted locally.
- `BackgroundReportingService` runs a low-importance foreground notification whenever reporting is enabled and a key is configured. The service supports real background media, battery, network, and permitted location collection on a best-effort basis.
- With Notification Listener Access, actual media-session changes trigger prompt collection; the configured periodic collector remains the fallback for background song/state updates.
- Screen-off is reported as snapshot `screenOff` metadata. It does not disconnect the socket or make a connected device offline. Screen state and actual connection state are separate.
- App resume, notification-listener recovery, boot completion, and package replacement attempt to restore an enabled reporting session. `START_STICKY` is used only while reporting remains enabled and configured. Android background-start restrictions can still deny a foreground service; the UI shows the failure and opening the app can retry.
- The service declares `specialUse` for explicit, continuous user-authorized device-presence/media/battery monitoring, with a manifest subtype describing this purpose. This is not a finite bulk-upload/data-sync job. The optional `location` service type is added only when permission and platform conditions permit; absent location permission does not block other reporting.
- If distributed through Google Play, the declared `specialUse` purpose/subtype requires Play Console justification and policy review. This manifest rationale is not a guarantee of Play approval; sideloaded runtime operation and distribution-policy approval are separate.
- `onTimeout(startId, fgsType)` stops and durably suspends the service instead of creating a restart loop. A visible app resume or configuration action can retry.
- Settings show this app's battery-optimization exemption and Android-reported background restriction status, with user-controlled battery-optimization and app-details actions. No battery policy is changed silently; a reported unrestricted state does not prove the absence of vendor-specific restrictions.
- On vendor ROMs, allow this app's auto-start/background-start setting and, when needed, unrestricted battery use. Auto-start permits recovery; it does not itself replace the foreground service or media-session subscription.

There are no privileged shell commands, process-priority adjustments, wake locks, keepalive alarms, or global Doze/battery-policy changes. A foreground notification does not guarantee updates during deep sleep, vendor process killing, network loss, or user force-stop. Unknown/unavailable device information is not fabricated.

## Permissions

- Usage Access: foreground application detection.
- Notification Listener Access: actual active media sessions and media changes.
- Coarse Location: optional permitted approximate/last-known location; no background-location grant is required to enable reporting.
- Notification permission (Android 13+): optional visibility of the low-priority reporting notification. An explanation precedes the request; refusal does not prevent foreground reporting.
- Boot completion: best-effort recovery of a previously enabled, configured ordinary service, subject to Android restrictions.

## Build

```bash
cd apps/android
./gradlew assembleDebug
```
