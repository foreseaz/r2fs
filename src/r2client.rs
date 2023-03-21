
mod sig;
mod parsers;

use std::error::Error;
use parsers::list_buckets_parser::ListAllMyBucketsResult;

pub struct R2Client {
    cf_account_id: String,
    r2_access: String,
    r2_secret: String,
    client: reqwest::blocking::Client,
}

impl R2Client {
    pub fn new(cf_account_id: String, r2_access_key_id: String, r2_secret_access_key: String) -> Self {
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

        let signed_headers = sig::get_sig_headers(&host, &self.r2_access, &self.r2_secret);
        println!("\n\nsigned_headers: {:?}", signed_headers);
        let res = self.client
            .get(endpoint)
            .headers(signed_headers.to_owned())
            .body("")
            .send()?;

        let body = res.text()?;
        println!("Body: {:?}", body);

        let result = parsers::list_buckets_parser::parse(body);
        Ok(result)
    }
}
