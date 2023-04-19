use std::env;

use derive_builder::Builder;

#[derive(Builder, Clone, Debug)]
#[builder(setter(into, strip_option))]
pub struct Config {
    #[builder(setter(into), default = "self.default_env_prefix()")]
    pub env_prefix: String,

    #[builder(setter(into), default = "self.default_interval()")]
    pub interval: u16,

    #[builder(setter(into), default = "self.default_metric_prefix()")]
    pub metric_prefix: String,

    #[builder(setter(into), default = "self.default_metric_name()")]
    pub metric_name: String,

    #[builder(setter(into), default = "self.default_heartbeat_metric()")]
    pub heartbeat_metric: String,

    #[builder(setter(into), default = "self.default_statsd_sink()")]
    pub statsd_sink: String,
}

impl ConfigBuilder {
    fn default_env_prefix(&self) -> String {
        env::var("ENV_PREFIX").unwrap_or("WATCH_KEY_".to_string())
    }

    fn default_interval(&self) -> u16 {
        env::var("INTERVAL")
            .unwrap_or("10".to_string())
            .parse::<u16>()
            .unwrap_or(10)
    }

    fn default_metric_prefix(&self) -> String {
        env::var("METRIC_PREFIX").unwrap_or("".to_string())
    }

    fn default_metric_name(&self) -> String {
        env::var("METRIC_NAME").unwrap_or("key_expire".to_string())
    }

    fn default_heartbeat_metric(&self) -> String {
        env::var("HEARTBEAT_METRIC").unwrap_or("heatbeat".to_string())
    }

    fn default_statsd_sink(&self) -> String {
        env::var("STATSD_SINK").unwrap_or("127.0.0.1:8125".to_string())
    }
}
