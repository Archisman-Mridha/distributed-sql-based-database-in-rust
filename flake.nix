{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        projectRootPath = ./.;
        rustToolchainFilePath = projectRootPath + /rust-toolchain.toml;
      in
      with pkgs;
      {
        nativeBuildInputs = [
          (rust-bin.fromRustupToolchainFile rustToolchainFilePath)
        ];

        devShells.default = mkShell {
          buildInputs = [
            llvm
            rustup
            (rust-bin.fromRustupToolchainFile rustToolchainFilePath)
            wasm-tools
          ];
        };
      }
    );
}
