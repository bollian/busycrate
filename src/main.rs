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
        .about("List directory contents")
        .subcommand(
            SubCommand::with_name("ls")
                .arg(Arg::with_name("files").takes_value(true).multiple(true))
                .arg(Arg::with_name("all").short("a").long("all"))
                .arg(Arg::with_name("dirnames").short("d"))
        )
        .subcommand(
            SubCommand::with_name("touch").arg(
                Arg::with_name("files")
                    .takes_value(true)
                    .multiple(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("mkdir")
                .arg(
                    Arg::with_name("dirs")
                        .takes_value(true)
                        .multiple(true)
                        .required(true),
                )
                .arg(Arg::with_name("parents").long("parents").short("p")),
        )
        .subcommand(
            SubCommand::with_name("rmdir").arg(
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

        let touch_args = touch::Args { paths };
        return Some(touch::main(touch_args));
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
