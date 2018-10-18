http-stress-test
================

Dumb concurrent requests of an url. written in rust.
Demonstrates how to do async stuff with tokio & reqwest.

    HTTP Stress testing 0.2.0
    Do some http requets with hopefully high rates

    USAGE:
        http-stress-test [OPTIONS] <URL>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -m, --max_concurrency <max_concurrency>    Maximum number of concurrent requests
        -t, --target_rate <target_rate>            Target requests rate (in req/s)

    ARGS:
        <URL>    URL to request


