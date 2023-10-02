use serde::Deserialize;

#[derive(Deserialize)]
pub enum S2C {
    ConnectNotification(ConnectNotification),
    DisconnectNotification(DisconnectNotification),
    Play { yt_link: String },
    Start,
    Skip,
}

#[derive(Deserialize)]
pub struct ConnectNotification {
    pub id: String,
}

#[derive(Deserialize)]
pub struct DisconnectNotification {
    pub id: String,
}
