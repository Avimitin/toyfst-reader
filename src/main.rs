use clap::Parser;
use fst_native::*;

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
  properties: String,
}

fn main() {
  let args = CliArgs::parse();

  println!("Reading FST from file: {}", args.fst);
  let file = std::fs::File::open(args.fst).unwrap();
  let mut reader = FstReader::open(std::io::BufReader::new(file)).unwrap();
  let header = reader.get_header();
  println!(
    "fst file start time: {}, fst file end time: {}",
    header.start_time, header.end_time
  );

  let h = FstSignalHandle::from_index(63);
  let f = FstFilter::filter_signals(vec![h]);
  reader
    .read_hierarchy(|hier: FstHierarchyEntry| println!("{}", hierarchy_to_str(&hier)))
    .unwrap();
  reader
    .read_signals(&f, |t, handle, value| {
      let v = match value {
        FstSignalValue::String(s) => s,
        FstSignalValue::Real(r) => format!("real: {}", r),
      };
      println!("time: {} handle: {} value: {}", t, handle, v);
    })
    .unwrap();
}
