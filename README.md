# HTTP Load Tester

Generate a load on any server.


## Usage

```
Usage: perf-test [OPTIONS] --url <URL>

Options:
  -u, --url <URL>
          Host to make requests to
  -c, --concurrent-requests <CONCURRENT_REQUESTS>
          How many requests to send at once [default: 10]
  -t, --test-time <TEST_TIME>
          How long the test should last for (seconds) [default: 30]
  -x, --headers <HEADERS>
          Any request headers to send with the request
  -m, --method <METHOD>
          HTTP method to use [default: GET]
  -o, --out-file <OUT_FILE>
          File to write logs to
  -d, --debug
          Perform some additional debug logging
  -h, --help
          Print help information
```
