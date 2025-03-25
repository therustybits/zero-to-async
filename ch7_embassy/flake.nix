{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
    };
  
  outputs = { self, nixpkgs, ... }@inputs:
    let
     system = "x86_64-linux"; # your version
     pkgs = nixpkgs.legacyPackages.${system};    
    in
    {
      devShells.${system}.default = pkgs.mkShell
      {
        name = "Rust Flake";
        packages = with pkgs; [ 
        # add this module, to enable cross-compilation.
          #crossSystem = {
            # target platform
            #system = "thumbv6m-none-eabi";
          #};
          rustup
          file
          elfutils
          elf2uf2-rs
          probe-rs-tools
          appimage-run
          segger-jlink
          lld
          cargo-machete
          #segger-ozone 
        ]; # whatever you need

        #services.udev.extraRules = builtins.readFile ./rules-file;
        # LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang.lib}/lib";
        #services.udev.extraRules = builtins.readFile ${pkgs.segger-jlink}/lib/udev/rules.d/99-jlink.rules
        #services.udev.extraRules = builtins.readFile /nix/store/vy5q0qv46sk2dm85wb7zl2hqgzfrvlzh-segger-jlink-810/lib/udev/rules.d/99-jlink.rules;

        shellHook = ''
          echo "Welcome to the $name development shell!"
          rustup component add rustfmt
          rustup component add clippy
          rustup target add thumbv7em-none-eabi
          # For rust-analyzer 'hover' tooltips to work
          export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/library"          export PS1='\n\[\033[1;34m\](Rust):\w]\$\[\033[0m\]'
          echo "All necessary libraries and tools are installed."
        '';
      };
    };
}