use chrono::{DateTime, Local, TimeZone, Utc};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::Mutex;

use tokio::time::{sleep, Duration};

pub struct LogManager {
    entries: Mutex<Vec<Log>>,
    weights: Mutex<Vec<(usize, String)>>,
    last_curr_rate: Mutex<usize>,
    burst_detected: Mutex<(bool, usize)>,
}

impl LogManager {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            weights: Mutex::new(Vec::new()),
            last_curr_rate: Mutex::new(0),
            burst_detected: Mutex::new((false, 0)),
        }
    }

    pub async fn push(&self, log: Log) {
        // unwrap for now, handle error later
        let mut lock = self.entries.lock().await;

        for entry in lock.iter() {
            if *entry == log {
                let mut lock_w = self.weights.lock().await;
                let mut insert = false;
                let cl = log.message.clone();

                for (weight, msg) in lock_w.iter_mut() {
                    if cl == msg.as_str() {
                        *weight += 1;
                    } else {
                        insert = true;
                    }
                }

                if insert {
                    lock_w.push((1, cl));
                }

                return;
            }
        }

        lock.push(log);
    }

    pub async fn get_curr_rate_per_second(&self) -> usize {
        let prev = self.entires_processed().await;

        sleep(Duration::from_secs(1)).await;

        let new_len = self.entires_processed().await;

        let rate = new_len - prev;

        let mut lock = self.last_curr_rate.lock().await;

        *lock = rate;

        if rate > 5000 {
            *self.burst_detected.lock().await = (true, rate);
            let mut lock = self.entries.lock().await;
            let cap = lock.capacity();

            lock.reserve(5000);

            println!("Increased buffer capacity");
        }

        rate
    }

    pub async fn entires_processed(&self) -> usize {
        self.entries.lock().await.len()
    }

    pub async fn last_curr_rate(&self) -> usize {
        *self.last_curr_rate.lock().await
    }

    pub async fn pattern_analysis(&self) -> (usize, usize, usize) {
        let mut debug = 0;
        let mut err = 0;
        let mut info = 0;

        for entry in self.entries.lock().await.iter() {
            match entry.log_type {
                TypeOfLog::Debug => debug += 1,
                TypeOfLog::Error => err += 1,
                TypeOfLog::Info => info += 1,
                _ => (),
            }
        }

        (debug, err, info)
    }

    pub async fn detect_burst(&self) -> Option<usize> {
        let mut lock = self.burst_detected.lock().await;

        if lock.0 {
            lock.0 = false;

            Some(lock.1)
        } else {
            None
        }
    }

    pub async fn print_statistics(&self, window: u64, peak_rate: usize) {
        let all = self.entires_processed().await as f64;

        println!("Log Analysis Report (Last Updated: {})", Local::now());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Runtime Stats:");
        println!("Entries Processed: {}", all);
        println!(
            "Current Rate: {} entires/sec (Peak: {} entires/sec)",
            self.last_curr_rate().await,
            peak_rate
        );
        println!("Adaptive window: {} sec", window);
        println!();

        let (debug, err, info) = self.pattern_analysis().await;

        let debug_per = (debug as f64 / all) * 100.0;
        let err_per = (err as f64 / all) * 100.0;
        let info_per = (info as f64 / all) * 100.0;

        println!("Pattern Analysis: ");
        println!("Error: {}% ({} entires)", debug_per, err);
        println!("Debug: {}% ({} entires) ", err_per, debug);
        println!("Info: {}% ({} entires)", info_per, info);

        println!();

        println!("Self Evolving alerts");

        if let Some(x) = self.detect_burst().await {
            println!("Burst detected!: {} entries in 1 sec", x);
        }

        if err_per > info_per && err_per > debug_per {
            println!("High error rate: {}", err_per);
        }

        println!();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeOfLog {
    Error,
    Info,
    Debug,
    Uncategorized,
}

impl FromStr for TypeOfLog {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "error" => TypeOfLog::Error,
            "debug" => TypeOfLog::Debug,
            "info" => TypeOfLog::Info,
            _ => TypeOfLog::Uncategorized,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Log {
    pub message: String,
    pub log_type: TypeOfLog,
    pub timestamp: DateTime<Utc>,
}

// parse log from string
pub fn parse_log(log: String) -> Log {
    let maybe_err = &log[23..28].to_lowercase();
    let maybe_info = &log[23..27].to_lowercase();

    let log_type = match TypeOfLog::from_str(maybe_err) {
        Ok(TypeOfLog::Uncategorized) => {
            // check if it can be info, if its not error and debug
            if let Ok(x) = TypeOfLog::from_str(maybe_info) {
                x
            } else {
                TypeOfLog::Uncategorized
            }
        }
        Ok(x) => x,
        Err(_) => TypeOfLog::Uncategorized,
    };

    let timestamp = log[1..21].parse::<DateTime<Utc>>().unwrap_or_default();

    let message = {
        match log_type {
            TypeOfLog::Info => &log[30..],
            TypeOfLog::Debug | TypeOfLog::Error => &log[31..],
            _ => &log,
        }
    };

    Log {
        message: message.to_string(),
        log_type,
        timestamp,
    }
}
