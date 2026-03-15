{
  description = "PageLens development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      systems,
      rust-overlay,
      ...
    }:
    let
      eachSystem =
        f:
        nixpkgs.lib.genAttrs (import systems) (
          system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            };
          in
          f pkgs
        );

      packages =
        pkgs: with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rustfmt"
            ];
          })

          bun
        ];
    in
    {
      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          buildInputs = packages pkgs;

          shellHook = ''
            if [ -z "${"CI:-"}" ]; then
              echo "PageLens development environment"
              echo "Bun   - $(${pkgs.bun}/bin/bun --version)"
              echo "Rustc - $(rustc --version)"
            fi
          '';

          RUST_BACKTRACE = "full";
        };
      });
    };
}
