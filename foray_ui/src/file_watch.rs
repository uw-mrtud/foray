use std::ffi::OsStr;
use std::path::Component;
use std::path::PathBuf;
use std::time::Duration;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::Stream;
use iced::stream;
use iced::Subscription;
use log::trace;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

fn _worker(root_dir: &PathBuf) -> impl Stream<Item = usize> {
    dbg!("running worker");
    let root_dir = root_dir.clone();
    // let root_dir = PathBuf::from("/home/john/projects/foray/book/");

    //// Does not work well after updating to ice 0.14
    //// Maybe an issue that receiver is blocking? if it were async maybe that could fix things?
    stream::channel(0, |mut output: mpsc::Sender<_>| async move {
        dbg!("initializing stream");
        trace!("Starting file watch subscription stream: {root_dir:?}");
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(250), None, sender).unwrap();
        debouncer.watch(root_dir, RecursiveMode::Recursive).unwrap();

        // std::thread::spawn(move || {
        loop {
            if dbg!(output.is_closed()) {
                break;
            }
            dbg!("about to receive");
            match dbg!(receiver.try_recv()) {
                Ok(Ok(events)) => {
                    // dbg!(&events);
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
                        .collect();
                    dbg!(&nodes);
                    if !nodes.is_empty() {
                        dbg!("sending");
                        let _ = output.send(1).await;
                        dbg!("sent");
                    }
                }
                Ok(Err(error)) => log::error!("Error: {error:?}"),
                Err(error) => log::error!("Receive Error: {error:?}"),
            }
        }
        // })
        // .join();
    })
}
pub fn make_file_watch_sub(_root_dir: PathBuf) -> Subscription<usize> {
    // turn off file watch till I fix this...
    return Subscription::none().map(|_: ()| 1);
    // iced::Subscription::run_with(root_dir, |r| worker(r))
}
