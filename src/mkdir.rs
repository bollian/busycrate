use std::path::Path;
use nix::sys::stat::Mode;

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
    for fpath in args.paths {
        let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IXUSR; // user can read+write+execute
        let mode = mode | Mode::S_IRGRP | Mode::S_IWGRP | Mode::S_IXGRP; // group can read+write+execute
        let mode = mode | Mode::S_IROTH | Mode::S_IXOTH; // others can read+execute

        match nix::unistd::mkdir(fpath, mode) {
            Ok(()) => {},
            Err(e) => {
                eprintln!("Unable to create directory '{}': {}", fpath.display(), e);
                status = crate::EXIT_CODE_UNKNOWN_ERR;
                continue
            }
        };
    }
    return status;
}
