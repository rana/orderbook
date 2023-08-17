use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from("src");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(path.clone())
        .compile(&[path.join("lib.proto")], &[path])?;

    Ok(())
}
