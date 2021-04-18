use std;
use std::io;
use std::fs;

use dialoguer::Select;
use dialoguer::Input;

use ron;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MinecraftServerConfiguration {
	bonusChest: bool,
	demo: bool,
	eraseCache: bool,
	forceUpgrade: bool,
	initSettings: bool,
	gui: bool,
	port: Option<u32>,
	safeMode: bool,
	serverId: Option<String>,
	singleplayer: bool,
	universe: Option<String>,
	world: Option<String>,
}

#[derive(Debug)]
struct ConfigurationOption<'a> {
	property: &'a str,
	name: &'a str,
	description: &'a str,
	r#type: &'a str,
}

fn get_configuration_path() -> String {
	let path_arg: Option<String> = std::env::args().nth(1);

	if let Some(path) = path_arg {
		return path;
	}

	if cfg!(windows) {
		let appdata = std::env::var("APPDATA").expect("No path to the Minecraft server folder was specified and the APPDATA environment variable does not exist.");
		return format!("{}\\.minecraft\\server\\", appdata)
	} else {
		return "~/.minecraft/server/".to_string();
	}
}

fn read_configuration() -> Option<MinecraftServerConfiguration> {
	let result = fs::read_to_string(get_configuration_path() + "msc-configuration.ron");
	if let Ok(configuration_string) = result {
		return match ron::from_str::<MinecraftServerConfiguration>(&configuration_string) {
			Ok(configuration) => Some(configuration),
			Err(error) => {
				// TODO: log the error
				return None;
			},
		};
	} else {
		return None;
	}
}

fn main() {
	let configuration = read_configuration().unwrap_or(MinecraftServerConfiguration {
		bonusChest: true,
		demo: false,
		eraseCache: false,
		forceUpgrade: false,
		initSettings: false,
		gui: false,
		port: None,
		safeMode: false,
		serverId: None,
		singleplayer: false,
		universe: None,
		world: None,
	});

	let configuration_options_information = [
		ConfigurationOption {
			property: "bonusChest",
			name: "Bonus chest",
			description: "Whether or not to add the bonus chest when creating a new world.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "demo",
			name: "Demo mode",
			description: "Shows the players a demo pop-up, players can't place/break/eat once the demo expires.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "eraseCache",
			name: "Erase the cache",
			description: "Erases the lighting caches, etc.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "forceUpgrade",
			name: "Force an upgrade",
			description: "Forces an upgrade on all the chunks.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "initSettings",
			name: "Initialize server settings",
			description: "Initializes 'server.properties' and 'eula.txt', then quits.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "gui",
			name: "GUI mode",
			description: "When enabled, opens the GUI upon launch of the server.",
			r#type: "bool",
		},
		ConfigurationOption {
			property: "port",
			name: "Port",
			description: "Which port to listen on, overrides the server.properties value.",
			r#type: "number",
		},
	];

	Select::new()
		.with_prompt("Please select the values you wish to change")
		.items(&[1]);
}
