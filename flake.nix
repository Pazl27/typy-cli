{
  description = "typy-cli - Minimalistic Monkeytype clone for the CLI";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        buildInputs = with pkgs; [ ];

        nativeBuildInputs = with pkgs; [
          rust-bin.stable.latest.default
          pkg-config
          makeWrapper
        ];

      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "typy-cli";
          version = "0.7.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          inherit buildInputs nativeBuildInputs;
          postInstall = ''
            mkdir -p $out/share/typy
            cp $src/resources/english.txt $out/share/typy/english.txt

            wrapProgram $out/bin/typy \
            --run "mkdir -p ~/.local/share/typy" \
            --run "cp -n $out/share/typy/english.txt ~/.local/share/typy/english.txt"
            '';

          meta = with pkgs.lib; {
            description = "typy-cli - Minimalistic Monkeytype clone for the CLI";
            homepage = "https://github.com/Pazl27/typy-cli";
            license = licenses.mit;
          };
        };
      }
  );
}

