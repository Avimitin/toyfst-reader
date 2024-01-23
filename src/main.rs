use std::collections::HashSet;
use std::io::Write;

use clap::Parser;
use flate2::write::GzEncoder;
use flate2::Compression;
use fst_native::*;
use prost::Message;
use serde::Deserialize;
use tracing::{info, trace, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod pprof;

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
  #[arg(short, long)]
  output: Option<String>,
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

  let file = std::fs::File::open(&args.fst)?;
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

  let mut str_tbl = pprof::StringTable::new();

  let mut p = pprof::Profile::default();
  p.time_nanos = 10000;
  p.period_type = Some(pprof::ValueType {
    r#type: str_tbl.id("cycle"),
    unit: str_tbl.id("number"),
  });
  p.period = 1;
  p.duration_nanos = (header.end_time - header.start_time).try_into().unwrap();

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
      trace!(
        "time: {} module: {} signal: {} value: {}",
        t,
        metadata.module_paths[i].join("."),
        metadata.names[i],
        v
      );
    }
  })?;

  p.string_table = str_tbl.to_string_table();

  let mut buf = Vec::new();
  buf.reserve(p.encoded_len());
  p.encode(&mut buf).unwrap();

  let mut encoder = GzEncoder::new(Vec::with_capacity(p.encoded_len()), Compression::default());
  encoder.write_all(&buf).unwrap();

  std::fs::write(
    // if output path is not given, pprof proto file will be default writed into current path
    // with same name as the .fst file
    args.output.unwrap_or_else(|| {
      let input_file_path = std::path::Path::new(&args.fst);
      let filename = input_file_path.file_stem().unwrap().to_str().unwrap();
      format!("{filename}.pprof.gz")
    }),
    encoder.finish().unwrap(),
  )
  .unwrap();
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
