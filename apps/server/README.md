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

The public share page does not require authentication and only exposes current window, current media, and last activity time.
