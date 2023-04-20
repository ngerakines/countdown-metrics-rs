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

    let client = match config.metric_tags.len() {
        0 => StatsdClient::from_sink(&config.metric_prefix, udp_sink),
        _ => {
            let mut client = StatsdClient::builder(&config.metric_prefix, udp_sink);
            for (key, value) in &config.metric_tags {
                client = client.with_tag(key, value);
            }
            client.build()
        }
    };

    let interval = Duration::seconds(config.interval as i64).to_std()?;

    let sleeper = sleep(interval);
    tokio::pin!(sleeper);

    'outer: loop {
        tokio::select! {
            _ = rx.recv() => {
                break 'outer;
            },
            () = &mut sleeper => {
                let now = Utc::now();
                for (key, value) in &keys {
                    trace!("Preparing {key} {value}");
                    if now > *value {
                        info!("Countdown expired: {key}");

                        if let Err(err) = client.gauge_with_tags(&config.countdown_key, 0).with_tag(&config.countdown_id, key).try_send() {
                            error!(cause = ?err, "Error sending metric");
                        }

                        continue
                    }
                    let remaining = (*value - now).num_seconds();
                    if let Err(err) = client.gauge_with_tags(&config.countdown_key, remaining as u64).with_tag(&config.countdown_id, key).try_send() {
                        error!(cause = ?err, "Error sending metric");
                    }
                }

                if let Err(err) = client.gauge(&config.heartbeat_metric, now.timestamp() as u64) {
                    error!(cause = ?err, "Error sending heartbeat metric");
                }

                sleeper.as_mut().reset(Instant::now() + interval);
            }
        }
    }

    trace!("metric_loop ended");
    Ok(())
}
