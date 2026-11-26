"""Exercise the runner through its CLI with a local streaming server."""
from http.server import ThreadingHTTPServer
from fixture_benchmark import Fixture
from pathlib import Path
import json
import subprocess
import sys
import tempfile
import threading
import unittest

from inference_teardown.bench import events


class RunnerTests(unittest.TestCase):
    """Verify public output, failure retention, and SSE framing."""

    def test_sse(self):
        """Comments and multiline fields retain event boundaries."""
        self.assertEqual(list(events([b": comment\r\n", b"data: one\r\n",
                                     b"data: two\r\n", b"\r\n"])), ["one\ntwo"])

    def test_cli(self):
        """A cut stream produces failed trial records and a failure exit."""
        with ThreadingHTTPServer(("127.0.0.1", 0), Fixture) as server, tempfile.TemporaryDirectory() as tmp:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                for route, code in (("ok", 0), ("cut", 1)):
                    out = Path(tmp) / route
                    command = [sys.executable, "-m", "inference_teardown.bench", "--engine", "fixture", "--model", "echo",
                               "--config", "fixture-config.json", "--concurrency", "2", "--out", str(out),
                               "--endpoint", f"http://127.0.0.1:{server.server_port}/{route}"]
                    run = subprocess.run(command, capture_output=True, text=True)
                    self.assertEqual(run.returncode, code, run.stderr)
                    rows = [json.loads(line) for line in (out / "trials.jsonl").read_text().splitlines()]
                    self.assertEqual(len(rows), 3)
                    summary = json.loads((out / "summary.json").read_text())
                    self.assertEqual(summary["succeeded"], 3 if code == 0 else 0)
                    self.assertEqual(summary["latency_s"]["p95"] is None, code != 0)
                    for row in rows:
                        self.assertEqual(row["ok"], code == 0)
                        if row["ok"]:
                            self.assertEqual(row["response"], row["prompt"])
                            self.assertEqual(row["completion_tokens"], 7)
                            self.assertIsNotNone(row["first_text_s"])
                        else:
                            self.assertIn("Incomplete", row["error"])
            finally:
                server.shutdown()
                thread.join()
