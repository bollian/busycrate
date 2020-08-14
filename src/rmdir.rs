/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

pub struct Args<'a> {
    pub paths: Vec<&'a Path>,
}

pub fn main(args: Args) -> i32 {
    if args.paths.is_empty() {
        eprintln!("missing file operand");
        eprintln!("Try 'rmdir --help' for more information");
        return crate::EXIT_CODE_INVALID_USAGE;
    }

    let mut status = 0;
    for fpath in args.paths {
        match std::fs::remove_dir(fpath) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Unable to remove '{}': {}", fpath.display(), e);
                status = crate::EXIT_CODE_UNKNOWN_ERR;
            }
        }
    }

    return status;
}
