{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    inherit (nixpkgs) lib;
    eachSystem = lib.genAttrs (import systems);
    pkgsFor = eachSystem (system:
      import nixpkgs {
        localSystem = system;
      });
  in {
    packages = eachSystem (system: let
      pkgs = pkgsFor.${system};
    in {
      default = self.packages.${system}.battery-notifier;
      battery-notifier = pkgs.callPackage ./default.nix {};
    });

    nixosModules.default = import ./nix/nixos-module.nix self;
    homeManagerModule.default = import ./nix/hm-module.nix self;

    formatter = eachSystem (system: let
      pkgs = pkgsFor.${system};
    in
      pkgs.alejandra);

    devShells.default = eachSystem (system: let
      pkgs = pkgsFor.${system};
    in
      pkgs.mkShell (let
        rust-latest = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in {
        buildInputs = with pkgs; [rust-analyzer cargo rust-latest];
      }));
  };
}
