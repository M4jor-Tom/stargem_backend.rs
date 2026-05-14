{
  description = "Stargem backend — Rust toolchain, crane, protoc, SQLx CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    stargem-protos = {
      url = "github:M4jor-Tom/stargem_protos";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane, stargem-protos }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;

        commonArgs = {
          PROTO_SRC = "${stargem-protos}";
          nativeBuildInputs = with pkgs; [ protobuf pkg-config ];
          buildInputs = with pkgs; [ openssl clang ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = craneLib.cleanCargoSource ./.;
        });

        backend = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          src = craneLib.cleanCargoSource ./.;
        });
      in
      {
        devShells.default = pkgs.mkShell {
          name = "stargem-backend";
          buildInputs = with pkgs; [
            rustToolchain protobuf openssl pkg-config sqlx-cli clang
          ];
          PROTO_SRC = "${stargem-protos}";
          shellHook = ''
            export PROTOC="${pkgs.protobuf}/bin/protoc"
            export PROTOC_INCLUDE="${pkgs.protobuf}/include"
            export PROTO_SRC="${stargem-protos}"
          '';
        };

        packages.default = backend;
        packages.dockerImage = pkgs.callPackage ./image.nix {
          inherit craneLib stargem-protos;
        };
      });
}
