# Load test baseline

Machine: Apple M2 Max, macOS 26.5.2. Release build, seeded dev DB, `oha -z 10s -c 64`, brotli accepted.
Recorded 2026-10-01 at commit after "cache before DB" fix.

| path | req/s | p99 |
|---|---|---|
| / | 135,676 | 1.89 ms |
| /projects/zed | 138,317 | 1.81 ms |
| /categories/developer-tools | 139,104 | 1.75 ms |

Numbers are for cache hits, which is the steady state. A purge costs one
render + compress per page on the next request (low single-digit ms).

Update this table when a change moves these by more than ~10% either way.

## 2026-10-08 Sentry overhead check

Shared Linux agent host (not the M2 baseline above), release build, 2 Tokio
workers, oha 1.16.0, 16 connections, 5 seconds/path, warmed cache, Brotli.
Same monitoring build with DSN absent vs enabled; local HTTP sink prevents
benchmark traffic reaching Sentry. Other shared-host work makes this indicative,
not an isolated performance guarantee.

| Path | Sentry | Requests/s | p99 (ms) | Success |
|---|---|---:|---:|---:|
| / | disabled | 70104 | 0.452 | 100% |
| /projects/zed | disabled | 69079 | 0.423 | 100% |
| /categories/developer-tools | disabled | 63055 | 0.475 | 100% |
| / | enabled | 39633 | 0.900 | 100% |
| /projects/zed | enabled | 42918 | 0.722 | 100% |
| /categories/developer-tools | enabled | 39326 | 0.764 | 100% |

Telemetry lowers saturated throughput by approximately 38–43%, while all
measured p99 latencies remain below 1 ms. This is an explicit cost of per-request
metrics and isolated tracing, not a claim of zero overhead. Public cache hits
still avoid SQL/render/compression and no Sentry network calls are awaited.
Health/static traffic bypass telemetry. Revisit sampling if sustained traffic
approaches these limits.
