use sha2::{Sha256, Digest};

fn sha256_string(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn get_sig_headers(
    host: &str,
    url: &str,
    r2_access: &str,
    r2_secret: &str,
) -> reqwest::header::HeaderMap {
    let datetime = chrono::Utc::now();
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "X-Amz-Date",
        datetime
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
            .parse()
            .unwrap(),
    );
    headers.insert("host", host.parse().unwrap());
    let payload = "".to_string();
    let hashed_payload = sha256_string(payload);
    headers.insert("x-amz-content-sha256", hashed_payload.parse().unwrap());

    let s = aws_sign_v4::AwsSign::new(
        "GET",
        &url,
        &datetime,
        &headers,
        "auto",
        &r2_access,
        &r2_secret,
        "s3",
        ""
    );
    let signature = s.sign();
    println!("\tSig: {:#?}", signature);
    headers.insert(reqwest::header::AUTHORIZATION, signature.parse().unwrap());

    headers
}