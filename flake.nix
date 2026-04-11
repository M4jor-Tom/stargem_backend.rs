{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    opencode.url = "github:anomalyco/opencode";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, opencode, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            sqlx-cli
            openssl
            pkg-config
          ];
          
          RUST_LOG = "stargem_server=debug,info";
          
          shellHook = ''
            echo "Stargem Server dev shell ready."
            echo "Rust $(rustc --version) installed."
            echo ""
            echo "Commands:"
            echo "  cargo build     - Build the project"
            echo "  cargo test      - Run tests"
            echo "  cargo run       - Run the server"
            echo "  cargo clippy    - Lint the code"
            echo "  cargo fmt       - Format code"
            echo ""
          '';
        };
      }
    );
}
