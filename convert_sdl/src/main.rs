use std::str::FromStr;

use uuid::Uuid;

const DB: &str = include_str!("../SDL_GameControllerDB/gamecontrollerdb.txt");

fn main() {
    for mapping in DB.lines().filter(|it| !it.is_empty() && !it.starts_with("#")) {
        let strings = mapping.split(",").filter(|it| !it.is_empty()).collect::<Box<[&str]>>();

        let uuid = strings[0];
        let platform = strings[strings.len() - 1];

        let name = strings[1];

        let platform_name = &platform[9..];

        let Ok(uuid) = Uuid::from_str(uuid) else {
            continue;
        };

        let bindings = &strings[2..strings.len() - 1];

        println!("{}\n{}\n{}\n{:?}\n\n", uuid, platform_name, name, bindings);
    }
}
