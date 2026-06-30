fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = match std::env::var("PROTO_SRC") {
        Ok(val) => std::path::PathBuf::from(val),
        Err(_) => return Ok(()),
    };

    let quic_protos = &[
        proto_root.join("quic/common.proto"),
        proto_root.join("quic/combat.proto"),
    ];

    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .out_dir("src/proto_gen/quic")
        .compile(quic_protos, &[proto_root.clone()])?;

    let grpc_protos = &[
        proto_root.join("grpc/auth.proto"),
        proto_root.join("grpc/shop.proto"),
        proto_root.join("grpc/hangar.proto"),
        proto_root.join("grpc/loadout.proto"),
        proto_root.join("grpc/matchmaking.proto"),
        proto_root.join("grpc/match_history.proto"),
        proto_root.join("grpc/spectator.proto"),
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto_gen/grpc")
        .extern_path(".stargem.quic", "crate::proto_gen::quic")
        .compile(grpc_protos, &[proto_root])?;

    Ok(())
}
