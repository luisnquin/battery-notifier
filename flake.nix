{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
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

    homeManagerModule.default = import ./nix/hm-module.nix self;
  };
}
