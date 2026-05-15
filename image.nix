{ pkgs, craneLib }:

let
  backend = craneLib.buildPackage {
    src = craneLib.cleanCargoSource ./.;
    nativeBuildInputs = with pkgs; [ pkg-config ];
    buildInputs = with pkgs; [ openssl clang ];
  };
in
pkgs.dockerTools.buildImage {
  name = "stargem-backend";
  tag = "latest";
  copyToRoot = pkgs.buildEnv {
    name = "image-root";
    paths = [
      backend
      pkgs.cacert
      (pkgs.runCommand "config" {} ''
        mkdir -p $out/app/config
        cp ${./config/damage_multipliers.toml} $out/app/config/damage_multipliers.toml
      '')
    ];
  };
  config = {
    Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
    ExposedPorts = { "50051/tcp" = {}; "50052/udp" = {}; };
    Entrypoint = [ "${backend}/bin/stargem-backend" ];
    WorkingDir = "/app";
  };
}
