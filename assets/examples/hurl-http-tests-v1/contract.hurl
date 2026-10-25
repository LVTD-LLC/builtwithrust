# Original Built with Rust fixture, verified with Hurl 8.0.1.
# https://hurl.dev/docs/running-tests.html
# https://hurl.dev/docs/asserting-response.html
GET http://127.0.0.1:8765/health
HTTP 200
[Asserts]
header "Content-Type" contains "application/json"
jsonpath "$.status" == "ok"
jsonpath "$.items" == 2
