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
