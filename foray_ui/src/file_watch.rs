use std::ffi::OsStr;
use std::path::Component;
use std::path::PathBuf;
use std::time::Duration;

use futures::StreamExt;
use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream;
use iced::Subscription;
use log::trace;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;
use notify_debouncer_full::DebounceEventResult;

fn worker(root_dir: &PathBuf) -> impl Stream<Item = Vec<PathBuf>> {
    let root_dir = root_dir.clone();
    stream::channel(0, |mut output: mpsc::Sender<_>| async move {
        trace!("Starting file watch subscription stream: {root_dir:?}");

        let (mut tx, mut rx) = iced::futures::channel::mpsc::channel(1);
        let mut debouncer = new_debouncer(
            Duration::from_millis(250),
            None,
            move |result: DebounceEventResult| {
                iced::futures::executor::block_on(async {
                    if let Err(e) = tx.send(result).await {
                        println!("Error sending event result: {:?}", e);
                    }
                });
            },
        )
        .unwrap_or_else(|err| panic!("Failed to watch directory {root_dir:?}:\n{err}"));

        let _ = debouncer.watch(root_dir, RecursiveMode::Recursive);

        while let Some(res) = rx.next().await {
            match res {
                Ok(events) => {
                    let nodes: Vec<_> = events
                        .into_iter()
                        .map(|debounce_event| debounce_event.event)
                        .filter(|e| {
                            (e.kind.is_modify() || e.kind.is_create())
                                && e.paths.iter().any(|p| {
                                    p.extension() == Some(OsStr::new("py"))
                                        && !p
                                            .components()
                                            .any(|s| s == Component::Normal(OsStr::new(".venv")))
                                    //TODO: more reliable check here?
                                })
                        })
                        .flat_map(|event| event.paths)
                        .collect();

                    if !nodes.is_empty() {
                        let _ = output.send(nodes).await;
                    }
                }
                Err(error) => log::error!("Receive Error: {error:?}"),
            }
        }
    })
}
pub fn make_file_watch_sub(root_dir: PathBuf) -> Subscription<Vec<PathBuf>> {
    iced::Subscription::run_with(root_dir, |r| worker(r))
}
