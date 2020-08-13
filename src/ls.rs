/* This Source Code Form is subject to the terms of the Mozilla Public
 *  * License, v. 2.0. If a copy of the MPL was not distributed with this
 *   * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;

pub struct Args<'a> {
    pub paths: Vec<&'a Path>,
    pub all: bool,
}

#[derive(Clone, Copy)]
pub struct PrintRules {
    print_hidden: bool,
}

pub fn main(args: Args) -> i32 {
    // These declarations hoist the lifetime of the backing array for the default paths.
    // Essentially, the memory for the default paths is always allocated, but only conditionally
    // used. The type annotations are just there for clarity.
    let cwd: PathBuf;
    let default_paths: [&Path; 1];

    // if we weren't provided any directories to list out, point a slice towards a default array
    // containing the current working directory. loop through that instead
    let paths = if args.paths.is_empty() {
        match std::env::current_dir() {
            Ok(d) => cwd = d,
            Err(e) => {
                eprintln!("Unable to determine current directory: {}", e);
                return crate::EXIT_CODE_NO_CWD;
            }
        }
        default_paths = [cwd.as_path()];
        &default_paths[..]
    } else {
        &args.paths[..]
    };

    let print_rules = PrintRules {
        print_hidden: args.all,
    };

    let mut status = 0;
    for &dir_name in paths {
        // the rust standard library filters out '.' and '..' for cross-platform compatibility
        // reasons, so we turn to the nix crate to actually list out directories
        let dir = nix::dir::Dir::open(dir_name, OFlag::O_RDONLY, Mode::empty());
        let mut dir = match dir {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("{:?}: {}", dir_name, e);
                status = crate::EXIT_CODE_READ_DIR;
                continue
            }
        };

        for entry in dir.iter() {
            match entry {
                Ok(entry) => maybe_print_entry(&entry, print_rules),
                Err(e) => eprintln!("Error reading {:?}: {}", dir_name, e),
            }
        }
    }
    return status
}

fn maybe_print_entry(entry: &nix::dir::Entry, print_rules: PrintRules) {
    let fname = entry.file_name();
    if !print_rules.print_hidden && entry_is_hidden(&fname) {
        return
    }
    println!("{}", fname.to_string_lossy())
}

fn entry_is_hidden(entry_name: &CStr) -> bool {
    // according to POSIX.2, ls only cares about whether or not the filename starts with a '.', and
    // doesn't consider the hidden bit flag that exists on some filesystems (FAT, ntfs)
    entry_name.to_bytes()[0] == b'.'
}
