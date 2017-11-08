#[macro_use]
extern crate combine;
#[macro_use]
extern crate error_chain;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate xdg;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use structopt::StructOpt;

mod args;
mod parser;

pub fn run() -> Result<(), Box<Error>> {
    let opt: args::Opt = args::Opt::from_args();
    let file = find_queue_file()?;
    let items = get_queue_items(&file)?;
    for (item,_) in items.iter().zip(0..opt.num) {
        if let Err(e) = process_item(&item, opt.progress) {
            eprintln!("Error while processing item: {}.", e.description());
        } else if let Err(e) = remove_from_queue(&item, &file) {
            eprintln!("Error while removing item from queue: {:?}.", e);
        }
    }
    Ok(())
}

fn process_item(item: &DownloadItem, show_progress: bool)-> Result<(), Box<Error>> {
    let path = &item.path;
    println!("Downloading {} from {}", path.to_string_lossy(), item.uri);
    let parent = path.parent();
    if let Some(dir) = parent {
        if !dir.exists() {
            let _ = fs::DirBuilder::new().recursive(true).create(&dir)?;
        }
    } else {
        bail!("Can't create output directory.")
    }
    let mut child = Command::new("curl")
        .arg(if show_progress { "--progress-bar" } else { "--silent" })
        .arg("--location") // follow redirects
        .arg("-C").arg("-")
        .arg("-o").arg(path)
        .arg(&item.uri)
        .spawn()?;
    let ecode = child.wait()?;
    if ecode.success() {
        Ok(())
    } else {
        bail!("Downloader returned failure")
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
        parser::tokenize_string(line).and_then(|items| {
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
        })
    }
}

fn get_queue_items(file: &PathBuf) -> Result<Vec<DownloadItem>, Box<Error>> {
    let file = File::open(file)?;
    let mut reader = io::BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(content.lines().filter_map(|line| DownloadItem::new(line)).collect())
}

fn find_queue_file() -> Result<PathBuf, Box<Error>> {
    let boat_path = xdg::BaseDirectories::with_prefix("newsboat")?;
    let beuter_path = xdg::BaseDirectories::with_prefix("newsbeuter")?;
    boat_path.find_data_file("queue").or_else(| | {
        beuter_path.find_data_file("queue")
    }).ok_or( "No podcast download queue found.".into())
}
