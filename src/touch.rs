use std::path::Path;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;

pub struct Args<'a> {
    pub paths: Vec<&'a Path>,
}

pub fn main(args: Args) -> i32 {
    if args.paths.is_empty() {
        eprintln!("missing file operand");
        eprintln!("Try 'touch --help' for more information");
        return crate::EXIT_CODE_INVALID_USAGE;
    }

    let mut status = 0;
    for fpath in args.paths {
        let flags = OFlag::O_CREAT | OFlag::O_WRONLY;
        let mode = Mode::S_IRUSR | Mode::S_IWUSR; // user can read+write
        let mode = mode | Mode::S_IRGRP | Mode::S_IWGRP; // group can read+write
        let mode = mode | Mode::S_IROTH | Mode::S_IWOTH; // others can read+write

        let fd = match nix::fcntl::open(fpath, flags, mode) {
            Ok(fd) => fd,
            Err(e) => {
                eprintln!("Unable to create file '{}': {}", fpath.display(), e);
                status = crate::EXIT_CODE_UNKNOWN_ERR;
                continue
            }
        };

        if let Err(e) = nix::unistd::close(fd) {
            eprintln!("Error closing file '{}': {}", fpath.display(), e);
            // don't set the status code here since, really, the fd will be closed regardless of
            // any error. The stderr message is here just to let the user know that _something_
            // happened. If we were writing anything to the file, there would be potential for
            // dataloss, but we aren't, so we don't care.
        }
    }
    return status;
}
