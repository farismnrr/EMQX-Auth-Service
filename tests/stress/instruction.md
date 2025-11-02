## Load / Stress Test Instructions

Purpose: Reproduce and run the load tests that produced `wrk_c2000.txt` and `hey_get_local.txt`, save outputs, and provide basic troubleshooting steps.

Note: Run tests in an isolated environment (staging) when possible. High-concurrency tests will place significant load on the system.

---

## Prerequisites

- The application server must be running and listening on `http://localhost:5500`.
- The test machine must have network access to the server and sufficient CPU/memory resources.
- Ensure write access to the output directory (example: `tests/stress/`

---

## Tool installation

1. wrk (build from source)

```zsh
git clone https://github.com/wg/wrk.git /tmp/wrk
cd /tmp/wrk
make
sudo cp wrk /usr/local/bin
wrk --version
```

2. hey (Go)

```zsh
# Requires Go toolchain installed
export GOPATH=$HOME/go
export PATH=$PATH:$GOPATH/bin
go install github.com/rakyll/hey@latest
hey -version
```

Alternative: use the distribution package manager if available (for example `sudo apt install wrk`).

---

## Reproduce the attached test runs

1. Reproduce `wrk_c2000.txt`

```zsh
# full threads, 2000 connections, duration 10s
wrk -c2000 -d10s http://localhost:5500 2>&1 | tee wrk_c2000.txt
```

Explanation: `-c2000` = concurrent connections, `-d10s` = duration.

2. Reproduce `hey_get_local.txt` (approximately 1,000,000 requests or ~108s duration)

```zsh
# Option A: total requests
hey -n 1000000 -c 200 -timeout 60s http://localhost:5500 2>&1 | tee hey_get_local.txt

# Option B: duration-based (~108s)
hey -z 108s -c 200 -timeout 60s http://localhost:5500 2>&1 | tee hey_get_local.txt
```

Explanation: `-n` = total requests, `-c` = concurrency, `-z` = duration, `-timeout` = client timeout.

---

## Monitoring during tests

Run these commands in a separate terminal to monitor system and network state:

```zsh
htop                    # CPU / memory
iotop -o                # disk I/O (root may be required)
ss -s                   # socket summary
ss -ltnp | grep 5500    # listener details for port 5500
ulimit -n               # check file descriptor limit
```

---

## Reading and interpreting results

- Requests/sec (RPS): average throughput. A high RPS combined with errors indicates failed requests.
- Latency percentiles (p50, p75, p90, p95, p99): measure response distribution; p95/p99 show long-tail behavior.
- Socket errors (e.g. `connect X` in `wrk`): failures to establish connections, frequently caused by OS limits or backlog constraints.
- Timeout errors (e.g. `Client.Timeout exceeded` in `hey`): client-side timeouts when the server does not respond within the configured timeout.

Examples from the attached outputs:

- `wrk_c2000.txt`: ~31k RPS with 983 connect errors — indicates connection acceptance limits (backlog/ulimit) or transient network contention.
- `hey_get_local.txt`: ~9.2k RPS average with 1025 timeouts — indicates long server-side processing or resource contention causing delayed responses.

---

## Recommended test procedure (ramp-up)

1. Run a lightweight baseline:

```zsh
hey -n 10000 -c 50 http://localhost:5500
```

2. Increase load in steps: 50 → 100 → 200 → 500 concurrency, monitoring CPU/MEM/IO at each step.
3. Record the concurrency level where errors or long-tail latency appear and focus tuning efforts at that point.
