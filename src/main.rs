extern crate clap;
extern crate futures;
extern crate reqwest;
extern crate tokio;

use clap::{App, Arg};

use std::time::Duration;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::{Future, Stream};
use reqwest::async::{ClientBuilder, Decoder};
use reqwest::RedirectPolicy;
use std::mem;
use tokio::timer::Interval;

use futures::future;

struct Config {
    max_concurrency: usize,
    target_rate: u64,
    follow_redir: usize,
    debug: bool,
}

fn main() {
    let matches = App::new("HTTP Stress testing")
        .version("0.2.1")
        .about("Do some http requets with hopefully high rates")
        .arg(
            Arg::with_name("max_concurrency")
                .short("m")
                .long("max-concurrency")
                .takes_value(true)
                .help("Maximum number of concurrent requests (default 1)"),
        ).arg(
            Arg::with_name("target_rate")
                .short("t")
                .long("target-rate")
                .takes_value(true)
                .help("Target requests rate (in req/s, default 1)"),
        ).arg(
            Arg::with_name("follow_redir")
                .short("f")
                .long("follow-redirects")
                .takes_value(true)
                .help("Max number of redirections to follow (default 0: do not follow)"),
        ).arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("Debug mode: print out some response attributes"),
        ).arg(Arg::with_name("URL").help("URL to request").required(true))
        .get_matches();

    let debug = matches.is_present("debug");

    let follow_redir = match matches.value_of("follow_redir") {
        None => 0,
        Some(n) => n
            .parse::<usize>()
            .expect("follow-redirects must be a positive integer"),
    };

    let max_concurrency = matches.value_of("max_concurrency");
    let max_concurrency = match max_concurrency {
        None => 1,
        Some(s) => s
            .parse::<usize>()
            .expect("max_concurrency need to be an int"),
    };
    let target_rate = matches.value_of("target_rate");
    let target_rate = match target_rate {
        None => 1,
        Some(s) => s.parse::<u64>().expect("target_rate need to be an int"),
    };
    let url = String::from(matches.value_of("URL").unwrap());

    let config = Config {
        max_concurrency,
        target_rate,
        follow_redir,
        debug,
    };
    lets_do_some_requests(url, config);
}

fn lets_do_some_requests(url: String, config: Config) {
    println!("Press Ctrl+C to stop this program");

    let url = Arc::new(url);
    let nb_requests_w = Arc::new(AtomicUsize::new(0));
    let nb_requests_r = nb_requests_w.clone();
    let inflight_requests_w = Arc::new(AtomicUsize::new(0));
    let inflight_requests_r = inflight_requests_w.clone();

    let timing_nano = 1_000_000_000 / config.target_rate;

    tokio::run({
        future::ok(())
            .and_then(move |_| {
                // need to spawn here otherwise, the progress_counter intervall won't start
                tokio::spawn({
                    Interval::new_interval(Duration::from_nanos(timing_nano))
                        .map_err(|e| panic!("timer failed; err={:?}", e))
                        .for_each(move |_| {
                            requestor(&url, &config, &nb_requests_w, &inflight_requests_w)
                        })
                })
            }).and_then(|_| {
                Interval::new_interval(Duration::from_secs(1))
                    .map_err(|e| panic!("timer failed; err={:?}", e))
                    .for_each(move |_| {
                        progress_counter(nb_requests_r.as_ref(), inflight_requests_r.as_ref())
                    })
            })
    });
}

fn progress_counter(
    nb_requests: &AtomicUsize,
    inflight_requests: &AtomicUsize,
) -> impl Future<Item = (), Error = ()> {
    println!(
        "Total requests: {}\tIn-flight requests: {}",
        nb_requests.load(Ordering::Relaxed),
        inflight_requests.load(Ordering::Relaxed)
    );
    future::ok(())
}

fn requestor(
    url: &String,
    config: &Config,
    nb_requests: &Arc<AtomicUsize>,
    inflight_requests: &Arc<AtomicUsize>,
) -> impl Future<Item = (), Error = ()> {
    if inflight_requests.load(Ordering::Relaxed) < config.max_concurrency {
        inflight_requests.fetch_add(1, Ordering::Relaxed);
        let inflight_requests = inflight_requests.clone();
        let inflight_requests_err = inflight_requests.clone();
        let nb_requests_w = nb_requests.clone();
        tokio::spawn({
            fetch(url, build_client_builder(config.follow_redir), config.debug)
                .map_err(move |_| {
                    inflight_requests_err.fetch_sub(1, Ordering::Relaxed);
                }).map(move |_| {
                    // request done !
                    inflight_requests.fetch_sub(1, Ordering::Relaxed);
                    nb_requests_w.fetch_add(1, Ordering::Relaxed);
                })
        });
    }
    future::ok(())
}

fn fetch(
    url: &String,
    client_builder: ClientBuilder,
    debug: bool,
) -> impl Future<Item = (), Error = ()> {
    client_builder
        .build()
        .unwrap()
        .get(url)
        .send()
        .and_then(move |mut res| {
            if debug {
                println!("---- Response status {}", res.status());
                println!("---- Response headers:");

                for (key, value) in res.headers().iter() {
                    println!("{}: {:?}", key, value);
                }
            }
            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        }).map_err(|err| println!("request error: {}", err))
        .map(|_body| {})
}

fn build_client_builder(follow_redir: usize) -> ClientBuilder {
    ClientBuilder::new().redirect(RedirectPolicy::custom(move |attempt| {
        if attempt.previous().len() > follow_redir {
            attempt.stop()
        } else {
            attempt.follow()
        }
    }))
}
