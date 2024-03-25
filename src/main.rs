use std::process::exit;

use clap::Parser;
use log::{debug, error, warn};
use nix::mount::{umount2, MntFlags};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "The path (or any other name) that the mount points will be checked on if they contain it")]
    path: String,
    #[arg(short, long, default_value_t = false, help = "Force all umounts")]
    force: bool,
    #[arg(short, long, default_value_t = false, help = "Exit on error. At default it will just force the mount point to go away")]
    exit_on_error: bool,
    #[arg(short, long, default_value_t = false, help = "Dump the structure this program contains. It can be usefull for something someday?. Oh and just give him any path so no complaining :D")]
    dump: bool,
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = Args::parse();
    let options = lfs_core::ReadOptions::default();
    let mut mounts = lfs_core::read_mounts(&options).unwrap();

    if args.dump {
        // only keep the one with size stats
        mounts.retain(|m| m.stats.is_ok());
        for mount in mounts {
            dbg!(mount);
        }
        exit(0);
    }

    mounts.retain(|m| m.info.mount_point.to_string_lossy().contains(&args.path));
    mounts.sort_by_key(|m| m.info.mount_point.to_string_lossy().len());
    mounts.reverse();

    debug!("Mounts with path: {:#?}", mounts);

    let mut flags = MntFlags::empty();

    if args.force {
        flags |= MntFlags::MNT_FORCE;
    }

    for m in mounts {
        debug!("Unmounting: {}", m.info.mount_point.to_string_lossy());
        if umount2(&m.info.mount_point, flags).is_err() {
            if args.exit_on_error {
                error!("Failed to unmount {} and exit_on_error is turn on, exiting...", m.info.mount_point.to_string_lossy());
                exit(1);
            }
            if args.force {
                error!("Failed to unmount {} and force is already turned on so this shouldn't happen, exiting...", m.info.mount_point.to_string_lossy());
                exit(1);
            }
            warn!("Failed to unmount: {}, retrying with force...", m.info.mount_point.to_string_lossy());
            if umount2(&m.info.mount_point, MntFlags::MNT_FORCE).is_err() {
                error!("Failed to unmount with force too, exiting...");
                exit(1);
            }
        }
    }
    debug!("All worked, exiting...");
    exit(0);
}
