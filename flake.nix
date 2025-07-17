{
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };
    outputs = { nixpkgs, rust-overlay, ... }:
    let
        forAllSystems = f: nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed f;
    in {
        devShells = forAllSystems (system: {
            default = nixpkgs.legacyPackages.${system}.mkShell (
            let
                pkgs = import nixpkgs {
                    localSystem = system;
                    overlays = [ rust-overlay.overlays.rust-overlay ];
                };
                cc = pkgs.lib.getExe pkgs.pkgsStatic.stdenv.cc;
            in {
                packages = with pkgs; [
                    (rust-bin.stable.latest.default.override {
                        targets = [ "x86_64-unknown-linux-musl" ];
                    })
                    pkg-config
                ];
                env = {
                    CC_x86_64_unknown_linux_musl = cc;
                    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = cc;
                    PKG_CONFIG_PATH = "${pkgs.systemd.dev}/lib/pkgconfig";
                };
            });
        });
    };
}
