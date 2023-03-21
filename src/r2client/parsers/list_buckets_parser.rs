use serde::Deserialize;
use serde_xml_rs::from_reader;
use std::io::Cursor;

#[derive(Debug, Deserialize)]
#[serde(rename = "ListAllMyBucketsResult")]
pub struct ListAllMyBucketsResult {
    #[serde(rename = "Buckets")]
    pub buckets: Buckets,
    #[serde(rename = "Owner")]
    pub owner: Owner,
}

#[derive(Debug, Deserialize)]
pub struct Buckets {
    #[serde(rename = "Bucket")]
    pub bucket: Vec<Bucket>,
}

#[derive(Debug, Deserialize)]
pub struct Bucket {
    #[serde(rename = "CreationDate")]
    pub creation_date: String,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "ID")]
    pub id: String,
}

pub fn parse(input: String) -> ListAllMyBucketsResult {
    let reader = Cursor::new(input);
    let parsed: ListAllMyBucketsResult = from_reader(reader).unwrap();
    parsed
}