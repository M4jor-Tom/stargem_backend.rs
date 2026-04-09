{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    opencode.url = "github:anomalyco/opencode";
  };

  outputs = { self, nixpkgs, opencode, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      # Run with: nix run .#opencode
      packages = forAllSystems (system: {
        opencode = opencode.packages.${system}.default;
      });
      
      # Run with: nix run .#opencode-with-ollama
      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              opencode.packages.${system}.default
            ];
            
            shellHook = ''
              echo "OpenCode is ready. Run 'opencode' to start."
              echo "Ollama is available for local LLM support."
            '';
          };
        }
      );
    };
}
