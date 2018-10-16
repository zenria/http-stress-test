extern crate clap;
extern crate reqwest;

use clap::{App, Arg};

use reqwest::Result;
use std::thread;
use std::time::Duration;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

fn main() {
    let matches = App::new("HTTP Stress testing")
        .version("0.1.0")
        .author("Philippe GASSMANN <philoops@gmail.com>")
        .about("Do some http requets with hopefully high rates")
        .arg(
            Arg::with_name("workers")
                .short("w")
                .long("workers")
                .takes_value(true)
                .help("Number of worker threads (default: 1)"),
        ).arg(
            Arg::with_name("delay")
                .short("d")
                .long("delay")
                .takes_value(true)
                .help("Delay between request in ms (default: 100)"),
        ).arg(Arg::with_name("URL").help("URL to request").required(true))
        .get_matches();

    let workers = matches.value_of("workers");
    let workers = match workers {
        None => 1,
        Some(s) => match s.parse::<i32>() {
            Ok(n) => n,
            Err(_) => panic!("workers need to be an int"),
        },
    };
    let delay = matches.value_of("delay");
    let delay = match delay {
        None => 100,
        Some(s) => match s.parse::<u64>() {
            Ok(n) => n,
            Err(_) => panic!("delay need to be an int"),
        },
    };
    let url = matches.value_of("URL").unwrap();
    println!(
        "Starting {} threads doing requests against {}. {} ms between each request.",
        workers, url, delay
    );
    println!("Press Ctrl+C to stop this program");

    let nb_requests = Arc::new(AtomicUsize::new(0));

    launch_workers(
        url.to_string(),
        workers,
        Duration::from_millis(delay),
        &nb_requests,
    );
    let mut s = 0;
    loop {
        s = s + 1;
        let count = nb_requests.load(Ordering::Relaxed);
        let rate = count / s;
        println!("{} req/s {} total", rate, count);
        thread::sleep(Duration::from_secs(1));
    }
}

fn launch_workers(
    url: String,
    workers_count: i32,
    sleep_duration: Duration,
    nb_requests: &Arc<AtomicUsize>,
) {
    for _i in 0..workers_count {
        let nb_requests = nb_requests.clone();
        let url = url.clone();

        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_millis(10000))
            .build()
            .unwrap();

        thread::spawn(move || loop {
            match read_url(&url, &client) {
                Err(e) => println!("Error requesting url {}", e),
                Ok(_) => {}
            }
            nb_requests.fetch_add(1, Ordering::Relaxed);
            thread::sleep(sleep_duration);
        });
    }
}

fn read_url(url: &str, client: &reqwest::Client) -> Result<()> {
    let _response = client.head(url).send()?;
    Ok(())
}
