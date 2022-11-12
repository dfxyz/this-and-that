use log::{debug, error, info, trace, warn};
use log::LevelFilter::Trace;

fn main() {
    let _logger_handle = async_logger::init(Trace, None);
    trace();
    debug();
    info();
    warn();
    error();
}

fn trace() {
    trace!("do you like what you see?");
}

fn debug() {
    debug!("do you like what you see?");
}

fn info() {
    info!("do you like what you see?");
}

fn warn() {
    warn!("do you like what you see?");
}

fn error() {
    error!("do you like what you see?");
}
