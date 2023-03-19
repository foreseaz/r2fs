use fuser::{Filesystem, MountOption};
use serde_json::Value;
use std::error::Error;
use std::{env, process};
use dotenv::dotenv;

struct R2FS {
    r2_client: R2Client,
}

impl Filesystem for R2FS {
}

struct R2Client {
    cf_account_id: String,
    r2_access_key_id: String,
    r2_secret_access_key: String,
    client: reqwest::blocking::Client,
}

impl R2Client {
    fn new(cf_account_id: String, r2_access_key_id: String, r2_secret_access_key: String) -> Self {
        let client = reqwest::blocking::Client::new();
        Self {
            cf_account_id,
            r2_access_key_id,
            r2_secret_access_key,
            client,
        }
    }

    fn list_buckets(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", self.cf_account_id);
        let response = self.client
            .get(endpoint)
            .header("AccessKey", format!("{}", self.r2_access_key_id))
            .header("SecretKey", format!("{}", self.r2_secret_access_key))
            .send()?;

        println!("{:?}", response.text()?);

        // let json: Value = response.json()?;
        // let namespaces = json["result"]["buckets"]
        //     .as_array()
        //     .ok_or("Unexpected JSON format")?;

        let mut buckets = Vec::new();
        // for namespace in namespaces {
        //     let name = namespace["name"].as_str().unwrap_or_default();
        //     if !name.is_empty() {
        //         buckets.push(name.to_owned());
        //     }
        // }

        println!("[INFO] Received buckets: {:#?}", buckets);

        Ok(buckets)
    }
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

    // Set up the mount options
    let mut mount_options = Vec::new();
    mount_options.push(MountOption::RO);
    mount_options.push(MountOption::FSName("r2".to_string()));

    // Mount the file system
    // fuser::mount2(R2FS, &mountpoint, &mount_options).unwrap();

    Ok(())
}