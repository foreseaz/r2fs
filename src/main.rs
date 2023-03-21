use fuser::{Filesystem, MountOption};
use r2client::R2Client;
use std::error::Error;
use std::{env, process};
use dotenv::dotenv;

mod r2client;

struct R2FS {
    r2_client: R2Client,
}

impl Filesystem for R2FS {
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting the R2FS...");

    dotenv().ok(); // load the .env file
    let cf_account_id = env::var("ACCOUNT_ID").unwrap();
    let r2_access_key_id = env::var("R2_ACCESS_KEY_ID").unwrap();
    let r2_secret_access_key = env::var("R2_SECRET_ACCESS_KEY").unwrap();

    let fs = R2FS {
        r2_client: R2Client::new(cf_account_id, r2_access_key_id, r2_secret_access_key)
    };

    // Get the mountpoint argument
    let mountpoint = match env::args_os().nth(1) {
        Some(mountpoint) => mountpoint,
        None => {
            eprintln!("Usage: {} <mountpoint>", env::args().next().unwrap());
            process::exit(1);
        }
    };
    println!("Mount at: {:?}", mountpoint);

    let res = fs.r2_client.list_buckets()?;
    println!("\t\tlist_buckets_parser result{:#?}", res);

    // Set up the mount options
    let mut mount_options = Vec::new();
    mount_options.push(MountOption::RO);
    mount_options.push(MountOption::FSName("r2".to_string()));

    // Mount the file system
    // fuser::mount2(R2FS, &mountpoint, &mount_options).unwrap();

    Ok(())
}