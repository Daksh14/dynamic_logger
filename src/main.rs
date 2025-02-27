extern crate lazy_static;

use chrono::format::parse;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use lazy_static::lazy_static;
use log::{parse_log, Log, LogManager, TypeOfLog};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::time::{self, interval, Duration};

use crate::args::Args;

lazy_static! {
    pub static ref LOGS: LogManager = LogManager::new();
}

mod args;
mod log;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    let log_manager = Arc::new(&LOGS);

    tokio::spawn(async move {
        let clone = Arc::clone(&log_manager);

        // start with 60 second interval window
        let mut curr_duration = 60;
        let mut interval = time::interval(Duration::from_secs(curr_duration));
        let mut peak_rate = 0;

        loop {
            let clone = Arc::clone(&clone);

            interval.tick().await;

            let per_sec_rate: usize = clone.get_curr_rate_per_second().await;

            if peak_rate < per_sec_rate {
                peak_rate = per_sec_rate;
            }

            // depending on the rate, we change the window interval
            if per_sec_rate > 2500 {
                if curr_duration != 30 {
                    curr_duration == 30;
                    interval = time::interval(Duration::from_secs(curr_duration));
                }
            } else if per_sec_rate < 600 {
                if curr_duration != 120 {
                    curr_duration = 120;
                    interval = time::interval(Duration::from_secs(curr_duration));
                }
            }

            clone.print_statistics(curr_duration, peak_rate).await;
        }
    });

    let log_manager = Arc::new(&LOGS);

    // This task reads the lines from the buffered reader and blocks to keep listening for logs from stdin
    loop {
        let logger = Arc::clone(&log_manager);

        if let Some(line) = lines.next_line().await.expect("Cannot obtain next line") {
            let log = parse_log(line);

            logger.push(log).await;
        }
    }
}
