use std::{path::PathBuf, process::exit, time::Duration};

use clap::Parser;

mod args;
use args::{Args, DEFAULT_PATH};

mod file_list;
use file_list::FileList;

mod file_monitor;
use file_monitor::{ls_inbox, monitor_inbox};

async fn get_inbox() -> std::io::Result<PathBuf> {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    println!("Directory string: {}", args.directory);

    // Resolving "~" as $HOME can be problematic
    let path = if args.directory.starts_with("~") {
        let mut path = home::home_dir().expect("Failed to determine local home directory");
        for elem in DEFAULT_PATH.split("/").skip(1) {
            path.push(elem);
        }
        path
    } else {
        PathBuf::from(&args.directory)
    };

    #[cfg(debug_assertions)]
    println!(
        "Directory (canonicalized) {:?} exists: {:?}",
        tokio::fs::canonicalize(&path).await,
        tokio::fs::try_exists(&path).await
    );

    if !tokio::fs::try_exists(&path).await? {
        tokio::fs::create_dir(&path).await?;
        println!("Successfully created directory {:?}", path);
    }

    #[cfg(debug_assertions)]
    println!(
        "Directory (canonicalized) {:?} exists: {:?}",
        tokio::fs::canonicalize(&path).await,
        tokio::fs::try_exists(&path).await
    );

    tokio::fs::canonicalize(&path).await
}

/// Main entry point
///  - Lists inbox,
///  - recursively monitors for [New/Mod/Del] under inbox and prints the events,
///  - lists last compiled list of files w/o reading inbox again
/// TODO: future improvement is error typing/handling using thiserror or snafu - replace all
///  expect()'s and unwrap()'s w/ proper handling
/// TODO: allow modification of polling timer
/// TODO: explore using notify crate and not handling manual polling (using tokio due to SOW)
/// TODO: unit tests - possibly create a sim-Inbox that can perform scripted operations
/// TODO: proper logging/tracing/(perf) monitoring
#[tokio::main]
async fn main() {
    let inbox = get_inbox().await.expect("Error getting inbox");

    #[cfg(debug_assertions)]
    println!("List inbox contents (no dirs):");

    let mut fl = FileList::new(&inbox);
    let entries = fl
        .get_inbox_entries(true)
        .await
        .expect("Error getting inbox file entries");

    ls_inbox(&entries, false)
        .await
        .expect("Error listing contents of inbox");

    let entries = monitor_inbox(&inbox, Duration::from_secs(1))
        .await
        .expect("Error monitoring contents of inbox");

    ls_inbox(&entries, false)
        .await
        .expect("Error listing contents of inbox");
}
