http-stress-test
================

Dumb concurrent requests of an url. written in rust.
Demonstrates how to do async stuff with tokio & reqwest.


    HTTP Stress testing 0.2.1
    Do some http requets with hopefully high rates

    USAGE:
        http-stress-test [FLAGS] [OPTIONS] <URL>

    FLAGS:
        -d, --debug      Debug mode: print out some response attributes
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -f, --follow-redirects <follow_redir>      Max number of redirection to follow (default 0: do not follow)
        -m, --max-concurrency <max_concurrency>    Maximum number of concurrent requests (default 1)
        -t, --target-rate <target_rate>            Target requests rate (in req/s, default 1)

    ARGS:
        <URL>    URL to request


