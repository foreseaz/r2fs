pub(crate) mod parsers;
mod sig;

use parsers::list_bucket_objects_parser::ListBucketObjectsResult;
use parsers::list_buckets_parser::ListAllMyBucketsResult;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct R2Client {
    cf_account_id: String,
    r2_access: String,
    r2_secret: String,
    client: reqwest::blocking::Client,
}

impl R2Client {
    pub fn new(
        cf_account_id: String,
        r2_access_key_id: String,
        r2_secret_access_key: String,
    ) -> Self {
        let client = reqwest::blocking::Client::new();
        Self {
            cf_account_id,
            r2_access: r2_access_key_id,
            r2_secret: r2_secret_access_key,
            client,
        }
    }

    pub fn list_buckets(&self) -> Result<ListAllMyBucketsResult, Box<dyn Error>> {
        let host = format!("{}.r2.cloudflarestorage.com", self.cf_account_id);
        let endpoint = format!("https://{}", host);
        println!("[DEBUG] requesting endpoint: {:?}", endpoint);

        let signed_headers =
            sig::get_sig_headers(&host, &endpoint, &self.r2_access, &self.r2_secret);
        let res = self
            .client
            .get(endpoint)
            .headers(signed_headers.to_owned())
            .body("")
            .send()?;

        let body = res.text()?;

        let result = parsers::list_buckets_parser::parse(body);
        Ok(result)
    }

    pub fn list_bucket_objects(
        &self,
        bucket_name: &str,
    ) -> Result<ListBucketObjectsResult, Box<dyn Error>> {
        let host = format!("{}.r2.cloudflarestorage.com", self.cf_account_id);
        let endpoint = format!("https://{}/{}?list-type=2", host, bucket_name); // using ListObjectV2
        println!("[DEBUG] requesting endpoint: {:?}", endpoint);

        let signed_headers =
            sig::get_sig_headers(&host, &endpoint, &self.r2_access, &self.r2_secret);

        let res = self
            .client
            .get(endpoint)
            .headers(signed_headers.to_owned())
            .body("")
            .send()?;

        let body = res.text()?;

        let result = parsers::list_bucket_objects_parser::parse(body);
        Ok(result)
    }

    pub fn get_object(
        &self,
        bucket_name: &str,
        object_key: &str,
        local_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let host = format!("{}.r2.cloudflarestorage.com", self.cf_account_id);
        let endpoint = format!("https://{}/{}/{}", host, bucket_name, object_key);

        let signed_headers =
            sig::get_sig_headers(&host, &endpoint, &self.r2_access, &self.r2_secret);

        let mut res = self
            .client
            .get(endpoint)
            .headers(signed_headers.to_owned())
            .send()?;

        if !res.status().is_success() {
            return Err("Failed to download object".into());
        }

        let mut file = File::create(local_path)?;

        res.copy_to(&mut file)?;

        Ok(())
    }
}
