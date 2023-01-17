{
  description =
    "qrscan - Scan a QR code in the terminal using the system camera or a given image";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    nix.url = "github:domenkozar/nix/relaxed-flakes";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, nix, ... }:
    let
      systems = [
        "x86_64-linux"
        "i686-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        builtins.listToAttrs (map
          (name: {
            inherit name;
            value = f name;
          })
          systems);
    in
    {
      packages = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in
        {
          qrscan = pkgs.rustPlatform.buildRustPackage rec {
            name = "qrscan";
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
          };
        });
      defaultPackage = forAllSystems (system: self.packages.${system}.qrscan);
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          devRequirements = with pkgs; [
            gcc
            gnumake
            clippy
            cargo
            rustc
            rustfmt
            rust-analyzer
          ];
        in
        {
          default = pkgs.mkShell {
            RUST_BACKTRACE = 1;
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

            buildInputs = devRequirements;
            nativeBuildInputs = [ pkgs.rustPlatform.bindgenHook ];
          };
        });
    };
}
