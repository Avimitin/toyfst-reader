include!(concat!(env!("OUT_DIR"), "/perftools.profiles.rs"));

#[test]
fn sample_profile() {
  use flate2::write::GzEncoder;
  use flate2::Compression;
  use std::io::Write;

  use prost::Message;

  let mut p = Profile::default();
  p.time_nanos = 10000;
  p.string_table = ["", "cpu", "cycle", "property", "property A"]
    .map(|s| s.to_string())
    .to_vec();
  p.period_type = Some(ValueType { r#type: 1, unit: 2 });
  p.sample_type = vec![ValueType { r#type: 1, unit: 2 }];
  p.period = 1;
  p.duration_nanos = 1000_000_000;

  let mut s = Sample::default();
  s.value = vec![11451];

  let mut label = Label::default();
  label.key = 3;
  label.str = 4;

  let mut buf = Vec::new();
  buf.reserve(p.encoded_len());
  p.encode(&mut buf).unwrap();

  let mut encoder = GzEncoder::new(Vec::with_capacity(p.encoded_len()), Compression::default());
  encoder.write_all(&buf).unwrap();
  std::fs::write("./sample_profile.pb.gz", encoder.finish().unwrap()).unwrap();
}
