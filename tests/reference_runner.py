"""Replay a fixed workload against an OpenAI-compatible streaming endpoint."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
from http.client import HTTPException
import json
import math
from pathlib import Path
import sys
import time
from urllib.request import Request, urlopen
from urllib.error import HTTPError


def load_workload(path):
    """Reject ambiguous trials before sending traffic."""
    trials = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    ids = set()
    for trial in trials:
        if not isinstance(trial["id"], str) or not trial["id"] or trial["id"] in ids:
            raise ValueError("Each trial needs a unique string ID")
        if not isinstance(trial["prompt"], str) or type(trial["max_tokens"]) is not int or trial["max_tokens"] < 1:
            raise ValueError("Each trial needs a prompt and a positive token limit")
        ids.add(trial["id"])
    if not trials:
        raise ValueError("Empty workload")
    return trials


def events(response):
    """Read SSE data fields, including comments and multiline events."""
    fields = []
    for raw in response:
        line = raw.decode("utf-8").rstrip("\r\n")
        if line.startswith("data:"):
            fields.append(line[5:].removeprefix(" "))
        elif not line and fields:
            yield "\n".join(fields)
            fields.clear()


def run_trial(trial, endpoint, model, timeout):
    """Capture text, usage, finish reason, and client-observed latency."""
    body = dict(model=model, prompt=trial["prompt"], max_tokens=trial["max_tokens"],
                temperature=0, stream=True, stream_options={"include_usage": True})
    result = dict(trial, model=model, response="", first_text_s=None, completion_tokens=None,
                  finish_reason=None, ok=False)
    started = time.perf_counter()
    try:
        req = Request(endpoint, json.dumps(body).encode(), {"Content-Type": "application/json"})
        with urlopen(req, timeout=timeout) as response:
            for data in events(response):
                if data == "[DONE]":
                    if result["finish_reason"] is None:
                        raise ValueError("Stream ended without a finish reason")
                    result["ok"] = True
                    break
                event = json.loads(data)
                if event.get("error"):
                    raise ValueError(str(event["error"]))
                usage = event.get("usage") or {}
                if "completion_tokens" in usage:
                    result["completion_tokens"] = usage["completion_tokens"]
                for choice in event.get("choices", []):
                    chunk = choice.get("text") or ""
                    if chunk and result["first_text_s"] is None:
                        result["first_text_s"] = time.perf_counter() - started
                    result["response"] += chunk
                    if choice.get("finish_reason"):
                        result["finish_reason"] = choice["finish_reason"]
            else:
                raise ValueError("Incomplete SSE stream")
    except (HTTPException, OSError, ValueError, TypeError, KeyError) as exc:
        result["error"] = str(exc)
        if isinstance(exc, HTTPError):
            exc.close()
    result["elapsed_s"] = time.perf_counter() - started
    return result


def percentiles(values):
    """Return nearest-rank latency percentiles for successful observations."""
    ordered = sorted(values)
    return {f"p{p}": ordered[math.ceil(len(ordered) * p / 100) - 1] if ordered else None
            for p in (50, 95, 99)}


def main():
    """Write a manifest, raw JSONL, and a summary for one engine run."""
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("engine", "model", "endpoint"):
        parser.add_argument("--" + option, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--workload", type=Path, default=Path(__file__).with_name("workload.jsonl"))
    parser.add_argument("--concurrency", type=int, default=1)
    parser.add_argument("--repetitions", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    if args.repetitions < 1 or args.concurrency < 1 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("Repetitions, concurrency and timeout must be positive")
    trials = load_workload(args.workload)
    trials = [dict(trial, repetition=index + 1) for index in range(args.repetitions) for trial in trials]
    config = json.loads(args.config.read_text())
    for key in ("engine_revision", "model_revision", "tokenizer_revision", "hardware", "precision"):
        if not isinstance(config.get(key), str) or not config[key].strip():
            parser.error(f"Missing configuration field: {key}")
    args.out.mkdir(parents=True, exist_ok=False)
    manifest = dict(schema_version=1, engine=args.engine, endpoint=args.endpoint, model=args.model,
                    concurrency=args.concurrency, repetitions=args.repetitions, timeout=args.timeout, config=config,
                    workload_sha256=hashlib.sha256(args.workload.read_bytes()).hexdigest())
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    started = time.perf_counter()
    latency = []
    first_text = []
    tokens = []
    with (args.out / "trials.jsonl").open("w") as output, ThreadPoolExecutor(args.concurrency) as pool:
        for result in pool.map(lambda trial: run_trial(trial, args.endpoint, args.model, args.timeout), trials):
            output.write(json.dumps(result) + "\n")
            output.flush()
            if result["ok"]:
                latency.append(result["elapsed_s"])
                tokens.append(result["completion_tokens"])
                if result["first_text_s"] is not None:
                    first_text.append(result["first_text_s"])
    wall = time.perf_counter() - started
    summary = dict(requests=len(trials), succeeded=len(latency), wall_s=wall,
                   requests_per_s=len(latency) / wall,
                   latency_s=percentiles(latency), first_text_s=percentiles(first_text),
                   output_tokens_per_s=sum(tokens) / wall if tokens and None not in tokens else None)
    (args.out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return int(len(latency) != len(trials))


if __name__ == "__main__":
    sys.exit(main())
