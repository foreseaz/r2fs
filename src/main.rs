use r2client::R2Client;
use std::error::Error;
use std::{env, process};
use dotenv::dotenv;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};

mod r2client;

const TTL: Duration = Duration::from_secs(1); // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
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

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

struct R2FS {
    r2_client: R2Client,
    bucket: String,
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
        if parent == 1 && name.to_str() == Some("hello.txt") {
            reply.entry(&TTL, &HELLO_TXT_ATTR, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("[getattr] calling");
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
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
        println!("[read] calling");
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
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
        entries.push((1, FileType::Directory, "."));
        entries.push((1, FileType::Directory, ".."));

        // Retrieve the list of objects in the bucket
        let objects = self.r2_client.list_bucket_objects(&self.bucket).unwrap();

        // Add the objects as directory entries
        for (i, obj) in objects.contents.iter().enumerate().skip(offset as usize) {
            // TODO: need handle the directory
            if obj.key.contains("/") {
                continue;
            }
            entries.push((i + 2, FileType::RegularFile, &obj.key));
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

    dotenv().ok(); // load the .env file
    let cf_account_id = env::var("ACCOUNT_ID").unwrap();
    let r2_access_key_id = env::var("R2_ACCESS_KEY_ID").unwrap();
    let r2_secret_access_key = env::var("R2_SECRET_ACCESS_KEY").unwrap();

    let mut fs = R2FS {
        r2_client: R2Client::new(cf_account_id, r2_access_key_id, r2_secret_access_key),
        bucket: String::new(),
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

    let list_buckets_res = fs.r2_client.list_buckets()?;
    fs.bucket = list_buckets_res.buckets.bucket[0].name.clone();
    println!("\t\tlist_buckets_parser result{:#?}", fs.bucket);

    // List bucket objects test
    let objects = fs.r2_client.list_bucket_objects(&fs.bucket);
    // println!("\t\tobjects result{:#?}", objects);

    // Set up the mount options
    let mut mount_options = Vec::new();
    let mount_path = mountpoint.clone().into_string().unwrap() + "/" + &fs.bucket;

    mount_options.push(MountOption::RW);
    mount_options.push(MountOption::FSName(fs.bucket.clone()));

    println!("[DEBUG] will mount at {:?}", mount_path);

    // Unmount existing FUSE mount if necessary
    if let Err(err) = unmount(&mount_path) {
        eprintln!("Failed to unmount existing FUSE mount: {}", err);
        return Ok(());
    }

    // Mount the file system
    match fuser::mount2(fs, &mount_path, &mount_options) {
        Ok(_) => println!("File system mounted successfully"),
        Err(err) => eprintln!("Failed to mount file system: {}", err),
    }

    Ok(())
}