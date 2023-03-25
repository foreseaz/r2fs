use serde::Deserialize;
use serde_xml_rs::from_reader;
use std::io::Cursor;

#[derive(Debug, Deserialize)]
#[serde(rename = "ListBucketResult")]
pub struct ListBucketObjectsResult {
    #[serde(rename = "Name")]
    pub name: String, // bucket naem
    #[serde(rename = "Contents")]
    pub contents: Vec<Content>,
    #[serde(rename = "IsTruncated")]
    pub is_truncated: bool,
    #[serde(rename = "MaxKeys")]
    pub max_keys: i32,
    #[serde(rename = "KeyCount")]
    pub key_count: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Contents")]
pub struct Content { // single object, all are files
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Size")]
    pub size: i64,
    #[serde(rename = "LastModified")]
    pub last_modified: String,
    #[serde(rename = "ETag")]
    pub etag: String,
    #[serde(rename = "StorageClass")]
    pub storage_class: String,
}

pub fn parse(input: String) -> ListBucketObjectsResult {
    let reader = Cursor::new(input);
    let parsed: ListBucketObjectsResult = from_reader(reader).unwrap();
    parsed
}