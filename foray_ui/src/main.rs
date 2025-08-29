use clap::Parser;
use env_logger::Env;
use foray_ui::{
    app::{subscriptions, theme, title, App},
    headless::run_headless,
};
use iced::{application, Font, Task};
use std::{error::Error, fs, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// network file
    network: Option<PathBuf>,
    /// Run the supplied network file without opening the graphical interface
    #[arg(long)]
    no_gui: bool,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("foray=warn")).init();

    let cli = Cli::parse();
    let absolute_network = cli
        .network
        .map(|p| fs::canonicalize(&p).unwrap_or_else(|_| panic!("network does not exist {p:?}")));

    let workspace_dir = match &absolute_network {
        Some(n) => n.parent().unwrap().parent().unwrap().to_owned(),
        None => std::env::current_dir().expect("current working directory should be availble"),
    };

    if cli.no_gui {
        match absolute_network {
            Some(network) => run_headless(network),
            None => {
                println!("No network file provided");
                Ok(())
            }
        }
    } else {
        application(title, App::update, App::view)
            .subscription(subscriptions)
            .theme(theme)
            .window(iced::window::Settings {
                min_size: Some((400., 300.).into()),
                ..Default::default()
            })
            .antialiasing(true)
            .window_size((1000., 800.))
            .decorations(true)
            .scale_factor(|_| 1.25)
            .font(include_bytes!("../data/CaskaydiaCoveNerdFont.ttf").as_slice())
            .font(include_bytes!("../data/CaskaydiaCove.ttf").as_slice())
            .font(include_bytes!("../data/cour.ttf").as_slice())
            .default_font(Font::with_name("CaskaydiaCove"))
            .run_with(|| (App::new(workspace_dir, absolute_network), Task::none()))?;
        Ok(())
    }
}
