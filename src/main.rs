mod app;
mod client;
mod color;
mod config;
mod constant;
mod data;
mod error;
mod event;
mod help;
mod util;
mod view;
mod widget;

use clap::Parser;

use crate::{app::App, client::Client, color::ColorTheme, config::Config, event::UserEventMapper};

/// DDV - Terminal DynamoDB Viewer ⚡️
#[derive(Parser)]
#[command(version)]
struct Args {
    /// AWS region
    #[arg(short, long)]
    region: Option<String>,

    /// AWS endpoint url
    #[arg(short, long, value_name = "URL")]
    endpoint_url: Option<String>,

    /// AWS profile name
    #[arg(short, long, value_name = "NAME")]
    profile: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config = Config::load();
    let theme = ColorTheme::default();
    let mapper = UserEventMapper::new();

    let client = Client::new(args.region, args.endpoint_url, args.profile).await;
    let (tx, rx) = event::init();

    tx.send(event::AppEvent::Initialize);

    let mut terminal = ratatui::init();

    let mut app = App::new(config, theme, mapper, client, tx);
    let ret = app.run(&mut terminal, rx);

    ratatui::restore();
    ret
}
