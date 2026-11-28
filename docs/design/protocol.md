# Completion stream protocol

This document defines the interface between the Rust client and one engine. See the [request builder](../../src/main.rs) and [response parser](../../src/stream.rs).

## Request

The client sends an HTTP POST to the exact URL supplied by the operator. It sends one prompt per request. The engine name identifies the experiment in its manifest. The model name selects the model at the server.

```json
{"model":"example","prompt":"Hello","max_tokens":8,"temperature":0,"stream":true,"stream_options":{"include_usage":true}}
```

## Response

The client reads UTF-8 lines from the response. It joins successive `data:` fields with a newline. A blank line ends an event. The following sequence is one complete response.

```text
data: {"choices":[{"text":"Hello","finish_reason":null}]}

data: {"choices":[{"text":"","finish_reason":"stop"}],"usage":{"completion_tokens":1}}

data: [DONE]

```

The first nonempty text sets the first-text timestamp. Each choice contributes its text to the trial response. A usage-only event can arrive after the finish reason.

## Failure behavior

The server must supply a finish reason before the terminal event. An HTTP error, an engine error, invalid event data, or a premature disconnect fails that trial. The writer preserves the text already received. Other trials continue. The command reports failure if any trial fails.
