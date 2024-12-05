//! TODO: implement a file download manager using async/await
//!
//! It should go through a list of links and download them all to a specified directory as fast
//! as possible, while periodically displaying progress and download speed.
//! Everything should happen on a single thread, it is not needed to create more threads manually.
//!
//! There are two test files with URLs that you can use for testing:
//! - `links-small.txt`
//! - `links-medium.txt`
//!
//! Start with a simple solution, and then incrementally make it better:
//! 1) Download links one-by-one.
//! For each link, first download the whole file to memory, then write it to disk.
//! Print progress in-between file downloads.
//! Use [`reqwest::Client`] for downloading the links. A simple GET request should be enough.
//!
//! 2) Download links one-by-one.
//! For each link, overlap the network download with writing the file to disk.
//! Create two futures:
//! - One will download the file using HTTP chunking (see [`reqwest::Response::chunk`]
//!   or [`reqwest::Response::bytes_stream`])
//! - The second will concurrently write the chunks to the destination file.
//!
//! Connect the two futures using a Tokio MPSC channel.
//!
//! Wait until both futures are completed. Remember, they should run concurrently!
//! You can use e.g. [`tokio::join`] or [`futures::future::join`] for this.
//!
//! 2a) Add periodic output (every 500ms) that will show the download progress (in %) and the
//!   download speed (in MiB/s, see [`humansize::format_size`] and [`humansize::BINARY`]).
//!   You can use [`tokio::select`] to overlap the periodic print with the network download.
//!   When using futures inside [`tokio::select`] branches, you might need to pin them using
//!   [`Box::pin`] (on the heap) or [`std::pin::pin`] (on the stack).
//!
//! 2b) Add disk I/O speed to the periodic output. This means that you will have to perform the
//!   output outside the two (network and disk) futures, and share state between them.
//!
//! 3) Download the files concurrently. You can simply spawn each download using
//! [`tokio::task::spawn_local`] and download everything at once.
//!
//! 4) Download the files concurrently, but only N files at once, to avoid overloading the
//! network interface/disk.
//! You can use e.g. [`tokio::task::JoinSet`] to execute N futures concurrently, periodically
//! read results of resolved futures, and add new futures.

use futures::StreamExt;
use humansize::BINARY;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::task::LocalSet;
use tokio::time::Instant;
use url::Url;

struct DownloadEntry {
    url: Url,
    file_name: String,
}

fn main() -> anyhow::Result<()> {
    let links: Vec<DownloadEntry> = std::fs::read_to_string("links-small.txt")?
        .lines()
        .map(|s| {
            let url = Url::parse(s)?;
            let file_name = url.path_segments().unwrap().last().unwrap().to_string();
            Ok(DownloadEntry { url, file_name })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let dest = PathBuf::from("downloads");
    if dest.is_dir() {
        std::fs::remove_dir_all(&dest)?;
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let start = Instant::now();
    runtime.block_on(async move {
        let localset = LocalSet::new();
        localset.run_until(download_files(links, dest)).await
    })?;
    println!("Duration: {:.2}s", start.elapsed().as_secs_f64());

    Ok(())
}

/*
/// Download by one-by-one, downloading the whole file to memory, then write it to disk.
async fn download_files(links: Vec<DownloadEntry>, dest: PathBuf) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&dest).await?;

    let client = reqwest::Client::new();
    for link in links {
        let response = client.get(link.url).send().await?;

        println!(
            "Downloading: {} ({})",
            link.file_name,
            humansize::format_size(response.content_length().unwrap_or(0), BINARY)
        );
        let data = response.bytes().await?;
        tokio::fs::write(dest.join(link.file_name), data).await?;
    }

    Ok(())
}
*/

/*
/// Download by one-by-one, overlapping the network download with writing the file to disk.
async fn download_files(links: Vec<DownloadEntry>, dest: PathBuf) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&dest).await?;

    let client = reqwest::Client::new();
    for link in links {
        let response = client.get(link.url).send().await?;

        println!(
            "Downloading: {} ({})",
            link.file_name,
            humansize::format_size(response.content_length().unwrap_or(0), BINARY)
        );

        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let network_downloader = async move {
            let mut stream = response.bytes_stream();
            while let Some(Ok(chunk)) = stream.next().await {
                tx.send(chunk).await.unwrap_or_default();
            }
        };
        let dest = dest.join(link.file_name);
        let disk_writer = async move {
            let mut file = tokio::fs::File::create(dest).await?;
            while let Some(chunk) = rx.recv().await {
                file.write_all(&chunk).await?;
            }
            Ok::<(), anyhow::Error>(())
        };

        // Wait until both futures are completed
        let (_, res2) = tokio::join!(network_downloader, disk_writer);
        res2?;
    }

    Ok(())
}
*/

async fn download_files(links: Vec<DownloadEntry>, dest: PathBuf) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&dest).await?;

    let client = reqwest::Client::new();
    for link in links {
        let response = client.get(link.url).send().await?;

        let size = response.content_length().unwrap_or(0);
        println!(
            "Downloading: {} ({})",
            link.file_name,
            humansize::format_size(response.content_length().unwrap_or(0), BINARY)
        );

        let byte_counter = Rc::new(RefCell::new(0));
        let byte_counter2 = byte_counter.clone();

        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let network_downloader = async move {
            let mut stream = response.bytes_stream();
            while let Some(Ok(chunk)) = stream.next().await {
                *byte_counter.borrow_mut() += chunk.len() as u64;
                tx.send(chunk).await.unwrap_or_default();
            }
        };
        let dest = dest.join(link.file_name);
        let disk_writer = async move {
            let mut file = tokio::fs::File::create(dest).await?;
            while let Some(chunk) = rx.recv().await {
                file.write_all(&chunk).await?;
            }
            Ok::<(), anyhow::Error>(())
        };

        let mut download_fut =
            std::pin::pin!(futures::future::join(network_downloader, disk_writer));

        loop {
            tokio::select! {
                _ = &mut download_fut => {
                    break;
                },
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    println!("Progress: {}/{size}", byte_counter2.borrow());
                }
            }
        }
    }

    Ok(())
}
