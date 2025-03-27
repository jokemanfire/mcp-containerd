fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.include_file("_includes.rs");
    config.enable_type_names();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_with_config(config, &["proto/api.proto"], &["proto"])?;
    Ok(())
}
