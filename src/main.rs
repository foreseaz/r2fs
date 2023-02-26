use fuser::{Filesystem, MountOption};
use serde_json::Value;
use std::error::Error;
use std::{env, process};

struct R2FileSystem;

impl Filesystem for R2FileSystem {
    // Implement the FUSE operations here
}

fn get_r2(client: &reqwest::blocking::Client) -> Result<Vec<String>, Box<dyn Error>> {
    let response = client
        .get("https://api.cloudflare.com/client/v4/accounts/:account_id/r2/buckets")
        .header("Authorization", "Bearer :token")
        .send()?;

    let json: Value = response.json()?;
    let namespaces = json["result"]
        .as_array()
        .ok_or("Unexpected JSON format")?;

    let mut buckets = Vec::new();
    for namespace in namespaces {
        let name = namespace["title"].as_str().unwrap_or_default();
        if !name.is_empty() {
            buckets.push(name.to_owned());
        }
    }

    Ok(buckets)
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

    let client = reqwest::blocking::Client::new();
    let res = get_r2(&client)?;
    // Print the list of buckets to confirm we received them
    println!("Buckets: {:?}", res);

    // Set up the mount options
    let mut mount_options = Vec::new();
    mount_options.push(MountOption::RO);
    mount_options.push(MountOption::FSName("r2".to_string()));

    // Mount the file system
    fuser::mount2(R2FileSystem, &mountpoint, &mount_options).unwrap();
}