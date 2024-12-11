use log::{LevelFilter, Log, Metadata, Record};
use std::fmt::Write;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::Subscriber;
use tracing_core::Event;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::layer::{Context, Layer};

use super::vsock_client::VsockClient;

// 标准日志实现
struct VsockLogger {
    client: Arc<Mutex<VsockClient>>,
}

impl Log for VsockLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Ok(client) = self.client.lock() {
            let message = format!(
                "[{}] {}: {}\n",
                record.level(),
                record.target(),
                record.args()
            );
            client.log(&message);
        }
    }

    fn flush(&self) {}
}

// Tracing 实现
struct VsockLayer {
    client: Arc<Mutex<VsockClient>>,
}

impl<S> Layer<S> for VsockLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if let Ok(client) = self.client.lock() {
            let mut message = String::new();
            let metadata = event.metadata();
            write!(
                &mut message,
                "[{}] {}: ",
                metadata.level(),
                metadata.target()
            )
            .unwrap();

            let mut visitor = StringVisitor(&mut message);
            event.record(&mut visitor);
            message.push('\n');

            client.log(&message);
        }
    }
}

struct StringVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.0, "{}={:?} ", field.name(), value).unwrap();
    }
}

pub fn init_logging(client: VsockClient) -> Result<(), Box<dyn std::error::Error>> {
    let client = Arc::new(Mutex::new(client));

    // 设置标准日志
    let logger = VsockLogger {
        client: client.clone(),
    };
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(LevelFilter::Trace);

    // 设置 tracing
    let layer = VsockLayer {
        client: client.clone(),
    };
    let subscriber = tracing_subscriber::registry().with(layer);
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
