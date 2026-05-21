use std::{thread::sleep, time::Duration};

use pawpad::PawPad;

const MAPPINGS_FILE: &str = include_str!("pawpad_mappings.json");

fn main() -> anyhow::Result<()> {
    let mappings = serde_json::from_str(MAPPINGS_FILE)?;

    let mut pawpad = PawPad::new(&mappings)?;

    loop {
        for event in pawpad.poll_events()? {
            println!("{:?}", event.kind);
        }
        sleep(Duration::from_secs(1));
    }
}
