use fuser::{Filesystem, MountOption};
use serde_json::Value;
use std::error::Error;
use std::{env, process};
use dotenv::dotenv;

struct R2FS {
    cf_account_id: String,
    cf_api_token: String,
}

impl Filesystem for R2FS {
    // Implement the FUSE operations here
}

fn get_r2(client: &reqwest::blocking::Client, fs: &R2FS) -> Result<Vec<String>, Box<dyn Error>> {
    let endpoint = format!("https://api.cloudflare.com/client/v4/accounts/{}/r2/buckets", fs.cf_account_id);
    let response = client
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", fs.cf_api_token))
        .send()?;

    let json: Value = response.json()?;
    let namespaces = json["result"]["buckets"]
        .as_array()
        .ok_or("Unexpected JSON format")?;

    let mut buckets = Vec::new();
    for namespace in namespaces {
        let name = namespace["name"].as_str().unwrap_or_default();
        if !name.is_empty() {
            buckets.push(name.to_owned());
        }
    }

    println!("[INFO] Received buckets: {:#?}", buckets);

    Ok(buckets)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting the R2FS...");

    dotenv().ok(); // load the .env file
    let cf_account_id = env::var("ACCOUNT_ID").unwrap();
    let cf_api_token = env::var("CLOUDFLARE_API_TOKEN").unwrap();

    let fs = R2FS {
        cf_account_id: cf_account_id.clone(),
        cf_api_token: cf_api_token.clone(),
    };
    println!("FS: {}", fs.cf_account_id);

    // Get the mountpoint argument
    let mountpoint = match env::args_os().nth(1) {
        Some(mountpoint) => mountpoint,
        None => {
            eprintln!("Usage: {} <mountpoint>", env::args().next().unwrap());
            process::exit(1);
        }
    };
    println!("Mount at: {:?}", mountpoint);

    let client = reqwest::blocking::Client::new();
    let res = get_r2(&client, &fs)?;

    // Set up the mount options
    let mut mount_options = Vec::new();
    mount_options.push(MountOption::RO);
    mount_options.push(MountOption::FSName("r2".to_string()));

    // Mount the file system
    // fuser::mount2(R2FS, &mountpoint, &mount_options).unwrap();

    Ok(())
}