use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, Party, Timestamps},
};
use std::{
    error::Error,
    sync::mpsc::{Sender, channel},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

// runs on its own thread so a slow discord never blocks the window
pub struct Discord {
    stop: Option<Sender<()>>,
}

impl Discord {
    pub fn new(app_id: &str) -> Result<Self, Box<dyn Error>> {
        if app_id.is_empty() || app_id == "0" {
            return Err("set app_id in src/config.toml".into());
        }

        let app_id = app_id.to_string();
        let (stop, wait) = channel();

        thread::spawn(move || {
            let mut client = DiscordIpcClient::new(&app_id);
            if client.connect().is_err() {
                return;
            }

            let started = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            let activity = Activity::new()
                .details("In the main menu")
                .state("Single Player")
                .party(Party::new().size([1, 1]))
                .timestamps(Timestamps::new().start(started));
            if client.set_activity(activity).is_err() {
                return;
            }
            println!(">> discord rpc ready!");

            let _ = wait.recv();
            let _ = client.clear_activity();
        });

        Ok(Self { stop: Some(stop) })
    }

    pub fn clear(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        Ok(())
    }
}
