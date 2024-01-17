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
    cache.sort_by(|(_, prev), (_, next)| prev.cmp(next));
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

  let mut p = Profile::default();
  p.time_nanos = 10000;
  p.period_type = Some(ValueType {
    r#type: str_tbl.id("cycle"),
    unit: str_tbl.id("number"),
  });
  p.sample_type = vec![ValueType {
    r#type: str_tbl.id("signal"),
    unit: str_tbl.id("value"),
  }];
  p.period = 1;
  p.duration_nanos = 1000_000_000;

  let mut s = Sample::default();
  s.value = vec![11451];

  let mut label = Label::default();
  label.key = 3;
  label.str = 4;

  p.string_table = str_tbl.to_string_table();
  dbg!(&p);

  let mut buf = Vec::new();
  buf.reserve(p.encoded_len());
  p.encode(&mut buf).unwrap();

  let mut encoder = GzEncoder::new(Vec::with_capacity(p.encoded_len()), Compression::default());
  encoder.write_all(&buf).unwrap();
  std::fs::write("./sample_profile.pb.gz", encoder.finish().unwrap()).unwrap();
}
