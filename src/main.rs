use anyhow::{anyhow, Error, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::{borrow::BorrowMut, env};
use tokio::signal;
use tokio::sync::broadcast::Receiver;
use tracing::{error, info, trace};

#[cfg(debug_assertions)]
use tracing::warn;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod metrics;

use crate::{config::ConfigBuilder, metrics::metric_loop};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().or_else(|_| {
                tracing_subscriber::EnvFilter::try_new("key_expire_metrics_rs=trace,warning")
            })?,
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    #[cfg(debug_assertions)]
    warn!("Debug assertions enabled");

    let config = ConfigBuilder::default().build()?;
    let env_key_prefix = config.clone().env_prefix;

    let mut keys: HashMap<String, DateTime<Utc>> = HashMap::new();

    for (key, value) in env::vars() {
        if key.starts_with(&env_key_prefix) {
            let trimmed_key = key
                .strip_prefix(&env_key_prefix)
                .map(str::to_lowercase)
                .ok_or_else(|| {
                    anyhow!("error processing key")
                        .context(format!("environment variable {key} {value}"))
                })?;

            let rfc3339 = DateTime::parse_from_rfc3339(&value).map_err(|err| {
                Error::msg(err).context(format!("environment variable {key} {value}"))
            })?;
            trace!("Watching {trimmed_key} with expiration {rfc3339}");

            keys.insert(trimmed_key, rfc3339.into());
        }
    }

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<bool>(2);
    let (rev_shutdown_tx, mut rev_shutdown_rx) = tokio::sync::broadcast::channel::<bool>(2);

    let loop_join_handler = tokio::spawn(async move {
        let loop_rx = shutdown_rx.borrow_mut();
        if let Err(err) = metric_loop(config, keys, loop_rx).await {
            error!(cause = ?err, "metric_loop error");
            rev_shutdown_tx.send(true).unwrap();
        }
    });

    shutdown_signal(rev_shutdown_rx.borrow_mut()).await;

    shutdown_tx.send(true)?;

    loop_join_handler.await?;

    Ok(())
}

async fn shutdown_signal(rx: &mut Receiver<bool>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = rx.recv() => {},
    }

    info!("signal received, starting graceful shutdown");
}
