{
	description = "A library for compile-time \"cache busting\", including hashes in file names in order to optimize for caching.";
	
	inputs = {
		nixpkgs = {
			url = "github:nixos/nixpkgs/nixpkgs-unstable";
		};
		
		flake-utils = {
			url = "github:numtide/flake-utils";
		};
		
		crane = {
			url = "github:ipetkov/crane";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};
	
	outputs = {self, nixpkgs, flake-utils, crane, fenix}:
		flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};
				
				fenixPackage = fenix.packages.${system};
				fenixToolchain = fenixPackage.stable.defaultToolchain;
				craneLib = (crane.mkLib pkgs).overrideToolchain fenixToolchain;
				
				src = with pkgs.lib; cleanSourceWith {
					src = craneLib.path ./.;
					filter = path: type:
						(hasInfix "/tests/" path) ||
						(hasInfix "/assets/" path) ||
						(craneLib.filterCargoSources path type)
					;
				};
				
				commonArgs = {
					pname = "cache_bust";
					version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
					
					inherit src;
					strictDeps = true;
					
					buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
						pkgs.libiconv
					];
				};
				
				cargoArtifacts = craneLib.buildDepsOnly commonArgs;
				
				cli = craneLib.buildPackage (commonArgs // {
					pname = "cachebust";
					cargoExtraArgs = "-p cache_bust_cli";
				});
			in {
				packages = {
					inherit cli;
				};
				
				checks = {
					test = craneLib.cargoTest (commonArgs // {
						inherit cargoArtifacts;
					});
					
					clippy = craneLib.cargoClippy (commonArgs // {
						inherit cargoArtifacts;
						cargoClippyExtraArgs = "--all-targets -- --deny warnings";
					});
				};
				
				devShells.default = craneLib.devShell {
					packages = with pkgs; [
						rust-analyzer
					];
				};
			}
		);
}
