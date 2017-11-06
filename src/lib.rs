#[macro_use]
extern crate combine;

use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

mod parser;

pub fn run() -> Result<(), Box<Error>> {
    let file = find_queue_file()?;
    let items = get_queue_items(&file)?;
    for (item,_) in items.iter().zip(0..2) {
        if let Err(e) = process_item(&item) {
            eprintln!("Error while processing item: {:?}.", e);
        } else if let Err(e) = remove_from_queue(&item, &file) {
            eprintln!("Error while removing item from queue: {:?}.", e);
        }
    }
    Ok(())
}

fn process_item(item: &DownloadItem)-> io::Result<()> {
    let path = &item.path;
    println!("Downloading {} from {}", path.to_string_lossy(), item.uri);
    let parent = path.parent();
    if let Some(dir) = parent {
        if !dir.exists() {
            let _ = fs::DirBuilder::new().recursive(true).create(&dir)?;
        }
    } else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Can't create output directory."))
    }
    let mut child = Command::new("curl")
        .arg("--progress-bar")
        .arg("--location") // follow redirects
        .arg("-C").arg("-")
        .arg("-o").arg(path)
        .arg(&item.uri)
        .spawn()?;
    let ecode = child.wait()?;
    if ecode.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Downloader returned failure"))
    }
}

fn remove_from_queue(item: &DownloadItem, path: &PathBuf)-> io::Result<()> {
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    let lines =  content.lines()
        .filter(|line| {
            DownloadItem::new(line)
                .map(|fileitem| fileitem.uri != item.uri)
                .unwrap_or(true)
        });
    let lines: Vec<&str> = lines.collect();
    let content = lines.join("\n");
    let tmp_file_path = path.with_file_name("queue.temp");
    let mut tmp_file = File::create(&tmp_file_path)?;
    tmp_file.write_all(content.as_bytes())?;
    tmp_file.write_all("\n".as_bytes())?;
    let _ = tmp_file.sync_data();
    let _ = fs::rename(tmp_file_path, path)?;
    Ok(())
}

struct DownloadItem {
    uri: String,
    path: PathBuf,
}

impl DownloadItem {
    fn new(line: &str)-> Option<DownloadItem> {
        if let Some(items) = parser::tokenize_string(line) {
            if items.len() >= 2 {
                let uri = &items[0];
                let out = &items[1];
                let path = Path::new(out);
                Some(DownloadItem {
                    uri: uri.to_string(),
                    path: path.to_owned(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn get_queue_items(file: &PathBuf) -> Result<Vec<DownloadItem>, Box<Error>> {
    let file = File::open(file)?;
    let mut reader = io::BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content.lines().filter_map(|line| DownloadItem::new(line)).collect())
}

fn find_queue_file() -> Result<PathBuf, &'static str> {
    let home: String = match env::var("HOME") {
        Ok(path) => path,
        Err(_) => return Err("Can't find user home directory.")
    };
    let data_home = Path::new(&home).join(".local").join("share");
    let newsboat_queue = data_home.join("newsboat").join("queue");
    if newsboat_queue.is_file() {
        return Ok(newsboat_queue);
    }
    let newsbeuter_queue = data_home.join("newsbeuter").join("queue");
    if newsbeuter_queue.is_file() {
        return Ok(newsbeuter_queue);
    }
    return Err("No podcast download queue found.");
}
