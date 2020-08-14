/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::mem::forget;

pub struct Args<'a> {
    pub paths: Vec<&'a Path>,
    pub all: bool,
    pub shallow_dirs: bool,
}

#[derive(Clone, Copy)]
struct PrintRules {
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

    let mut print_shallow = Vec::new(); // print just the provided path, used for testing existence
    let mut print_contents = Vec::new(); // print directory contents

    // print files first
    for &fpath in paths {
        if !fpath.exists() {
            eprintln!("'{}': No such file or directory", fpath.display());
        } else if !fpath.is_dir() || args.shallow_dirs {
            print_shallow.push(fpath);
        } else { // this is an existing directory
            print_contents.push(fpath);
        }
    }

    for &fpath in print_shallow.iter() {
        println!("{}", fpath.display());
    }

    let mut group_spacing = !print_shallow.is_empty();
    let label_dir_groups = group_spacing || print_contents.len() > 1;

    for &dpath in print_contents.iter() {
        let dir = nix::dir::Dir::open(dpath, OFlag::O_RDONLY, Mode::empty());
        let mut dir = match dir {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("{:?}: {}", dpath, e);
                status = crate::EXIT_CODE_READ_DIR;
                continue;
            }
        };

        if group_spacing {
            println!()
        }
        if label_dir_groups {
            println!("{}:", dpath.display());
        }

        for entry in dir.iter() {
            match entry {
                Ok(entry) => maybe_print_entry(&entry.file_name(), print_rules),
                Err(e) => eprintln!("Error reading {}: {}", dpath.display(), e),
            }
        }

        group_spacing = true;
    }

    // the program is short-lived and the OS is gonna clean up after us anyways
    forget(print_shallow);
    forget(print_contents);

    return status;
}

fn maybe_print_entry(entry: &CStr, print_rules: PrintRules) {
    if !print_rules.print_hidden && entry_is_hidden(&entry) {
        return;
    }
    // TODO: display CStr without dynamically allocating
    println!("{}", entry.to_string_lossy())
}

fn entry_is_hidden(entry_name: &CStr) -> bool {
    // according to POSIX.2, ls only cares about whether or not the filename starts with a '.', and
    // doesn't consider the hidden bit flag that exists on some filesystems (FAT, ntfs)
    entry_name.to_bytes()[0] == b'.'
}
