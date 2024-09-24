{
  description = "SaaS Shield Demo Notes App";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      lib = import <nixpkgs/lib>;
      pkgs = nixpkgs.legacyPackages.${system};
      dependencies = with pkgs; [
        nodejs_20
        autoconf
        mozjpeg
        libtool
        automake
        nasm
        libpng
        optipng
        pkg-config
        gcc
        dpkg
        docker-compose
        # (pkgs.yarn.override {nodejs = pkgs.nodejs-20_x;})
        nodePackages.typescript
        nodePackages.typescript-language-server
        nodePackages.diagnostic-languageserver
        nodePackages.eslint_d
        (google-cloud-sdk.withExtraComponents [google-cloud-sdk.components.gke-gcloud-auth-plugin])
        docker-credential-gcr
        docker-credential-helpers
      ];
    in {
      devShell = pkgs.mkShell {
        shellHook = ''
        '';
        buildInputs = dependencies;
      };
    });
}
