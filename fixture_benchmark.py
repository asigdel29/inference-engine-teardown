"""Measure client transport overhead with a deterministic local HTTP server."""
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
import sys
import threading


class Fixture(BaseHTTPRequestHandler):
    """Serve deterministic completion streams for transport tests."""

    def log_message(self, *_):
        """Keep fixture requests out of the test output."""

    def do_POST(self):
        """Return a complete response or simulate a closed stream."""
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.end_headers()
        if self.path == "/cut":
            return
        choice = {"index": 0, "text": body["prompt"], "finish_reason": "stop"}
        event = {"choices": [choice], "usage": {"completion_tokens": 7}}
        self.wfile.write(("data: " + json.dumps(event) + "\n\ndata: [DONE]\n\n").encode())


if __name__ == "__main__":
    with ThreadingHTTPServer(("127.0.0.1", 0), Fixture) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            subprocess.run([sys.executable, "-m", "inference_teardown.bench", "--engine", "fixture",
                            "--model", "echo", "--config", "fixture-config.json", "--endpoint",
                            f"http://127.0.0.1:{server.server_port}/ok", "--concurrency", "4",
                            "--repetitions", "20", "--out", sys.argv[1]], check=True)
            print((Path(sys.argv[1]) / "summary.json").read_text())
        finally:
            server.shutdown()
            thread.join()
