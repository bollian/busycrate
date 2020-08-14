/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use nix::sys::stat::Mode;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct Args<'a> {
    pub create_parents: bool,
    pub paths: Vec<&'a Path>,
}

pub fn main(args: Args) -> i32 {
    if args.paths.is_empty() {
        eprintln!("missing file operand");
        eprintln!("Try 'mkdir --help' for more information");
        return crate::EXIT_CODE_INVALID_USAGE;
    }

    let mut status = 0;
    'ARG_LOOP: for total_path in args.paths {
        let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IXUSR; // user can read+write+execute
        let mode = mode | Mode::S_IRGRP | Mode::S_IWGRP | Mode::S_IXGRP; // group can read+write+execute
        let mode = mode | Mode::S_IROTH | Mode::S_IXOTH; // others can read+execute

        let no_parents = [total_path.as_os_str()];
        let mut with_parents; // collection of ['first', 'first/second', 'first/second/third', ...]
        let creations = if args.create_parents {
            // split total_path into slices over all prefix directories and collect them into a Vec
            with_parents = Vec::new();
            let raw = total_path.as_os_str().as_bytes();
            let mut search_from = 0;

            while let Some(i) = raw
                .get(search_from..) // start search from this index, checked slice
                .unwrap_or(&[]) // trick so position() returns None when we've reached the end
                .iter()
                .position(|&b| b == b'/')
            {
                let prefix_end = search_from + i;
                search_from = prefix_end + 1;
                with_parents.push(OsStrExt::from_bytes(&raw[0..prefix_end]));
            }

            // the path may not end with '/', so we have to add the final component
            if search_from < raw.len() {
                with_parents.push(OsStrExt::from_bytes(raw));
            }

            with_parents.as_slice()
        } else {
            &no_parents[..]
        };

        for &fpath in creations {
            match nix::unistd::mkdir(fpath, mode) {
                Ok(()) => {}
                Err(nix::Error::Sys(nix::errno::Errno::EEXIST)) if args.create_parents => {
                    // ignore this error
                    // allow directories to exist already if we're creating each component in the
                    // path
                }
                Err(e) => {
                    eprintln!(
                        "Unable to create directory '{}': {}",
                        Path::new(fpath).display(),
                        e
                    );
                    status = crate::EXIT_CODE_UNKNOWN_ERR;
                    continue 'ARG_LOOP;
                }
            }
        }
    }

    status
}
