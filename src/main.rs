/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(clippy::needless_return)]

mod ls;
mod mkdir;
mod rmdir;
mod touch;

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::fmt::Display;

const EXIT_CODE_INVALID_USAGE: i32 = 1;
const EXIT_CODE_NO_CWD: i32 = 2;
const EXIT_CODE_READ_DIR: i32 = 3;
const EXIT_CODE_UNKNOWN_ERR: i32 = 255;

/// We put the actual main code inside another function so that we aren't calling exit() without
/// calling destructors. This is just the required rust entrypoint.
fn main() {
    match main_code() {
        Some(exit_code) => std::process::exit(exit_code),
        None => return,
    }
}

fn main_code() -> Option<i32> {
    let mut args: Vec<_> = std::env::args_os().collect();
    if args.is_empty() {
        print_usage();
        return Some(EXIT_CODE_INVALID_USAGE);
    } else {
        let cmd = match executable_name(&args[0]) {
            Some(cmd) => cmd,
            None => {
                print_usage();
                return Some(EXIT_CODE_INVALID_USAGE);
            }
        };

        if cmd == "busycrate" {
            if args.len() < 2 {
                print_usage();
                return None;
            }
            return run_with_args(cmd, &args[1..]);
        } else {
            if let Some(exe) = executable_name(&args[0]) {
                // If we were executed using a file path, like '/usr/bin/ls', clap won't recognize
                // the command. Here we strip out the leading directory parts, leaving just the
                // name of the executable. This is what clap matches on, and is consistent with
                // subcommand-style execution, like `busycrate ls`.
                let cmd_name = exe.to_os_string();
                args[0] = cmd_name;
            }
            return run_with_args(OsStr::new("busycrate"), &args[0..]);
        }
    }
}

/// The first cli argument contains the command that was used to execute the executable. If this
/// was done using $PATH searching, it'll be exactly equal to the name of the command we want to
/// run. Otherwise, it'll be the file path to the binary. This function finds the last file name in
/// the potential file path, or `None` if it wasn't a valid path.
fn executable_name(first_cli_arg: &OsStr) -> Option<&OsStr> {
    let cmd: &Path = first_cli_arg.as_ref();
    let cmd = match cmd.file_name() {
        Some(c) => c,
        None => {
            eprintln!("{}: not a command", cmd.display());
            return None;
        }
    };
    Some(cmd)
}

/// Main function with the "busycrate" argument split off from the rest
fn run_with_args(busycrate: &OsStr, args: &[OsString]) -> Option<i32> {
    use clap::{App, Arg, SubCommand};

    fn map_os_args_to_path_vec<'a>(os_args: clap::OsValues<'a>) -> Vec<&'a Path> {
        os_args.map(|os_str| os_str.as_ref()).collect()
    };

    let cmd = [busycrate.to_os_string()];
    let args = cmd.iter().chain(args);
    let app = App::new("BusyCrate")
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("Collection of Unix utilities")
        // these commands are often logged, written to files, etc.
        // color is usually unnecessary and potentially harmful
        .global_setting(clap::AppSettings::ColorNever)
        .subcommand(
            SubCommand::with_name("ls")
                .about("List directory contents")
                .args(&[
                    Arg::with_name("files")
                        .takes_value(true)
                        .multiple(true),
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("List hidden files"),
                    Arg::with_name("dirnames")
                        .short("d")
                        .long("directory")
                        .help("List directory names, not their contents"),
                ])
        )
        .subcommand(
            SubCommand::with_name("touch")
                .about("Create files and update their modified or access times")
                .args(&[
                    Arg::with_name("files")
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                    Arg::with_name("no-create")
                        .short("c")
                        .long("no-create")
                        .help("Do not create any files"),
                    Arg::with_name("atime-only")
                        .short("a")
                        .long("atime")
                        .help("Only update atime"),
                    Arg::with_name("mtime-only")
                        .short("m")
                        .long("mtime")
                        .help("Only update mtime")
                ])
        )
        .subcommand(
            SubCommand::with_name("mkdir")
                .about("Create directories")
                .args(&[
                    Arg::with_name("dirs")
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                    Arg::with_name("parents")
                        .short("p")
                        .long("parents")
                        .help("Create parent directories if they don't exist")
                ]),
        )
        .subcommand(
            SubCommand::with_name("rmdir")
                .about("Remove empty directories")
                .arg(
                    Arg::with_name("dirs")
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                ),
        );

    let matches = app.get_matches_from(args);
    if let Some(ls_args) = matches.subcommand_matches("ls") {
        let paths = ls_args
            .values_of_os("files")
            .map(map_os_args_to_path_vec)
            .unwrap_or(Vec::new());

        let ls_args = ls::Args {
            paths,
            all: ls_args.is_present("all"),
            shallow_dirs: ls_args.is_present("dirnames"),
        };
        return Some(ls::main(ls_args));
    } else if let Some(touch_args) = matches.subcommand_matches("touch") {
        let paths = touch_args
            .values_of_os("files")
            .map(map_os_args_to_path_vec)
            .unwrap_or(Vec::new());

        let create = !touch_args.is_present("no-create");
        let mtime_only = touch_args.is_present("mtime-only");
        let atime_only = touch_args.is_present("atime-only");
        let atime = atime_only || !mtime_only;
        let mtime = mtime_only || !atime_only;

        let touch_args = touch::Args {
            paths,
            create,
            atime,
            mtime,
        };
        return Some(touch::main(touch_args) as i32);
    } else if let Some(mkdir_args) = matches.subcommand_matches("mkdir") {
        let paths = mkdir_args
            .values_of_os("dirs")
            .map(map_os_args_to_path_vec)
            .unwrap_or(Vec::new());

        let mkdir_args = mkdir::Args {
            create_parents: mkdir_args.is_present("parents"),
            paths,
        };
        return Some(mkdir::main(mkdir_args));
    } else if let Some(rmdir_args) = matches.subcommand_matches("rmdir") {
        let paths = rmdir_args
            .values_of_os("dirs")
            .map(map_os_args_to_path_vec)
            .unwrap_or(Vec::new());

        let rmdir_args = rmdir::Args { paths };
        return Some(rmdir::main(rmdir_args));
    } else {
        print_usage();
        return Some(1);
    }
}

fn print_usage() {
    println!(
        "Usage: busycrate [--help] <command> [options]
                     <command> [options]"
    );
}

/// Common exit codes across all commands
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    InvalidUsage = 1,
    NoCwd = 2,
    ReadDir = 3,
    Stat = 4,
    Time = 5,
    UnknownErr = 255,
}

pub struct FdPathDropper<P: Display>(i32, P);

impl<P: Display> FdPathDropper<P> {
    pub fn new(fd: i32, fpath: P) -> Self {
        Self(fd, fpath)
    }
}

impl<P: Display> Drop for FdPathDropper<P> {
    fn drop(&mut self) {
        if let Err(e) = nix::unistd::close(self.0) {
            eprintln!("Error closing file '{}': {}", self.1, e);
            // don't set the status code here since, really, the fd will be closed regardless of
            // any error. The stderr message is here just to let the user know that _something_
            // happened. If we were writing anything to the file, there would be potential for
            // dataloss, but we aren't, so we don't care.
        }
    }
}
