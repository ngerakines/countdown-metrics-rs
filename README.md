# Countdown Metrics

A daemon that publishes the number of seconds until a given date to a statsd sink.

# Configuration

The daemon is configured via environment variables:

* `ENV_PREFIX`: The prefix for environment variables. Defaults to `WATCH_KEY_`.
* `INTERVAL`: The number of seconds between publishing metrics. Defaults to `10`.
* `STATSD_HOST`: The host to publish metrics to. Defaults to `127.0.0.1:8125`.
* `METRIC_PREFIX`: The prefix to use for metrics. Blank by default.
* `METRIC_NAME`: The name of the countdown metric. Defaults to `key_expire`.
* `HEARTBEAT_METRIC`: The name of the heartbeat metric. Defaults to `heartbeat`.

The metrics to watch are read as environment variables that start with the value of the `ENV_PREFIX` environment variable. The value of the environment variable should be a date in the format `YYYY-MM-DDTHH:MM:SSZ`. The prefix is stripped and the remaining text is used as the name of the metric.

For example, consider the following environment variables:

```
ENV_PREFIX=COUNTDOWN
METRIC_PREFIX=wildmagic_rocks
COUNTDOWN_FOO="2024-01-01T14:00:00-04:00"
WATCH_THIS_BAR="2024-01-01T14:00:00-04:00"
```

The above environment variable would result in a metric named `wildmagic_rocks.foo` with a value of the number of seconds until the date `2024-01-01T14:00:00-04:00` and a tag named `name` with a value of `foo` published every 10 seconds. The `WATCH_THIS_BAR` environment variable would be ignored because it is not one of the configuration environment variables nor does it have the prefix `COUNTDOWN` as defined by the `ENV_PREFIX` environment variable.

# Example

First start a fake statsd sink:

    $ nc -u -l -p 8125

Then run the daemon with an example metric:

    $ WATCH_KEY_FOO="2024-01-01T14:00:00-04:00" cargo run

You'll see metrics published with the default values:

```
key_expire:22204489|g|#name:foo
heatbeat:1681927510|g
key_expire:22204479|g|#name:foo
heatbeat:1681927520|g
```

# Usage

Use this to monitor your stuff.

Deploy it with something like this:

```yaml
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: devopswatcher
  labels:
    app: devopswatcher
spec:
  revisionHistoryLimit: 1
  selector:
    matchLabels:
      app: devopswatcher
  template:
    metadata:
      labels:
        app: devopswatcher
    spec:
      containers:
        - name: app
          image: ngerakines/countdown-metrics:latest
          env:
            - name: WATCH_KEY_SITE_CERT
              value: "2024-01-01T14:00:00-04:00"
            - name: WATCH_KEY_TS_KEY_DC1
              value: "2024-02-01T14:00:00-04:00"
```

Then create a monitor in your observability stack that alerts when the value of the metric is less than 345600 (4 days) or 950400 (11 days).