{
  description = "SaaS Shield Demo Notes App Infrastructure";
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
        docker-compose
        (google-cloud-sdk.withExtraComponents [google-cloud-sdk.components.gke-gcloud-auth-plugin])
        docker-credential-gcr
        docker-credential-helpers
        ollama
        docker
      ];
    in {
      devShell = pkgs.mkShell {
        shellHook = ''
          export OLLAMA_HOST=127.0.0.1:11434
        '';
        buildInputs = dependencies;
      };
    });
}
