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
use reqwest::async::{Client, ClientBuilder, Decoder};
use reqwest::RedirectPolicy;
use std::mem;
use tokio::timer::Interval;

use futures::future;

fn main() {
    let matches = App::new("HTTP Stress testing")
        .version("0.2.0")
        .author("Philippe GASSMANN <philoops@gmail.com>")
        .about("Do some http requets with hopefully high rates")
        .arg(
            Arg::with_name("max_concurrency")
                .short("m")
                .long("max_concurrency")
                .takes_value(true)
                .help("Maximum number of concurrent requests"),
        ).arg(
            Arg::with_name("target_rate")
                .short("t")
                .long("target_rate")
                .takes_value(true)
                .help("Target requests rate (in req/s)"),
        ).arg(Arg::with_name("URL").help("URL to request").required(true))
        .get_matches();

    let max_concurrency = matches.value_of("max_concurrency");
    let max_concurrency = match max_concurrency {
        None => 1,
        Some(s) => match s.parse::<usize>() {
            Ok(n) => n,
            Err(_) => panic!("max_concurrency need to be an int"),
        },
    };
    let target_rate = matches.value_of("target_rate");
    let target_rate = match target_rate {
        None => 100,
        Some(s) => match s.parse::<u64>() {
            Ok(n) => n,
            Err(_) => panic!("target_rate need to be an int"),
        },
    };
    let url = Arc::new(String::from(matches.value_of("URL").unwrap()));
    println!("Press Ctrl+C to stop this program");

    let nb_requests_w = Arc::new(AtomicUsize::new(0));
    let nb_requests_r = nb_requests_w.clone();
    let inflight_requests_w = Arc::new(AtomicUsize::new(0));
    let inflight_requests_r = inflight_requests_w.clone();

    let timing_nano = 1_000_000_000 / target_rate;

    tokio::run({
        future::ok(())
            .and_then(move |_| {
                tokio::spawn({
                    Interval::new_interval(Duration::from_nanos(timing_nano))
                        .map_err(|e| panic!("timer failed; err={:?}", e))
                        .for_each(move |_| {
                            if inflight_requests_w.load(Ordering::Relaxed) < max_concurrency {
                                inflight_requests_w.fetch_add(1, Ordering::Relaxed);
                                let inflight_requests = inflight_requests_w.clone();
                                let nb_requests_w = nb_requests_w.clone();
                                tokio::spawn({
                                    fetch(url.as_ref()).map(move |_| {
                                        // request done !
                                        inflight_requests.fetch_sub(1, Ordering::Relaxed);
                                        nb_requests_w.fetch_add(1, Ordering::Relaxed);
                                    })
                                });
                            }
                            future::ok(())
                        })
                })
            }).and_then(|_| {
                Interval::new_interval(Duration::from_secs(1))
                    .map_err(|e| panic!("timer failed; err={:?}", e))
                    .for_each(move |_| {
                        println!(
                            "Total requests: {}\tIn-flight requests: {}",
                            nb_requests_r.load(Ordering::Relaxed),
                            inflight_requests_r.load(Ordering::Relaxed)
                        );
                        future::ok(())
                    })
            })
    });
}
fn fetch(url: &String) -> impl Future<Item = (), Error = ()> {
    build_cliend()
        .get(url)
        .send()
        .and_then(|mut res| {
            let body = mem::replace(res.body_mut(), Decoder::empty());
            body.concat2()
        }).map_err(|err| println!("request error: {}", err))
        .map(|_body| {})
}

fn build_cliend() -> Client {
    ClientBuilder::new()
        .redirect(RedirectPolicy::none())
        .build()
        .unwrap()
}
