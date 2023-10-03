use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum S2C {
    ConnectNotification(ConnectNotification),
    DisconnectNotification(DisconnectNotification),
    Play { yt_link: String },
    Start,
    Skip,
}

#[derive(Deserialize, Debug)]
pub struct ConnectNotification {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct DisconnectNotification {
    pub id: String,
}
