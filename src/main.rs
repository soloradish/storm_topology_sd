mod error;
mod file_sd;
mod storm_client;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::*;

use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
#[macro_use]
extern crate clap;
use clap::{App, Arg};
use clokwerk::Interval::*;
use std::collections::HashSet;
use std::fs::File;
use std::thread;
use std::time::Duration;

use crate::file_sd::FileSD;
use crate::storm_client::{StormRestClient, TopologyWorkers};

// Create a new scheduler
fn main() {
    let matches = App::new("Storm Topology SD for Prometheus")
        .version("0.1.0")
        .arg(
            Arg::with_name("storm_ui_url")
                .short("t")
                .long("storm_ui_url")
                .value_name("URL")
                .help("Storm UI base url with http schema, eg. http://stormui.com/")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("sd_file_path")
                .short("f")
                .long("sd_file_path")
                .value_name("PATH")
                .help("Path of Prometheus file_sd's file, eg. /etc/prometheus/file_sd/storm_sd .")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("cluster_name")
                .short("n")
                .long("cluster_name")
                .value_name("NAME")
                .help("cluster name in Prometheus target lables, default `storm_cluster`.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("SECOND")
                .help("interval between scrape storm ui service, Default `30`s.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("verbosity level, eg: -v or -vvv"),
        )
        .get_matches();

    let storm_url = matches.value_of("storm_ui_url").unwrap();
    let sd_file = matches.value_of("sd_file_path").unwrap().to_string();
    let cluster_name = matches
        .value_of("cluster_name")
        .unwrap_or("storm_cluster")
        .to_string();
    let interval = value_t!(matches.value_of("interval"), u32).unwrap_or(30);

    let log_level_filter = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 | _ => LevelFilter::Debug,
    };

    CombinedLogger::init(vec![TermLogger::new(
        log_level_filter,
        Config::default(),
        TerminalMode::Stdout,
    )
    .unwrap()])
    .unwrap();

    debug!("Starting storm_sd service...");
    debug!("    With Config: [storm_url]: {}", storm_url);
    debug!("    With Config: [sd_file_path]: {}", sd_file);
    debug!("    With Config: [cluster_name]: {}", cluster_name);
    debug!("    With Config: [interval]: {}s", interval);

    {
        debug!("Checking sd_file exist & permissions...");
        let open_file = File::create(&sd_file).expect("fail when check sd_file permission.");
    }

    debug!("Starting cronjob scheduler...");
    let mut scheduler = Scheduler::new();
    // or a scheduler with a given timezone
    debug!("Set scheduler tz: UTC.");
    let mut scheduler = Scheduler::with_tz(chrono::Utc);

    let client = StormRestClient::new(storm_url);
    let mut last_sd = HashSet::new();

    scheduler.every(interval.seconds()).run(move || {
        let topology_ids = client.list_active_topologies();
        match topology_ids {
            Ok(ids) => {
                let workers: Vec<TopologyWorkers> = ids
                    .iter()
                    .map(|x| client.get_topology_workers(x).unwrap())
                    .collect();

                let mut current_sd: HashSet<FileSD> = workers
                    .iter()
                    .map(|x| {
                        let mut s = FileSD::from(x.clone());
                        s.add_label("cluster", &cluster_name);
                        s
                    })
                    .collect();

                if last_sd == current_sd {
                    debug!("topology status not change.");
                } else {
                    last_sd = current_sd.clone();
                    info!("find topology status changed.");
                    let wr =
                        serde_json::to_writer_pretty(&File::create(&sd_file).unwrap(), &last_sd);
                    match wr {
                        Ok(ok) => info!("success update sd file."),
                        Err(error) => warn!("failed update sd file. cause: {:?}", error),
                    }
                }
            }
            Err(error) => {
                warn!("Fail to get active topologies. cause: {}", error);
            }
        }
    });

    let thread_handle = scheduler.watch_thread(Duration::from_millis(100));
    info!("Started storm_sd service.");

    loop {
        thread::sleep(Duration::from_millis(500));
    }

    //    thread_handle.stop();
}
