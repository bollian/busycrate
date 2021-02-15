/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `touch` is mostly a utility for creating new files, but it can also be used to update the
//! atime/mtime of an existing file. Most of this implementation is dedicated to complexities
//! with the latter usecase.

use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::errno::Errno;
use nix::time;
use nix::sys::time::TimeSpec;
use std::path::Path;
use libc::timespec;
use crate::{ExitCode, FdPathDropper};

pub struct Args<'a> {
    /// Files to touch
    pub paths: Vec<&'a Path>,
    /// Create the file if it doesn't exist
    pub create: bool,
    /// Update atime of the file if it exists
    pub atime: bool,
    /// Update mtime of the file if it exists
    pub mtime: bool,
}

pub fn main(args: Args) -> ExitCode {
    match main_code(args) {
        Ok(_) => ExitCode::Success,
        Err(e) => e,
    }
}

fn main_code(args: Args) -> Result<(), ExitCode> {
    if args.paths.is_empty() {
        eprintln!("missing file operand");
        eprintln!("Try 'touch --help' for more information");
        return Err(ExitCode::InvalidUsage)
    }

    for fpath in args.paths {
        let mut flags = OFlag::O_WRONLY;
        if args.create {
            // allow the user to disable file creation. this will have the effect of changing the
            // atime/mtime in the case that the file exists, but is a nop if the file doesn't
            flags |= OFlag::O_CREAT;
        }

        let mode = Mode::S_IRUSR | Mode::S_IWUSR; // user can read+write
        let mode = mode | Mode::S_IRGRP | Mode::S_IWGRP; // group can read+write
        let mode = mode | Mode::S_IROTH | Mode::S_IWOTH; // others can read+write

        let fd = match nix::fcntl::open(fpath, flags, mode) {
            Ok(fd) => fd,
            // If the file doesn't exist, and we've been told not to create files, then this
            // condition isn't an error. That's just the expected behavior of that option.
            Err(nix::Error::Sys(Errno::ENOENT)) if !args.create => continue,
            Err(e) => {
                eprintln!("Unable to create file '{}': {}", fpath.display(), e);
                return Err(ExitCode::UnknownErr)
            }
        };
        let _dropper = FdPathDropper::new(fd, fpath.display());
        update_fd_times(fd, args.atime, args.mtime, fpath)?;
    }
    return Ok(())
}

/// If we've been configured to update file times, this function will set them to the system time.
fn update_fd_times(fd: i32, update_atime: bool, update_mtime: bool, fpath: &Path) -> Result<(), ExitCode> {
    if update_atime || update_mtime {
        // only bother with these extra syscalls if we're actually expected to update the times
        // on this file

        let clock = time::ClockId::CLOCK_REALTIME;
        let time_now = match time::clock_gettime(clock) {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("Unable to get system time: {}", e);
                return Err(ExitCode::Time);
            }
        };

        let (new_atime, new_mtime): (TimeSpec, TimeSpec);
        if !update_atime || !update_mtime {
            // in this case, we need to get the old times already set on the file
            let s = match nix::sys::stat::fstat(fd) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Unable to stat file '{}': {}", fpath.display(), e);
                    return Err(ExitCode::Stat);
                },
            };
            let st_atime = timespec {
                tv_sec: s.st_atime,
                tv_nsec: s.st_atime_nsec,
            }.into();
            let st_mtime = timespec {
                tv_sec: s.st_mtime,
                tv_nsec: s.st_mtime_nsec,
            }.into();

            new_atime = if !update_atime { st_atime } else { time_now };
            new_mtime = if !update_mtime { st_mtime } else { time_now };
        } else {
            new_atime = time_now;
            new_mtime = time_now;
        }

        if let Err(e) = nix::sys::stat::futimens(fd, &new_atime, &new_mtime) {
            eprintln!("Couldn't modify times on '{}': {}", fpath.display(), e);

            // Stat is a better description of the error here than Time since we're modifying a file's
            // metadata, not reading/setting clocks
            return Err(ExitCode::Stat);
        }
    }

    return Ok(());
}
