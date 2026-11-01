# @shikenmatrix/server

ShikenMatrix server application.

## Endpoints

1.  `/reporter` - Desktop client relay endpoint.
2.  `/mobile` - Android client relay endpoint.
3.  `/share` - Public share page.
4.  `/` - Admin panel.
5.  `/health` - Health check endpoint.

## Runtime Configuration

Server runtime settings use environment variables only for process-level behavior:

```bash
export SHIKENMATRIX_SERVER_ADDR="127.0.0.1:4317"
export SHIKENMATRIX_DB_PATH="shikenmatrix.sqlite3"
export SHIKENMATRIX_ADMIN_TOKEN="replace-with-admin-token"
```

## Upstream Configuration

Desktop and Android clients only connect to this server.

Native WebSocket, MX-Space, and S3 credentials are stored in SQLite and configured from the admin panel.

The server keeps the original upstream auth fields:

1.  Native WebSocket uses `ws_url` plus `token`.
2.  MX-Space uses `endpoint`, `method`, and `token`.
3.  S3 uses bucket, region, access key, secret key, endpoint, custom domain, and key template.

## Connection Rules

At the same time, one server accepts at most one Desktop client and one Android client.

The admin panel requires `SHIKENMATRIX_ADMIN_TOKEN`.

The public share APIs expose the latest reported activity, media, device information, and sync time without authentication.

## Android Snapshot Screen State

Android sends `{"type":"android_snapshot","snapshot":{...}}` over `/mobile`.
The optional `snapshot.screenOff` boolean is `true` when Android reports its screen is off and `false` when it is interactive.

The public `/api/share/mobile` response and `/api/share/mobile/events` SSE snapshots expose this as `status.screen_off`:

- `true`: the current Android WebSocket session reports a screen-off device.
- `false`: the current Android session reports an interactive screen.
- `null`: no screen state is known for the current session, or the device is disconnected. Desktop status always uses `null`.

`status.is_online` reports WebSocket transport connectivity only. Screen state is independent: a connected Android phone remains online when `screen_off` is `true`, and turning the screen off or on does not change connectivity.
Android background reporting may continue while the screen is off using the user-enabled foreground service. Device, location, activity, and media updates continue to reach the API for as long as the client remains connected; only an actual disconnect marks it offline.

Screen state belongs to the WebSocket session, not the last stored device snapshot. Reconnection starts with unknown screen state, and snapshots from a replaced session are ignored. Older senders may omit `screenOff`; that snapshot reports unknown screen state and uses connected presence.

`status.last_activity_at` remains the latest data-sync timestamp, including screen-off background updates; it is not a last-online timestamp. The panel labels it “最后同步”, shows “在线” or “离线” from connectivity and a separate “屏幕已点亮” / “屏幕已熄屏” row when known, and retains the latest media and device panels.
