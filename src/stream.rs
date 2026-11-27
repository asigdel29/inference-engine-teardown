//! Parse server-sent events and retain measurements from incomplete trials.
use anyhow::{Result, bail, ensure};
use futures_util::TryStreamExt;
use serde::Deserialize;
use std::{io, time::Instant};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};
use tokio_util::io::StreamReader;

/// Accumulate output and engine usage independently of the final trial status.
#[derive(Default)]
pub struct Observation {
    pub response: String,
    pub first_text_s: Option<f64>,
    pub completion_tokens: Option<u64>,
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Default)]
struct Event {
    #[serde(default)]
    choices: Vec<Choice>,
    usage: Option<Usage>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    text: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    completion_tokens: Option<u64>,
}

impl Observation {
    /// Consume one response and retain its partial output if the peer disconnects.
    pub async fn receive(&mut self, response: reqwest::Response, start: Instant) -> Result<()> {
        let chunks = response
            .error_for_status()?
            .bytes_stream()
            .map_err(io::Error::other);
        self.parse(BufReader::new(StreamReader::new(chunks)), start)
            .await
    }

    async fn parse(&mut self, input: impl AsyncBufRead + Unpin, start: Instant) -> Result<()> {
        let mut lines = input.lines();
        let mut data = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Some(field) = line.strip_prefix("data:") {
                data.push(field.strip_prefix(' ').unwrap_or(field).to_owned());
            } else if line.is_empty() && !data.is_empty() {
                let payload = data.join("\n");
                data.clear();
                if payload == "[DONE]" {
                    ensure!(
                        self.finish_reason.is_some(),
                        "Stream ended without a finish reason"
                    );
                    return Ok(());
                }
                let event: Event = serde_json::from_str(&payload)?;
                if let Some(error) = event.error {
                    bail!("Engine error: {error}");
                }
                if let Some(count) = event.usage.and_then(|u| u.completion_tokens) {
                    self.completion_tokens = Some(count);
                }
                for choice in event.choices {
                    if let Some(text) = choice.text.filter(|s| !s.is_empty()) {
                        self.first_text_s
                            .get_or_insert_with(|| start.elapsed().as_secs_f64());
                        self.response.push_str(&text);
                    }
                    if let Some(reason) = choice.finish_reason.filter(|s| !s.is_empty()) {
                        self.finish_reason = Some(reason);
                    }
                }
            }
        }
        bail!("Incomplete SSE stream")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn multiline_and_cut_stream() {
        let prefix = ": comment\r\ndata: {\"choices\":\r\ndata: [{\"text\":\"abc\",\"finish_reason\":\"stop\"}]}\r\n\r\n";
        for tail in ["", "data: [DONE]\n\n"] {
            let mut result = Observation::default();
            let input = format!("{prefix}{tail}");
            assert_eq!(
                result.parse(input.as_bytes(), Instant::now()).await.is_ok(),
                !tail.is_empty()
            );
            assert_eq!(result.response, "abc");
            assert!(result.first_text_s.is_some());
        }
    }
}
