{
  description = "Stargem backend — Rust toolchain, crane, protoc, SQLx CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" "rustfmt" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          nativeBuildInputs = with pkgs; [ pkg-config ];
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
          shellHook = ''
            export PROTOC="${pkgs.protobuf}/bin/protoc"
            export PROTOC_INCLUDE="${pkgs.protobuf}/include"
          '';
        };

        packages.default = backend;
        packages.dockerImage = pkgs.callPackage ./image.nix {
          inherit craneLib;
        };
      });
}
