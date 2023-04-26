# In order to evaluate the cost of JSON conversion. This binding provides implements two subcalls calling the same demo api with different

data converters:

* one call with no data (api/probe)
* one subcall in json mode
* one subcall in binary mode

For 10000 loop with an elementary data type (2*u32+1*String)

* no-data: 252 (cost of micro-service architecture)
* jsonc  : 541-251= 289ms (1 input + 1 output)
* direct : 290-251= 38ms  (1 input + 1 output)

The cost on conversion depends deeply on data object complexity, but even on very simple structure as the one used for the test, the difference is huge:

* ~15ns per conversion for json (I7-Intel Desktop)
* ~ 2ns for direct mode

The second outcome of this bench is about AFB microservice performance with the elementary cost of a raw call. The performance obviously depends
on debug/monitoring level and in a lesser extend from the number of security privilege to check.

* ~25ns per API call (I7-Intel desktop)

```bash
./examples/bench/etc/binding-bench.sh

INFO: [API tap-test] callsync idx:1 tap->uid:no-data afb-api->'/loop-bench/nodata-convert'
NOTICE: [REQ/API loop-bench] no-data loop:10000 duration:252.355309ms

INFO: [API tap-test] callsync idx:2 tap->uid:json-args afb-api->'/loop-bench/json-convert'
NOTICE: [REQ/API loop-bench] json converter loop:10000 duration:541.442557ms

INFO: [API tap-test] callsync idx:3 tap->uid:binary-args afb-api->'/loop-bench/lazy-convert'
NOTICE: [REQ/API loop-bench] direct converter loop:10000 duration:290.409269ms

1..2 # autostart
ok 1 - json-args
ok 2 - binary-args
```
