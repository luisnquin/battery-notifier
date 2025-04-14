{pkgs ? import <nixpkgs> {}, ...}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "battery-notifier";
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

  src = builtins.path {
    name = "${pname}-source";
    path = ./.;
  };

  nativeBuildInputs = with pkgs; [git cmake makeWrapper];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  preConfigure = ''
    substituteInPlace ./src/notify.rs \
    --replace './assets/battery-danger.png' '${placeholder "out"}/assets/battery-danger.png'
  '';

  preInstall = ''
    mkdir -p $out/assets/
    cp ./assets/*.png $out/assets/
  '';
}
