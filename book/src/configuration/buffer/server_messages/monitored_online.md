# `[buffer.server_messages.monitored_online]`

Server message is sent if a monitored user goes online.

> 💡 Read more about [monitoring users](../../../guides/monitor-users.html).

**Example**

```toml
[buffer.server_messages.monitored_online]
enabled = true
smart = 180
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
