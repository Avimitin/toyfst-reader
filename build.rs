use std::io::Result;

fn main() -> Result<()> {
  let mut compile_config = prost_build::Config::default();
  compile_config.type_attribute(".", "#[derive(typed_builder::TypedBuilder)]");
  compile_config.compile_protos(&["src/profile.proto"], &["src/"])?;
  Ok(())
}
