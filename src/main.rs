use std::collections::HashSet;

use clap::Parser;
use fst_native::*;
use serde::Deserialize;
use tracing::{info, trace, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug)]
#[command(
  author = "Avimitin",
  version = "v0.1.0",
  about = "Extract signals from FST file"
)]
struct CliArgs {
  /// File path to the fst file
  #[arg(short, long)]
  fst: String,
  /// File path to the properties file
  #[arg(short, long)]
  properties: Option<String>,
  /// File path to the runtime configuration
  #[arg(short, long)]
  config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
  signals: Vec<String>,
}

type MyFstReader = FstReader<std::io::BufReader<std::fs::File>>;

fn main() -> anyhow::Result<()> {
  let global_logger = FmtSubscriber::builder()
    .with_env_filter(EnvFilter::from_default_env())
    .with_max_level(Level::TRACE)
    .without_time()
    .with_target(false)
    .compact()
    .finish();
  tracing::subscriber::set_global_default(global_logger)
    .expect("internal error: fail to setup log subscriber");

  let args = CliArgs::parse();
  info!("Reading FST from file: {}", args.fst);

  let file = std::fs::File::open(args.fst)?;
  let mut reader = FstReader::open(std::io::BufReader::new(file))?;

  let header = reader.get_header();
  trace!(
    version = header.version,
    date = header.date,
    start_time = header.start_time,
    end_time = header.end_time,
    "Header info"
  );

  info!("Reading config from file {}", args.config);
  let config = std::fs::read(args.config)?;
  let config: Config = serde_json::from_slice(&config)?;

  info!("Iterating hierachy to get signal information");
  let metadata = collect_signals(&mut reader, &config.signals)?;

  info!("Fetching signals value");

  let filter = FstFilter::filter_signals(metadata.handle.clone());
  reader.read_signals(&filter, |t, handle, value| {
    let v = match value {
      FstSignalValue::String(s) => s,
      FstSignalValue::Real(r) => format!("real: {}", r),
    };
    let result = metadata
      .handle
      .iter()
      .enumerate()
      .find(|(_, item)| item.get_index() == handle.get_index());
    if let Some((i, _)) = result {
      trace!("time: {} signal: {} value: {}", t, metadata.names[i], v);
    }
  })?;

  Ok(())
}

#[derive(Default, Debug)]
struct SignalMetadata {
  module_paths: Vec<Vec<String>>,
  names: Vec<String>,
  handle: Vec<FstSignalHandle>,
}

impl SignalMetadata {
  fn push(&mut self, module_path: Vec<String>, name: String, handle_id: FstSignalHandle) {
    self.module_paths.push(module_path);
    self.names.push(name);
    self.handle.push(handle_id);
  }
}

fn collect_signals(
  reader: &mut MyFstReader,
  expected: &[String],
) -> anyhow::Result<SignalMetadata> {
  let mut metadata = SignalMetadata::default();
  let mut module_path: Vec<String> = Vec::new();
  let mut dedup_pool = HashSet::new();
  reader.read_hierarchy(|hier| match hier {
    FstHierarchyEntry::Var { name, handle, .. } => {
      if expected.contains(&name) && !dedup_pool.contains(&handle.get_index()) {
        let id = handle.get_index();
        metadata.push(module_path.clone(), name, handle);
        dedup_pool.insert(id);
      }
    }
    FstHierarchyEntry::Scope { name, .. } => module_path.push(name.to_string()),
    FstHierarchyEntry::UpScope => {
      module_path.pop();
    }
    _ => (),
  })?;

  Ok(metadata)
}
