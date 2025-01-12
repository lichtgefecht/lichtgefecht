use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["../../api/what.proto"], &["src/", "../../api/"])?;
    Ok(())
}
