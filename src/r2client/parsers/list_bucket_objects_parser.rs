use serde::Deserialize;
use serde_xml_rs::from_reader;
use std::io::Cursor;

#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListBucketObjectsResult {
    #[serde(rename = "Name")]
    name: String, // bucket naem
    #[serde(rename = "Contents")]
    contents: Vec<Content>,
    #[serde(rename = "IsTruncated")]
    is_truncated: bool,
    #[serde(rename = "MaxKeys")]
    max_keys: i32,
    #[serde(rename = "KeyCount")]
    key_count: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Contents")]
pub struct Content { // single object, all are files
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Size")]
    size: i64,
    #[serde(rename = "LastModified")]
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
    #[serde(rename = "StorageClass")]
    storage_class: String,
}

pub fn parse(input: String) -> ListBucketObjectsResult {
    let reader = Cursor::new(input);
    let parsed: ListBucketObjectsResult = from_reader(reader).unwrap();
    parsed
}