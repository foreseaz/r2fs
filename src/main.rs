use fuser::{Filesystem, MountOption};
use std::{env, process};

struct R2FileSystem;

impl Filesystem for R2FileSystem {
    // Implement the FUSE operations here
}

fn main() {
    println!("Starting the R2FS...");

    // Get the mountpoint argument
    let mountpoint = match env::args_os().nth(1) {
        Some(mountpoint) => mountpoint,
        None => {
            eprintln!("Usage: {} <mountpoint>", env::args().next().unwrap());
            process::exit(1);
        }
    };

    // Set up the mount options
    let mut mount_options = Vec::new();
    mount_options.push(MountOption::RO);
    mount_options.push(MountOption::FSName("r2".to_string()));

    // Mount the file system
    fuser::mount2(R2FileSystem, &mountpoint, &mount_options).unwrap();
}