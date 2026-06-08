{
  description = "nix flake for egui-greeter";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {

        devShells.default =
          with pkgs;
          mkShell rec {
            buildInputs = [
              pkg-config
              rustc
              rust-analyzer
              rustfmt
              cargo

              openssl

              libGL
              libxkbcommon
              libxcb
              libxkbcommon
              vulkan-loader
              wayland
              libpulseaudio
            ];

            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
            '';
          };

      }
    );
}
