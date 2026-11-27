//! Replay completion workloads with bounded concurrency and ordered trial records.
mod stream;
use anyhow::{Context, Result, ensure};
use clap::Parser;
use futures_util::{StreamExt, stream as futures};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

/// Replay one workload and write the full trial records to a new directory.
#[derive(Parser)]
#[command(version, about)]
struct Options {
    #[arg(long)]
    engine: String,
    #[arg(long)]
    model: String,
    #[arg(long)]
    endpoint: String,
    #[arg(long)]
    config: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    workload: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    concurrency: usize,
    #[arg(long, default_value_t = 1)]
    repetitions: usize,
    #[arg(long, default_value_t = 120.0)]
    timeout: f64,
}

/// Read input rows and validate their IDs before starting a run.
fn load_workload(bytes: &[u8]) -> Result<Vec<Value>> {
    let mut trials = Vec::new();
    let mut seen = HashSet::new();
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|s| !s.trim().is_empty())
    {
        let row: Value = serde_json::from_str(line)?;
        let id = row["id"].as_str().context("Missing string id")?;
        ensure!(
            !id.is_empty() && seen.insert(id.to_owned()),
            "Invalid or duplicate trial ID"
        );
        ensure!(
            row["prompt"].is_string() && row["max_tokens"].as_u64().unwrap_or(0) > 0,
            "Invalid prompt or token limit"
        );
        trials.push(row);
    }
    ensure!(!trials.is_empty(), "Empty workload");
    Ok(trials)
}

/// Keep the wire payload and result schema compatible with archived runs.
async fn run_trial(
    client: &reqwest::Client,
    opts: &Options,
    mut row: Value,
    repetition: usize,
) -> Value {
    let start = Instant::now();
    let mut obs = stream::Observation::default();
    let result = async {
        let response = client
            .post(&opts.endpoint)
            .json(&json!({
                "model":opts.model,"prompt":row["prompt"],"max_tokens":row["max_tokens"],
                "temperature":0,"stream":true,"stream_options":{"include_usage":true}
            }))
            .send()
            .await?;
        obs.receive(response, start).await
    }
    .await;
    row["elapsed_s"] = json!(start.elapsed().as_secs_f64());
    row["repetition"] = json!(repetition);
    row["model"] = json!(opts.model);
    row["ok"] = json!(result.is_ok());
    if let Err(err) = result {
        row["error"] = json!(format!("{err:#}"));
    }
    row["response"] = obs.response.into();
    row["first_text_s"] = obs.first_text_s.into();
    row["completion_tokens"] = obs.completion_tokens.into();
    row["finish_reason"] = obs.finish_reason.into();
    row
}

/// Compute nearest-rank percentiles for successful measurements.
fn percentiles(values: &mut [f64]) -> Value {
    values.sort_by(f64::total_cmp);
    let at = |p: usize| (!values.is_empty()).then(|| values[(values.len() * p).div_ceil(100) - 1]);
    json!({"p50":at(50),"p95":at(95),"p99":at(99)})
}

/// Flush each JSON document before returning to the caller.
fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    serde_json::to_writer_pretty(&mut file, value)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

async fn run(opts: &Options) -> Result<bool> {
    ensure!(
        opts.concurrency > 0 && opts.repetitions > 0 && opts.timeout > 0.0,
        "Expected positive concurrency, repetitions, and timeout"
    );
    let timeout = Duration::try_from_secs_f64(opts.timeout)?;
    let client = reqwest::Client::builder()
        .connect_timeout(timeout)
        .read_timeout(timeout)
        .build()?;
    let bytes = match &opts.workload {
        Some(p) => fs::read(p)?,
        None => include_bytes!("../inference_teardown/workload.jsonl").to_vec(),
    };
    let rows = load_workload(&bytes)?;
    let total = rows
        .len()
        .checked_mul(opts.repetitions)
        .context("Too many trials")?;
    let config: Value = serde_json::from_slice(&fs::read(&opts.config)?)?;
    for key in [
        "engine_revision",
        "model_revision",
        "tokenizer_revision",
        "hardware",
        "precision",
    ] {
        ensure!(
            config[key].as_str().is_some_and(|s| !s.trim().is_empty()),
            "Missing config field: {key}"
        );
    }
    if let Some(parent) = opts.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&opts.out).context("Choose a fresh output directory")?;
    write_json(
        &opts.out.join("manifest.json"),
        &json!({
            "schema_version": 1,
            "engine": opts.engine,
            "model": opts.model,
            "endpoint": opts.endpoint,
            "concurrency": opts.concurrency,
            "repetitions": opts.repetitions,
            "timeout": opts.timeout,
            "config": config,
            "workload_sha256": format!("{:x}", Sha256::digest(bytes)),
        }),
    )?;
    let start = Instant::now();
    let mut out = BufWriter::new(File::create(opts.out.join("trials.jsonl"))?);
    let mut latency = Vec::new();
    let mut first = Vec::new();
    let mut tokens = Some(0u64);
    let mut next = 0;
    let mut pending = BTreeMap::new();
    let mut results = futures::iter(0..total)
        .map(|i| {
            let trial = run_trial(
                &client,
                opts,
                rows[i % rows.len()].clone(),
                i / rows.len() + 1,
            );
            async move { (i, trial.await) }
        })
        .buffer_unordered(opts.concurrency);
    while let Some((i, row)) = results.next().await {
        pending.insert(i, row);
        while let Some(row) = pending.remove(&next) {
            serde_json::to_writer(&mut out, &row)?;
            writeln!(out)?;
            out.flush()?;
            if row["ok"] == true {
                latency.push(row["elapsed_s"].as_f64().unwrap());
                if let Some(t) = row["first_text_s"].as_f64() {
                    first.push(t);
                }
                tokens = tokens
                    .zip(row["completion_tokens"].as_u64())
                    .and_then(|(sum, n)| sum.checked_add(n));
            }
            next += 1;
        }
    }
    let wall = start.elapsed().as_secs_f64();
    let success = latency.len();
    write_json(
        &opts.out.join("summary.json"),
        &json!({
            "requests": total,
            "succeeded": success,
            "wall_s": wall,
            "requests_per_s": success as f64 / wall,
            "latency_s": percentiles(&mut latency),
            "first_text_s": percentiles(&mut first),
            "output_tokens_per_s": tokens.filter(|_| success > 0).map(|n| n as f64 / wall),
        }),
    )?;
    Ok(success == total)
}

#[tokio::main]
async fn main() -> Result<ExitCode> {
    Ok(if run(&Options::parse()).await? {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
