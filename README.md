sha1dir
=======

[![Build Status](https://img.shields.io/github/workflow/status/dtolnay/sha1dir/CI/master)](https://github.com/dtolnay/sha1dir/actions?query=branch%3Amaster)
[![Latest Version](https://img.shields.io/crates/v/sha1dir.svg)](https://crates.io/crates/sha1dir)

Compute a checksum of a directory tree, for example to validate that a directory
was copied successfully to a different machine.

## Installation

```console
$ RUSTFLAGS='-C target-cpu=native' cargo install sha1dir
```

## Usage

Run `sha1dir` to checksum the current directory, or run `sha1dir path/to/dir1
path/to/dir2 ...` to checksum one or more other directories.

## Behavior

The checksum is computed as the bitwise XOR of SHA-1 hashes one per directory
entry. The hash for each directory entry is the hash of the following body:

- For regular files — the one byte `'f'`, 4 little endian bytes for the path
  length, the bytes of the path, 4 little endian bytes for the Unix file mode
  as given by st\_mode, and finally the file contents.

- For symbolic links — the one byte `'l'`, the path length / path / mode as for
  regular files, and then the path of the link target.

- For directories — the one byte `'d'`, and the path length / path / mode.

The resulting checksum is 160 bits wide like SHA-1.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
