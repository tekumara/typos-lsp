{
  description = "Source code spell checker for Visual Studio Code and LSP clients";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat.url = "github:edolstra/flake-compat/master";
    flake-utils.url = "github:numtide/flake-utils/main";
    nixpkgs.url = "github:NixOS/nixpkgs/master";
  };

  outputs =
    {
      self,
      fenix,
      flake-utils,
      nixpkgs,
      ...
    }:
    let
      inherit (builtins) fromTOML readFile;
      inherit ((fromTOML (readFile ./crates/typos-lsp/Cargo.toml)).package) name version;
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-analyzer"
          "rust-src"
          "rustc"
          "rustfmt"
        ];
      in
      {
        apps =
          let
            default = flake-utils.lib.mkApp { drv = self.packages.${system}.default; };
          in
          {
            inherit default;
            ${name} = default;
          };

        devShells.default = pkgs.mkShell {
          inherit name;

          nativeBuildInputs = with pkgs; [
            glib
            pkg-config
            rustToolchain
          ];
        };

        formatter = pkgs.nixfmt-rfc-style;

        packages =
          let
            default =
              (pkgs.makeRustPlatform {
                cargo = rustToolchain;
                rustc = rustToolchain;
              }).buildRustPackage
                {
                  inherit version;
                  pname = name;
                  src = ./.;
                  cargoLock.lockFile = ./Cargo.lock;
                };
          in
          {
            inherit default;
            ${name} = default;
          };
      }
    );
}
