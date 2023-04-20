# Countdown Metrics

A daemon that publishes the number of seconds until a given date to a statsd sink.

# Configuration

The daemon is configured via environment variables:

* `ENV_PREFIX`: The prefix for environment variables. Defaults to `COUNTDOWNS_`.
* `INTERVAL`: The number of seconds between publishing metrics. Defaults to `10`.
* `STATSD_HOST`: The host to publish metrics to. Defaults to `127.0.0.1:8125`.
* `METRIC_PREFIX`: The prefix to use for metrics. Blank by default.
* `METRIC_TAGS`: A comma separated list of tags to add to all metrics. Blank by default.
* `COUNTDOWN_KEY`: The name of the countdown metric. Defaults to `countdown`.
* `COUNTDOWN_ID`: The name of the countdown metric tag. Defaults to `name`.
* `HEARTBEAT_METRIC`: The name of the heartbeat metric. Defaults to `heartbeat`.

The metrics to watch are read as environment variables that start with the value of the `ENV_PREFIX` environment variable. The value of the environment variable should be a date in the format `YYYY-MM-DDTHH:MM:SSZ`. The prefix is stripped and the remaining text is used as the name of the metric.

For example, consider the following environment variables:

```
ENV_PREFIX=SURGE
METRIC_PREFIX=wildmagic_rocks
SURGE_FOO="2024-01-01T14:00:00-04:00"
WATCH_THIS_BAR="2024-01-01T14:00:00-04:00"
```

The above environment variable would result in a metric named `wildmagic_rocks.foo` with a value of the number of seconds until the date `2024-01-01T14:00:00-04:00` and a tag named `name` with a value of `foo` published every 10 seconds. The `WATCH_THIS_BAR` environment variable would be ignored because it is not one of the configuration environment variables nor does it have the prefix `SURGE` as defined by the `ENV_PREFIX` environment variable.

# Example

First start a fake statsd sink:

    $ nc -u -l -p 8125

Then run the daemon with an example metric:

    $ COUNTDOWNS_FOO="2024-01-01T14:00:00-04:00" cargo run

You'll see metrics published with the default values:

```
countdown:22192887|g|#name:foo
heatbeat:1681939112|g
countdown:22192877|g|#name:foo
heatbeat:1681939122|g
countdown:22192867|g|#name:foo
heatbeat:1681939132|g
```

## Global Tags

When running:

    $ METRIC_TAGS="DD_SERVICE=cntdwn,DD_VERSION=1.2.0" COUNTDOWNS_FOO="2024-01-01T14:00:00-04:00" cargo run

You'll get:

```
countdown:22132947|g|#DD_SERVICE:cntdwn,DD_VERSION:1.2.0,name:foo
heatbeat:1681999052|g|#DD_SERVICE:cntdwn,DD_VERSION:1.2.0
```

# Usage

Use this to monitor your stuff.

Deploy it with something like this:

```yaml
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cntdwn
  labels:
    app: cntdwn
spec:
  revisionHistoryLimit: 1
  selector:
    matchLabels:
      app: cntdwn
  template:
    metadata:
      labels:
        app: cntdwn
    spec:
      containers:
        - name: app
          image: ngerakines/countdown-metrics:v1.2.0
          env:
            - name: COUNTDOWNS_SITE_CERT
              value: "2024-01-01T14:00:00-04:00"
            - name: COUNTDOWNS_TS_KEY_DC1
              value: "2024-02-01T14:00:00-04:00"
```

Then create a monitor in your observability stack that alerts when the value of the metric is less than 345600 (4 days) or 950400 (11 days).
