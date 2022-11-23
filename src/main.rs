//! [![github]](https://github.com/dtolnay/sha1dir)&ensp;[![crates-io]](https://crates.io/crates/sha1dir)&ensp;[![docs-rs]](https://docs.rs/sha1dir)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

#![allow(
    clippy::cast_possible_truncation,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::unseparated_literal_suffix
)]

use clap::Parser;
use memmap::Mmap;
use parking_lot::Mutex;
use rayon::{Scope, ThreadPoolBuilder};
use sha1::{Digest, Sha1};
use std::cmp;
use std::env;
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::{self, File, Metadata};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Once;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn die<P: AsRef<Path>, E: Display>(path: P, error: E) -> ! {
    static DIE: Once = Once::new();

    DIE.call_once(|| {
        let path = path.as_ref().display();
        let _ = writeln!(io::stderr(), "sha1sum: {}: {}", path, error);
        process::exit(1);
    });

    unreachable!()
}

#[derive(Debug, Parser)]
#[command(about = "Compute checksum of directory.", version, author)]
struct Opt {
    /// Number of hashes to compute in parallel
    #[arg(short)]
    jobs: Option<usize>,

    /// Directories to hash
    #[arg(value_name = "DIR")]
    dirs: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::parse();

    configure_thread_pool(&opt);

    if opt.dirs.is_empty() {
        let checksum = checksum_current_dir();
        let _ = writeln!(io::stdout(), "{}", checksum);
        return;
    }

    let absolute_dirs: Vec<_> = opt.dirs.iter().map(canonicalize).collect();
    for (canonical, label) in absolute_dirs.into_iter().zip(opt.dirs) {
        debug_assert!(canonical.is_absolute());
        if let Err(error) = env::set_current_dir(canonical) {
            die(label, error);
        }
        let checksum = checksum_current_dir();
        let _ = writeln!(io::stdout(), "{}  {}", checksum, label.display());
    }
}

fn configure_thread_pool(opt: &Opt) {
    let threads = if let Some(jobs) = opt.jobs {
        jobs
    } else {
        // Limit to 8 threads by default to avoid thrashing disk.
        cmp::min(num_cpus::get(), 8)
    };

    let result = ThreadPoolBuilder::new().num_threads(threads).build_global();

    // This is the only time the thread pool is initialized.
    result.unwrap();
}

fn canonicalize<P: AsRef<Path>>(path: P) -> PathBuf {
    match fs::canonicalize(&path) {
        Ok(canonical) => canonical,
        Err(error) => die(path, error),
    }
}

struct Checksum {
    bytes: Mutex<[u8; 20]>,
}

impl Checksum {
    fn new() -> Self {
        Checksum {
            bytes: Mutex::new([0u8; 20]),
        }
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.bytes.lock().as_ref() {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl Checksum {
    fn put(&self, rhs: Sha1) {
        for (lhs, rhs) in self.bytes.lock().iter_mut().zip(rhs.finalize()) {
            *lhs ^= rhs;
        }
    }
}

fn checksum_current_dir() -> Checksum {
    let checksum = Checksum::new();
    rayon::scope(|scope| entry(scope, &checksum, Path::new(".")));
    checksum
}

fn entry<'scope>(scope: &Scope<'scope>, checksum: &'scope Checksum, path: &Path) {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) => die(path, error),
    };

    let file_type = metadata.file_type();
    let result = if file_type.is_file() {
        file(checksum, path, metadata)
    } else if file_type.is_symlink() {
        symlink(checksum, path, metadata)
    } else if file_type.is_dir() {
        dir(scope, checksum, path, metadata)
    } else if file_type.is_socket() {
        socket(checksum, path, metadata)
    } else {
        die(path, "Unsupported file type");
    };

    if let Err(error) = result {
        die(path, error);
    }
}

fn file(checksum: &Checksum, path: &Path, metadata: Metadata) -> Result<()> {
    let mut sha = begin(path, &metadata, b'f');

    // Enforced by memmap: "memory map must have a non-zero length"
    if metadata.len() > 0 {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        sha.update(&mmap);
    }

    checksum.put(sha);

    Ok(())
}

fn symlink(checksum: &Checksum, path: &Path, metadata: Metadata) -> Result<()> {
    let mut sha = begin(path, &metadata, b'l');
    sha.update(path.read_link()?.as_os_str().as_bytes());
    checksum.put(sha);

    Ok(())
}

fn dir<'scope>(
    scope: &Scope<'scope>,
    checksum: &'scope Checksum,
    path: &Path,
    metadata: Metadata,
) -> Result<()> {
    let sha = begin(path, &metadata, b'd');
    checksum.put(sha);

    for child in path.read_dir()? {
        let child = child?.path();
        scope.spawn(move |scope| entry(scope, checksum, &child));
    }

    Ok(())
}

fn socket(checksum: &Checksum, path: &Path, metadata: Metadata) -> Result<()> {
    let sha = begin(path, &metadata, b's');
    checksum.put(sha);

    Ok(())
}

fn begin(path: &Path, metadata: &Metadata, kind: u8) -> Sha1 {
    let mut sha = Sha1::new();
    let path_bytes = path.as_os_str().as_bytes();
    sha.update([kind]);
    sha.update((path_bytes.len() as u32).to_le_bytes());
    sha.update(path_bytes);
    sha.update(metadata.mode().to_le_bytes());
    sha
}

#[test]
fn test_cli() {
    <Opt as clap::CommandFactory>::command().debug_assert();
}
