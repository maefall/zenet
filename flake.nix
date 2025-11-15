{
	inputs = {
    	nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    	flake-parts.url = "github:hercules-ci/flake-parts";
    	rust-overlay.url = "github:oxalica/rust-overlay";
  	};

  	outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
		systems = [ "x86_64-linux" ];
      	perSystem = { config, self', pkgs, lib, system, ... }:
        let
			rustVersion = "1.91.0";
          	cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
			rustToolchain = pkgs.rust-bin.stable.${rustVersion}.complete;

        	buildInputs = with pkgs; [ git ];

			mkDevShellRust = rustc:
  			pkgs.mkShell {
				inherit buildInputs;
    			nativeBuildInputs = with pkgs; [
					pkg-config
					rustc
					sccache
				];

				RUSTC_WRAPPER = lib.getExe pkgs.sccache;
    			RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
    			LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
  			};
    	in {
          	_module.args.pkgs = import inputs.nixpkgs {
            	inherit system;

            	overlays = [ (import inputs.rust-overlay) ];
          	};

			devShells.default = mkDevShellRust rustToolchain;
        };
    };
}
