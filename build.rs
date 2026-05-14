fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = std::path::PathBuf::from(
        std::env::var("PROTO_SRC").expect("PROTO_SRC must be set"),
    );

    let grpc_protos = &[
        proto_root.join("grpc/auth.proto"),
        proto_root.join("grpc/shop.proto"),
        proto_root.join("grpc/hangar.proto"),
        proto_root.join("grpc/loadout.proto"),
        proto_root.join("grpc/matchmaking.proto"),
        proto_root.join("grpc/match_history.proto"),
    ];

    let quic_protos = &[
        proto_root.join("quic/common.proto"),
        proto_root.join("quic/combat.proto"),
    ];

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(grpc_protos, &[proto_root.clone()])?;

    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .compile(quic_protos, &[proto_root])?;

    Ok(())
}
