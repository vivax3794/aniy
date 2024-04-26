{
  description = "Animation library in rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
            buildInputs = [
                ( rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
                    extensions = ["rust-src" "llvm-tools-preview" "rust-analyzer"];
                }) )
                just
                cargo-nextest

                ffmpeg
                pkg-config

            ];

            LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib";
            LD_LIBRARY_PATH = "${ffmpeg}/lib";
        };
    }
    );
}

