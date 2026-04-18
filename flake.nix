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
        integration-test = pkgs.writeScriptBin "integration-test" ''
          #!${pkgs.bash}/bin/bash
          set -e

          COMPOSE_FILE="compose/docker-compose.test.yml"
          CONTAINER_NAME="stargem-postgres-test"

          cleanup() {
            echo "Stopping test database..."
            podman-compose -f "$COMPOSE_FILE" down -v 2>/dev/null || true
          }
          trap cleanup EXIT

          echo "Starting test database..."
          podman-compose -f "$COMPOSE_FILE" up -d

          echo "Waiting for database to be ready..."
          for i in {1..30}; do
            if podman exec "$CONTAINER_NAME" pg_isready -U stargem_test -d stargem_test > /dev/null 2>&1; then
              echo "Database is ready!"
              break
            fi
            if [ $i -eq 30 ]; then
              echo "Database failed to start within 30 seconds"
              exit 1
            fi
            sleep 1
          done

          echo "Running integration tests..."
          cargo test
        '';
      in
      {
        packages = {
          inherit integration-test;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            integration-test
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            sqlx-cli
            openssl
            pkg-config
            podman
            podman-compose
          ];
          
          RUST_LOG = "stargem_server=debug,info";
          
          shellHook = ''
            echo "Stargem Server dev shell ready."
            echo "Rust $(rustc --version) installed."
            echo ""
            echo "Commands:"
            echo "  cargo build        - Build the project"
            echo "  cargo test        - Run tests"
            echo "  cargo run         - Run the server"
            echo "  cargo clippy      - Lint the code"
            echo "  cargo fmt         - Format code"
            echo ""
            echo "Integration tests:"
            echo "  nix run .#integration-test                   # Run all integration tests"
            echo "  nix run .#integration-test -- --nocapture    # With output"
            echo "  nix run .#integration-test combat            # Run combat tests only"
          '';
        };
      }
    );
}
