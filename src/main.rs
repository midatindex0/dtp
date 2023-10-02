use clap::Parser;
use colored::Colorize;
use packet::S2C;
use spinners::{Spinner, Spinners};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex}, process::exit, time::Duration,
};
use tungstenite::{connect, Message};
use vlc::{Event, EventType, Instance, Media, MediaPlayer, State};
use youtube_dl::YoutubeDl;

mod packet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, value_name = "USERNAME")]
    id: String,
}

const DTP_SERVER: &str = "wss://dtp-server.shuttleapp.rs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let url = url::Url::parse(&format!("{}/{}", DTP_SERVER, cli.id)).unwrap();
    let mut sp = Spinner::new(
        Spinners::Dots12,
        format!(
            "[{}] Connecting to DTP server at {} with ID: {}",
            "INFO".bold().blue(),
            DTP_SERVER.bold(),
            cli.id.bold()
        ),
    );
    let con = connect(url);
    if con.is_err() {
        sp.stop_with_message(format!("[{}] Failed to connect", "ERROR".bold().red()));
        exit(1);
    }
    let (socket, _) = con.unwrap();
    let socket = Arc::new(Mutex::new(socket));

    sp.stop_with_message(format!(
        "[{}] Connected to DTP server",
        "INFO".bold().blue()
    ));

    let instance = Instance::new().unwrap();
    let mdp = MediaPlayer::new(&instance).unwrap();

    socket
        .lock()
        .unwrap()
        .send(Message::Text("\"Ready1\"".into()))
        .unwrap();
    loop {
        std::thread::sleep(Duration::from_millis(10));
        if socket.lock().unwrap().can_read() {
            let mut l = socket.lock().unwrap();
            let msg = l.read().expect("Error reading message");
            drop(l);
            match msg {
                Message::Text(t) => {
                    let msg = serde_json::from_str::<S2C>(&t).unwrap();
                    match msg {
                        S2C::ConnectNotification(notif) => {
                            if notif.id != cli.id {
                                println!(
                                    "[{}] Client with ID: {} joined",
                                    "INFO".bold().blue(),
                                    notif.id
                                )
                            }
                        }
                        S2C::DisconnectNotification(notif) => {
                            println!(
                                "[{}] Client with ID: {} disconnected",
                                "INFO".bold().blue(),
                                notif.id
                            )
                        }
                        S2C::Play { yt_link } => {
                            let mut sp = Spinner::new(
                                Spinners::Dots12,
                                format!(
                                    "[{}] Downloading audio: {}",
                                    "INFO".bold().blue(),
                                    yt_link.bold(),
                                ),
                            );
                            let m = socket.clone();
                            std::thread::spawn(move || {
                                let _ = std::fs::remove_file("./music/out.m4a");
                                let r = YoutubeDl::new(yt_link)
                                    .format("140")
                                    .output_template("out.m4a")
                                    .download_to("./music");

                                if r.is_err() {
                                    sp.stop_with_message(format!(
                                        "[{}] Failed to download audio",
                                        "ERROR".bold().red()
                                    ));
                                } else {
                                    sp.stop_with_message(format!(
                                        "[{}] Downloaded audio. Waiting for other clients",
                                        "SUCCESS".green().bold()
                                    ));
                                    m.lock()
                                        .unwrap()
                                        .send(Message::Text("\"Ready2\"".into()))
                                        .unwrap();
                                }
                            });
                        }
                        S2C::Start => {
                            let md = Media::new_path(&instance, "./music/out.m4a");
                            if let Some(md) = md {
                                mdp.set_media(&md);

                                let em = md.event_manager();
                                let m = socket.clone();
                                let _ =
                                    em.attach(EventType::MediaStateChanged, move |e, _| match e {
                                        Event::MediaStateChanged(s) => match s {
                                            State::Playing => {
                                                println!(
                                                    "[{}] Playing media",
                                                    "INFO".blue().bold()
                                                );
                                                m.lock()
                                                    .unwrap()
                                                    .send(Message::Text("\"Playing\"".into()))
                                                    .unwrap();
                                            }
                                            State::Stopped => {
                                                println!(
                                                    "[{}] Current media stopped/ended",
                                                    "INFO".blue().bold()
                                                );
                                                m.lock()
                                                    .unwrap()
                                                    .send(Message::Text("\"Ready1\"".into()))
                                                    .unwrap();
                                            }
                                            State::Error => {
                                                eprintln!(
                                                    "[{}] Error while playing media",
                                                    "ERROR".red().bold()
                                                );
                                            }
                                            _ => {}
                                        },
                                        _ => (),
                                    });
                                let _ = mdp.play();
                            }
                        }
                        S2C::Skip => {
                            println!("[{}] Skipping media", "INFO".blue().bold());
                            mdp.stop();
                        }
                    }
                }
                Message::Ping(b) => {
                    socket.lock().unwrap().send(Message::Pong(b)).unwrap();
                }
                Message::Pong(_) => {
                    println!("Pong");
                }
                Message::Close(_) => todo!(),
                _ => {
                    unreachable!()
                }
            }
        }
    }
}
