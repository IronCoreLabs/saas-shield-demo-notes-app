{
  description = "SaaS Shield Demo Notes App";
  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        lib = import <nixpkgs/lib>;
        pkgs = import nixpkgs { inherit system overlays; };
        rusttoolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust/rust-toolchain.toml;
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
          (google-cloud-sdk.withExtraComponents [ google-cloud-sdk.components.gke-gcloud-auth-plugin ])
          docker-credential-gcr
          docker-credential-helpers
        ];
      in
      rec {
        devShell = pkgs.mkShell {
          buildInputs =
            with pkgs;
            dependencies
            ++ [
              rusttoolchain
              pkg-config
              pkgs.libiconv
              pkgs.prometheus
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
        };

      }
    );
}
