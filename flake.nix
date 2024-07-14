{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/24.05";
    naersk.url = "github:nix-community/naersk";

    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, naersk }: let
    forAllSystems = function: nixpkgs.lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"
    ] (system: function (import nixpkgs { inherit system; }));
    cargoToml = with builtins; fromTOML (readFile ./Cargo.toml);
  in {
    packages = forAllSystems (pkgs: let
      naersk' = pkgs.callPackage naersk {};
    in rec {
      inventor_bot = naersk'.buildPackage {
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        src = ./.;

        nativeBuildInputs = with pkgs; [ pkg-config ];

        buildInputs = with pkgs; [
          systemd dbus openssl
        ];

        meta = {
          description = cargoToml.package.description;
          homepage = cargoToml.package.repository;
          mainProgram = cargoToml.package.name;
        };
      };
      default = inventor_bot;
    });

    apps = forAllSystems (pkgs: builtins.mapAttrs
      (_: pkg: {
        type = "app";
        program = pkgs.lib.getExe pkg;
      })
      self.packages.${pkgs.system});

    devShells = forAllSystems (pkgs: {
      default = with pkgs; mkShell {
        buildInputs = [
          cargo rustc rustfmt pre-commit clippy
          pkg-config systemd dbus openssl
        ];
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    });
  };
}
