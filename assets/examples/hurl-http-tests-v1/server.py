#!/usr/bin/env python3
"""Original Built with Rust fixture; Python 3 standard library only.

Verified with Hurl 8.0.1; syntax: https://hurl.dev/docs/asserting-response.html
Run: python3 server.py --mode healthy (Ctrl+C stops it).
Only 127.0.0.1:8765 is bound. No files, credentials, or persistent state are used.
"""

import argparse
from http.server import BaseHTTPRequestHandler, HTTPServer


MODES = ("healthy", "wrong-json", "wrong-status", "wrong-header", "malformed-json")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=MODES, default="healthy")
    mode = parser.parse_args().mode

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path != "/health":
                self.send_response(404)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return

            body = b'{"status":"ok","items":2}'
            status = 200
            content_type = "application/json; charset=utf-8"
            if mode == "wrong-json":
                body = b'{"status":"error","items":2}'
            elif mode == "wrong-status":
                status = 503
            elif mode == "wrong-header":
                content_type = "text/plain; charset=utf-8"
            elif mode == "malformed-json":
                body = b'{"status":"ok","items":'

            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format, *args):
            pass

    with HTTPServer(("127.0.0.1", 8765), Handler) as server:
        print(f"Fixture: http://127.0.0.1:8765/health ({mode}); Ctrl+C to stop", flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
