use std::{
    collections::HashMap,
    net::{ToSocketAddrs, UdpSocket},
};

use anyhow::Result;
use cadence::{BufferedUdpMetricSink, Gauged, StatsdClient};
use chrono::{DateTime, Duration, Utc};
use tokio::{
    sync::broadcast::Receiver,
    time::{sleep, Instant},
};
use tracing::{error, info, trace};

use crate::config::Config;

pub async fn metric_loop(
    config: Config,
    keys: HashMap<String, DateTime<Utc>>,
    rx: &mut Receiver<bool>,
) -> Result<()> {
    trace!("metric_loop started");

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let host = config
        .statsd_sink
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow::anyhow!("Unable to resolve statsd sink {}", config.statsd_sink))?;
    let udp_sink = BufferedUdpMetricSink::from(host, socket)?;
    let client = StatsdClient::from_sink(&config.metric_prefix, udp_sink);

    let interval = Duration::seconds(config.interval as i64).to_std()?;

    let sleeper = sleep(interval);
    tokio::pin!(sleeper);

    let metric = &config.metric_name;
    let heartbeat_metric = &config.heartbeat_metric;

    'outer: loop {
        tokio::select! {
            _ = rx.recv() => {
                break 'outer;
            },
            () = &mut sleeper => {
                let now = Utc::now();
                for (key, value) in &keys {
                    if now > *value {
                        info!("Key {} expired", key);
                        continue
                    }
                    let remaining = (*value - now).num_seconds();
                    if let Err(err) = client.gauge_with_tags(metric, remaining as u64).with_tag("name", key).try_send() {
                        error!(cause = ?err, "Error sending metric");
                    }
                }

                if let Err(err) = client.gauge(heartbeat_metric, now.timestamp() as u64) {
                    error!(cause = ?err, "Error sending heartbeat metric");
                }

                sleeper.as_mut().reset(Instant::now() + interval);
            }
        }
    }

    trace!("metric_loop ended");
    Ok(())
}
