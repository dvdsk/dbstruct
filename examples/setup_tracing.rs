use tracing_subscriber::filter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn setup(additional_filter: &str) {
    let base_filter = "trace,sled=warn";
    let filter = filter::EnvFilter::builder()
        .parse(format!("{base_filter},{additional_filter}"))
        .unwrap();

    let fmt = fmt::layer()
        .pretty()
        .with_line_number(true)
        .with_test_writer();

    let _ignore_err = tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .try_init();
}

// every file not in a folder in the examples dir
// needs a main.rs
#[allow(dead_code)]
fn main() {}
