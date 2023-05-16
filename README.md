sha1dir
=======

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/sha1dir-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/sha1dir)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sha1dir.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sha1dir)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/sha1dir/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/sha1dir/actions?query=branch%3Amaster)

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

The mode for the topmost directory may be excluded from the checksum using the
`--exclude-rootdir-metadata` command line option.

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
