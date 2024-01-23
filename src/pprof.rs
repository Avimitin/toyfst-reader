use std::collections::HashMap;

// ProtoBuf struct defined in target/<debug/release>/build/<name>/out
include!(concat!(env!("OUT_DIR"), "/pprof.profiles.rs"));

/// The protobuf definition given by pprof specified that every string usage needs to be store in
/// an array, and all the string fields should be an index to the field in the String Table.
///
/// This struct wraps a HashMap for quick string searching and insertion.
#[derive(Debug, Default)]
pub struct StringTable {
  data: HashMap<String, i64>,
  next: i64,
}

impl StringTable {
  /// Return a default empty StringTable.
  pub fn new() -> Self {
    // According to pprof spec, the first item of the string table should left blank.
    Self {
      data: HashMap::from([(String::new(), 0)]),
      next: 0,
    }
  }

  /// Return the id of the given string. If the string doesn't exists in StringTable, then it will
  /// be allocated and assigned a new ID.
  pub fn id(&mut self, q: &str) -> i64 {
    self
      .data
      .entry(q.to_string())
      .or_insert_with(|| {
        self.next += 1;
        self.next
      })
      .to_owned()
  }

  /// Convert the StringTable struct to a list of strings.
  pub fn to_string_table(&self) -> Vec<String> {
    let mut cache = self.data.iter().collect::<Vec<_>>();
    // sort by index
    cache.sort_by(|(_, prev), (_, next)| prev.cmp(next));
    // waive index
    cache.into_iter().map(|(str, _)| str.to_owned()).collect()
  }
}

#[test]
fn sample_profile() {
  use flate2::write::GzEncoder;
  use flate2::Compression;
  use std::io::Write;

  use prost::Message;

  let mut str_tbl = StringTable::new();

  let value_types = vec![ValueType {
    r#type: str_tbl.id("cycle"),
    unit: str_tbl.id("cycle"),
  }];

  let instructions = vec![
    Function {
      id: 1000,
      name: str_tbl.id("vadd.vv"),
      filename: 0,
      system_name: 0,
      start_line: 0,
    },
    Function {
      id: 1001,
      name: str_tbl.id("vdiv.vv"),
      filename: 0,
      system_name: 0,
      start_line: 0,
    },
  ];

  let lines = vec![
    Line {
      function_id: 1000,
      line: 0,
    },
    Line {
      function_id: 1001,
      line: 0,
    },
  ];

  let loc = vec![
    Location {
      id: 1,
      mapping_id: 0,
      address: 0,
      line: vec![lines[0].clone()],
      is_folded: false,
    },
    Location {
      id: 2,
      mapping_id: 0,
      address: 0,
      line: vec![lines[1].clone()],
      is_folded: false,
    },
  ];

  let samples = vec![
    Sample {
      location_id: vec![1],
      value: vec![1],
      label: vec![
        Label {
          key: str_tbl.id("signalAReady"),
          str: str_tbl.id("0"),
          num: 0,
          num_unit: 0,
        },
        Label {
          key: str_tbl.id("signalAValid"),
          str: str_tbl.id("0"),
          num: 0,
          num_unit: 0,
        },
        Label {
          key: str_tbl.id("signalBQueueData [2:0]"),
          str: str_tbl.id("010"),
          num: 0,
          num_unit: 0,
        },
      ],
    },
    Sample {
      location_id: vec![1, 2],
      value: vec![5],
      label: vec![
        Label {
          key: str_tbl.id("signalAReady"),
          str: str_tbl.id("0"),
          num: 0,
          num_unit: 0,
        },
        Label {
          key: str_tbl.id("signalAValid"),
          str: str_tbl.id("1"),
          num: 0,
          num_unit: 0,
        },
        Label {
          key: str_tbl.id("signalBQueueData [2:0]"),
          str: str_tbl.id("110"),
          num: 0,
          num_unit: 0,
        },
      ],
    },
    Sample {
      location_id: vec![2],
      value: vec![6],
      label: vec![
        Label {
          key: str_tbl.id("signalAReady"),
          str: str_tbl.id("1"),
          num: 0,
          num_unit: str_tbl.id("bit"),
        },
        Label {
          key: str_tbl.id("signalAValid"),
          str: str_tbl.id("0"),
          num: 0,
          num_unit: 0,
        },
        Label {
          key: str_tbl.id("signalBQueueData [2:0]"),
          str: str_tbl.id("010"),
          num: 0,
          num_unit: 0,
        },
      ],
    },
  ];

  let mut p = Profile::default();
  p.time_nanos = 10000;
  p.sample_type = value_types;
  p.period = 1;
  p.duration_nanos = 1000_000_000;
  p.sample = samples;
  p.location = loc;
  p.function = instructions;

  p.string_table = str_tbl.to_string_table();

  let mut buf = Vec::new();
  buf.reserve(p.encoded_len());
  p.encode(&mut buf).unwrap();

  let mut encoder = GzEncoder::new(Vec::with_capacity(p.encoded_len()), Compression::default());
  encoder.write_all(&buf).unwrap();
  std::fs::write("./sample_profile.pb.gz", encoder.finish().unwrap()).unwrap();
}
