//! [![github]](https://github.com/dtolnay/sha1dir)&ensp;[![crates-io]](https://crates.io/crates/sha1dir)&ensp;[![docs-rs]](https://docs.rs/sha1dir)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

#![allow(
    clippy::cast_possible_truncation,
    clippy::let_underscore_untyped,
    clippy::needless_collect,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::unseparated_literal_suffix
)]

use clap::Parser;
use std::cmp;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha1dir::{canonicalize, checksum_current_dir, configure_thread_pool, die};

#[derive(Debug, Parser)]
#[command(about = "Compute checksum of directory.", version, author)]
struct Opt {
    /// Number of hashes to compute in parallel
    #[arg(short)]
    jobs: Option<usize>,

    /// Directories to hash
    #[arg(value_name = "DIR")]
    dirs: Vec<PathBuf>,

    /// Whether to ignore unknown filetypes (otherwise fatal)
    #[arg(long)]
    ignore_unknown_filetypes: bool,
}

fn main() {
    let opt = Opt::parse();

    let threads = if let Some(jobs) = opt.jobs {
        jobs
    } else {
        // Limit to 8 threads by default to avoid thrashing disk.
        cmp::min(num_cpus::get(), 8)
    };

    configure_thread_pool(threads);

    if opt.dirs.is_empty() {
        let path = Path::new(".");
        let checksum = checksum_current_dir(path, opt.ignore_unknown_filetypes);
        let _ = writeln!(io::stdout(), "{}", checksum);
        return;
    }

    let absolute_dirs: Vec<_> = opt.dirs.iter().map(canonicalize).collect();
    for (canonical, label) in absolute_dirs.into_iter().zip(opt.dirs) {
        debug_assert!(canonical.is_absolute());
        if let Err(error) = env::set_current_dir(canonical) {
            die(label, error);
        }
        let checksum = checksum_current_dir(&label, opt.ignore_unknown_filetypes);
        let _ = writeln!(io::stdout(), "{}  {}", checksum, label.display());
    }
}
