http-stress-test
================

Dumb concurrent requests of an url. written in rust

HTTP Stress testing 0.1.0
Do some http requets with hopefully high rates

    USAGE:
        http-stress-test [OPTIONS] <URL>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -d, --delay <delay>        Delay between request in ms (default: 100)
        -w, --workers <workers>    Number of worker threads (default: 1)

    ARGS:
        <URL>    URL to request


