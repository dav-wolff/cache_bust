# cache_bust_cli

This is a CLI-tool to be used in combination with **[cache_bust]**
to add hashes to file names.

[cache_bust]: https://crates.io/crates/cache_bust

## Installation

Using cargo:
```sh
cargo install cache_bust_cli --locked
```

Using nix:
```
nix shell github:dav-wolff/cache_bust#cli
```

## Usage

Rename all files in a directory in-place:
```sh
cachebust assets
```

Copy all files in a directory to a new directory with hashes added:
```sh
cachebust assets --out hashed_assets
```

Rename a single file in-place and print its new name:
```sh
cachebust assets --file image.png --print-file-name # image-d0a2[...].png
```

Copy a single file to a new directory with its hash added and print its new path:
```sh
cachebust assets --file image.png --print-file-path # /path/to/image-d0a2[...].png
```
