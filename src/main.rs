mod r2client;
mod utils;

use r2client::R2Client;
use std::error::Error;
use std::{env, process};
use std::collections::HashMap;
use dotenv::dotenv;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

struct R2FS {
    r2_client: R2Client,
    bucket: String,
    ino_attribute_map: HashMap<u64, FileAttr>, // ino -> FileAttr
    name_ino_map: HashMap<String, u64>, // filename -> ino
}

impl R2FS {
    fn new(
        cf_account_id: String,
        r2_access_key_id: String,
        r2_secret_access_key: String,
    ) -> R2FS {
        let r2_client = R2Client::new(cf_account_id, r2_access_key_id, r2_secret_access_key);

        R2FS {
            r2_client,
            bucket: String::new(),
            ino_attribute_map: HashMap::new(),
            name_ino_map: HashMap::new(),
        }
    }

    fn init_bucket_dirs(&mut self) {
        let list_buckets_res = self.r2_client.list_buckets();
        println!("\t\tlist_buckets_parser result{:#?}", list_buckets_res);
        let bucket_0 = &list_buckets_res.unwrap().buckets.bucket[0];
        let creation_date = &bucket_0.creation_date;
        self.bucket = bucket_0.name.clone();

        let bucket_attr: FileAttr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: utils::parse_system_time(&creation_date),
            mtime: utils::parse_system_time(&creation_date),
            ctime: utils::parse_system_time(&creation_date),
            crtime: utils::parse_system_time(&creation_date),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        };

        self.ino_attribute_map.insert(1, bucket_attr);
    }
}

impl Filesystem for R2FS {
    fn lookup(
        &mut self,
        _req: &Request,
        parent: u64,
        name: &OsStr,
        reply: ReplyEntry
    ) {
        println!("[lookup] calling, parent {:?}, name {:?}", parent, name);

        let name_str = name.to_str().unwrap_or("");
        if parent == 1 {
            if let Some(ino) = self.name_ino_map.get(name_str) {
                println!("\t[lookup] will retrieve fileAttr for ino {:?}", ino);
                if let Some(file_attr) = self.ino_attribute_map.get(&ino) {
                    reply.entry(&TTL, &file_attr, 0);
                    return;
                }
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("[getattr] calling, ino {:?}", ino);
        if let Some(file_attr) = self.ino_attribute_map.get(&ino) {
            reply.attr(&TTL, file_attr);
            return;
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        println!("[read] calling, ino: {:?}, offset: {:?}", ino, offset);

        if let Some(file_attr) = self.ino_attribute_map.get(&ino) {
            // reply.entry(&TTL, &file_attr, 0);
            // reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[0..]);
            return;
        }
        reply.error(ENOENT);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("[readdir] calling");
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let mut entries = Vec::new();

        // Add the current and parent directory entries
        const DIR_ATTR: FileAttr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: UNIX_EPOCH, // 1970-01-01 00:00:00
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 501,
            gid: 20,
            rdev: 0,
            flags: 0,
            blksize: 512,
        };
        entries.push((1, FileType::Directory, "."));
        entries.push((1, FileType::Directory, ".."));
        self.ino_attribute_map.insert(1, DIR_ATTR);
        self.name_ino_map.insert(".".to_string(), 1);
        self.name_ino_map.insert("..".to_string(), 1);

        // Retrieve the list of objects in the bucket
        let objects = self.r2_client.list_bucket_objects(&self.bucket).unwrap();

        // Add the objects as directory entries
        let mut ino_idx = 2; // offset "." and ".."
        for (_, obj) in objects.contents.iter().enumerate().skip(offset as usize) {
            // TODO: need handle the directory
            if obj.key.contains("/") {
                continue;
            }
            entries.push((ino_idx, FileType::RegularFile, &obj.key));

            self.ino_attribute_map.insert(ino_idx, FileAttr {
                ino: ino_idx,
                size: obj.size as u64,
                blocks: 0,
                atime: utils::parse_system_time(&obj.last_modified),
                mtime: utils::parse_system_time(&obj.last_modified),
                ctime: utils::parse_system_time(&obj.last_modified),
                crtime: utils::parse_system_time(&obj.last_modified),
                kind: FileType::RegularFile,
                perm: 0o644,
                nlink: 1,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
                blksize: 512,
            });
            self.name_ino_map.insert(obj.key.clone(), ino_idx);

            ino_idx = ino_idx + 1;
        }
        println!("\twill add entries: {:?}", entries);

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0 as u64, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

fn unmount(mountpoint: &str) -> Result<(), String> {
    let mnt_dir = std::ffi::CString::new(mountpoint).map_err(|_| "Invalid mountpoint")?;
    let result = unsafe { libc::unmount(mnt_dir.as_ptr(), libc::MNT_FORCE) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Failed to unmount mountpoint {}: {}", mountpoint, std::io::Error::last_os_error()))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting the R2FS...");

    // Get the mountpoint argument
    let mountpoint = match env::args_os().nth(1) {
        Some(mountpoint) => mountpoint,
        None => {
            eprintln!("Usage: {} <mountpoint>", env::args().next().unwrap());
            process::exit(1);
        }
    };
    println!("Mount at: {:?}", mountpoint);

    dotenv().ok(); // load the .env file
    let cf_account_id = env::var("ACCOUNT_ID").unwrap();
    let r2_access_key_id = env::var("R2_ACCESS_KEY_ID").unwrap();
    let r2_secret_access_key = env::var("R2_SECRET_ACCESS_KEY").unwrap();
    let mut fs = R2FS::new(cf_account_id, r2_access_key_id, r2_secret_access_key);
    fs.init_bucket_dirs();

    // Set up the mount options
    let mountpoint = mountpoint.clone().into_string().unwrap();

    let mut options = Vec::new();
    options.push(MountOption::RW);
    options.push(MountOption::AutoUnmount);
    options.push(MountOption::FSName("r2fs".to_string()));

    // Unmount existing FUSE mount at init
    if let Err(err) = unmount(&mountpoint) {
        eprintln!("Unmount at init: {}", err);
    }

    // Mount the file system
    match fuser::mount2(fs, &mountpoint, &options) {
        Ok(_) => println!("File system mounted successfully"),
        Err(err) => eprintln!("Failed to mount file system: {}", err),
    }

    Ok(())
}