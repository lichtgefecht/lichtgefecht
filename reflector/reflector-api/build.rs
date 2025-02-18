use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["../../api/lg.proto"], &["src/", "../../api/"])?;
    Ok(())
}
