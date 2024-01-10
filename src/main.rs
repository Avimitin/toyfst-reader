use std::collections::HashSet;

use clap::Parser;
use fst_native::*;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn hierarchy_tpe_to_str(tpe: &FstScopeType) -> &'static str {
  match tpe {
    FstScopeType::Module => "Module",
    FstScopeType::Task => "Task",
    FstScopeType::Function => "Function",
    FstScopeType::Begin => "Begin",
    FstScopeType::Fork => "Fork",
    FstScopeType::Generate => "Generate",
    FstScopeType::Struct => "Struct",
    FstScopeType::Union => "Union",
    FstScopeType::Class => "Class",
    FstScopeType::Interface => "Interface",
    FstScopeType::Package => "Package",
    FstScopeType::Program => "Program",
    FstScopeType::VhdlArchitecture => "VhdlArchitecture",
    FstScopeType::VhdlProcedure => "VhdlProcedure",
    FstScopeType::VhdlFunction => "VhdlFunction",
    FstScopeType::VhdlRecord => "VhdlRecord",
    FstScopeType::VhdlProcess => "VhdlProcess",
    FstScopeType::VhdlBlock => "VhdlBlock",
    FstScopeType::VhdlForGenerate => "VhdlForGenerate",
    FstScopeType::VhdlIfGenerate => "VhdlIfGenerate",
    FstScopeType::VhdlGenerate => "VhdlGenerate",
    FstScopeType::VhdlPackage => "VhdlPackage",
    FstScopeType::AttributeBegin => "AttributeBegin",
    FstScopeType::AttributeEnd => "AttributeEnd",
    FstScopeType::VcdScope => "VcdScope",
    FstScopeType::VcdUpScope => "VcdUpScope",
  }
}

pub fn hierarchy_to_str(entry: &FstHierarchyEntry) -> String {
  match entry {
    FstHierarchyEntry::Scope {
      name,
      tpe,
      component,
    } => format!("Scope: {name} ({}) {component}", hierarchy_tpe_to_str(tpe)),
    FstHierarchyEntry::UpScope => "UpScope".to_string(),
    FstHierarchyEntry::Var { name, handle, .. } => format!("({}): {name}", handle.get_index()),
    FstHierarchyEntry::AttributeEnd => "EndAttr".to_string(),
    FstHierarchyEntry::PathName { name, id } => format!("PathName: {id} -> {name}"),
    FstHierarchyEntry::SourceStem {
      is_instantiation,
      path_id,
      line,
    } => format!("SourceStem:: {is_instantiation}, {path_id}, {line}"),
    FstHierarchyEntry::Comment { string } => format!("Comment: {string}"),
    FstHierarchyEntry::EnumTable {
      name,
      handle,
      mapping,
    } => {
      let names = mapping
        .iter()
        .map(|(_v, n)| n.clone())
        .collect::<Vec<_>>()
        .join(" ");
      let values = mapping
        .iter()
        .map(|(v, _n)| v.clone())
        .collect::<Vec<_>>()
        .join(" ");
      format!(
        "EnumTable: {name} {} {names} {values} ({handle})",
        mapping.len()
      )
    }
    FstHierarchyEntry::EnumTableRef { handle } => format!("EnumTableRef: {handle}"),
  }
}

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
  /// Signals that should be capture and outputs
  #[arg(short, long)]
  signals: String,
}

type MyFstReader = FstReader<std::io::BufReader<std::fs::File>>;

fn main() -> anyhow::Result<()> {
  let global_logger = FmtSubscriber::builder()
    .with_env_filter(EnvFilter::from_default_env())
    .with_max_level(Level::TRACE)
    .compact()
    .finish();
  tracing::subscriber::set_global_default(global_logger)
    .expect("internal error: fail to setup log subscriber");

  let args = CliArgs::parse();
  info!("Reading FST from file: {}", args.fst);

  let file = std::fs::File::open(args.fst)?;
  let mut reader = match FstReader::open(std::io::BufReader::new(file)) {
    Ok(r) => r,
    Err(e) => anyhow::bail!("{e:?}"),
  };

  let header = reader.get_header();
  info!(
    version = header.version,
    date = header.date,
    start_time = header.start_time,
    end_time = header.end_time,
    "header info"
  );

  let expected = args.signals.split(',').collect::<Vec<_>>();
  info!("Iterating hierachy to get signal information");

  let metadata = collect_signals(&mut reader, &expected)?;

  info!("Fetching signals value");

  let f = FstFilter::filter_signals(metadata.handle.clone());
  reader
    .read_signals(&f, |t, handle, value| {
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
        info!("time: {} signal: {} value: {}", t, metadata.names[i], v);
      }
    })
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

fn collect_signals(reader: &mut MyFstReader, expected: &[&str]) -> anyhow::Result<SignalMetadata> {
  let mut metadata = SignalMetadata::default();
  let mut module_path: Vec<String> = Vec::new();
  let mut dedup_pool = HashSet::new();
  let read_result = reader.read_hierarchy(|hier| match hier {
    FstHierarchyEntry::Var { name, handle, .. } => {
      if expected.contains(&name.as_str()) && !dedup_pool.contains(&handle.get_index()) {
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
  });
  if let Err(err) = read_result {
    anyhow::bail!("{:?}", err)
  }
  Ok(metadata)
}
