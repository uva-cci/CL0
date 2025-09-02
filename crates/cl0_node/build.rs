fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Rebuild if these files (or the proto dir) change
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rerun-if-changed=proto/common/types.proto");
    println!("cargo:rerun-if-changed=proto/node/node.proto");
    println!("cargo:rerun-if-changed=proto/control_plane/control_plane.proto");

    std::fs::create_dir_all("src/generated")?;

    tonic_prost_build::configure()
        .out_dir("src/generated")
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/common/types.proto",
                "proto/common/rules.proto",
                "proto/node/node.proto",
                "proto/control_plane/control_plane.proto",
                "proto/web/web.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
