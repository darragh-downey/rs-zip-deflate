use reqwest;
use tempfile;
use zip;
use std::path::Path;
use std::fs::{create_dir_all, File};
use std::ffi::OsStr;

use futures_util::StreamExt;
use std::io::{Read,Write};

const URI: &str = "https://github.com/CISecurity/OVALRepo/archive/refs/heads/master.zip";
const DEST: &str = "data/";


pub async fn fetch() -> Result<(), String> {
    let mut tmp = tempfile::tempfile().or(Err(format!("failed to create tempfile")))?;
    let res = reqwest::get(URI)
        .await
        .or(Err(format!("failed to download zip file")))?;
    let mut stream = res
        .bytes_stream();

    while let Some(next) = stream.next().await {
        let chunk = next
            .or(Err(format!("issue grabbing chunk")))?;
        tmp
            .write_all(&chunk)
            .or(Err(format!("issue writing to tmp file")))?;
    }

    deflate(tmp).await?;
    Ok(())
}

async fn deflate(tmp: File) -> Result<(), String> {
    let mut zip = zip::ZipArchive::new(tmp)
        .or(Err(format!("failed to open zip archive")))?;
    let local_path = Path::new(DEST);

    create_dir_all(local_path)
        .or(Err(format!("failed to create local directories {}", DEST)))?;

    for i in 0..zip.len() {
        let file = zip
            .by_index(i)
            .or(Err(format!("failed to get file by its index in zip archive")))?;

        let name = Path::new(file.name());

        // only interested in xml and xsd
        if name.is_dir() {
            println!("skipping directory {}", name.display());
            continue;
        }

        let ext = name.extension()
            .and_then(OsStr::to_str);
        if ext != Some("xml") {
            println!("skipping {} as not xml file", name.display());
            continue;
        }

        let n = match name.file_name().and_then(OsStr::to_str) {
            Some(n) => n,
            _ => {
                println!("appears to not be a file - should skip ");
                continue;
            },
        };

        println!("continuing to process {}", name.display());

        let dest_path = local_path.join(n);

        // create file
        let mut f = File::create(dest_path)
            .or(Err(format!("failed to create the local file {}", file.name())))?;

        println!("created {} at {}", file.name(), local_path.display());

        let mut buf: Vec<u8> = vec![];
        let mut stream = file.bytes();

        while let Some(next) = stream.next() {
            // get chunk
            let chunk = next
                .or(Err(format!("issue grabbing chunk")))?;

            // check that our buffer hasn't grown too large
            if buf.len() >= 512 {
                // write buffer to file
                f.write_all(&buf)
                    .or(Err(format!("failed to write the data from zip archive to local file")))?;
                // clear buffer
                buf.clear();
            }
            // push data to buffer
            buf.push(chunk);
        }
        f.write_all(&buf)
            .or(Err(format!("failed to flush buffer to file")))?;
    }
    Ok(())
}
