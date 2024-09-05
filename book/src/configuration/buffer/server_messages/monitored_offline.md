# `[buffer.server_messages.monitored_offline]`

Server message is sent if a monitored user goes offline.

**Example**

```toml
[buffer.server_messages.monitored_offline]
enabled = true
smart = 180
username_format = "full"
```

## `enabled`

Control if internal message type is enabled.

- **type**: boolean
- **values**: `true`, `false`
- **default**: `true`

## `smart`

Only show server message if the user has sent a message in the given time interval (seconds) prior to the server message.

- **type**: integer
- **values**: any positive integer
- **default**: not set