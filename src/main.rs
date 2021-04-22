mod machine;
use machine::*;

use std::{self, collections::HashMap};
use std::fs;

use dialoguer::Select;
use dialoguer::Confirm;
use dialoguer::Input;

use ron;
use enumflags2::make_bitflags;
use statechart::*;
use strum_macros::Display;

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

fn get_configuration() -> MinecraftServerConfiguration {
	let default_configuration = MinecraftServerConfiguration {
		bonusChest: true,
		demo: false,
		eraseCache: false,
		forceUpgrade: false,
		initSettings: false,
		gui: false,
		port: None,
		safeMode: false,
		singleplayer: false,
		universe: None,
		world: None,
	};

	let result = fs::read_to_string(get_configuration_path() + "msc-configuration.ron");
	if let Ok(configuration_string) = result {
		if let Ok(configuration) = ron::from_str::<MinecraftServerConfiguration>(&configuration_string) {
			return configuration;
		} else {
			// TODO: Log error: Unable to parse "msc-configuration.ron", falling back to default configuration.
		}
	} else {
		// TODO: Log error: Unable to read "msc-configuration.ron", falling back to default configuration.
	}

	return default_configuration;
}

fn get_config_option_info() -> Vec<ConfigurationOption> {
	vec![
		ConfigurationOption {
			property: "bonusChest".to_string(),
			name: "Bonus chest".to_string(),
			description: "Whether or not to add the bonus chest when creating a new world.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "demo".to_string(),
			name: "Demo mode".to_string(),
			description: "Shows the players a demo pop-up, players can't place/break/eat once the demo expires.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "eraseCache".to_string(),
			name: "Erase the cache".to_string(),
			description: "Erases the lighting caches, etc.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "forceUpgrade".to_string(),
			name: "Force an upgrade".to_string(),
			description: "Forces an upgrade on all the chunks.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "initSettings".to_string(),
			name: "Initialize server settings".to_string(),
			description: "Initializes 'server.properties' and 'eula.txt', then quits.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "gui".to_string(),
			name: "GUI mode".to_string(),
			description: "When enabled, opens the GUI upon launch of the server.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "port".to_string(),
			name: "Port".to_string(),
			description: "Which port to listen on, overrides the server.properties value.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Option | U16}),
		},
		ConfigurationOption {
			property: "safeMode".to_string(),
			name: "Safe mode".to_string(),
			description: "Loads level with vanilla data pack only.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "singleplayer".to_string(),
			name: "Single-player mode".to_string(),
			description: "Runs the server in offline mode without authentication. This is insecure, do not use this when online.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Bool}),
		},
		ConfigurationOption {
			property: "universe".to_string(),
			name: "Universe name".to_string(),
			description: "".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Option | String}),
		},
		ConfigurationOption {
			property: "world".to_string(),
			name: "World name".to_string(),
			description: "".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Option | String}),
		}
	]
}

fn get_names(config_option_info: Vec<ConfigurationOption>) -> Vec<String> {
	let configuration_option_names: Vec<String> = config_option_info.into_iter().map(|option_information| option_information.name).collect();
	vec![
		vec!["Start server now".to_string(), "Exit".to_string()],
		configuration_option_names
	].concat()
}

fn main() {
	let mut machine = Machine {
		state: AppState::ChoiceMenu,
		editorState: None,
		selectedConfigurationOption: None,
		configuration: get_configuration(),
	};

	while machine.state != AppState::Exited {
		match machine.state.clone() {
			AppState::ChoiceMenu => {
				let select_options = get_names(get_config_option_info());
				let result = Select::new()
					.with_prompt("Please select the value you wish to change")
					.items(&select_options)
					.interact();
				if let Ok(result) = result {
					if result == 0 {
						machine.dispatch(machine::Event::AppEvent(AppEvent::StartServer), None);
					} else if result == 1 {
						machine.dispatch(machine::Event::AppEvent(AppEvent::Exit), None);
					} else {
						match get_config_option_info().get(result - 2) {
							Some(option) => {
								let payload = Some(Payload::ConfigurationOption(option.clone()));
								machine.dispatch(machine::Event::AppEvent(AppEvent::SelectedOption), payload);
							},
							None => {
								// TODO: error/warning
							}
						}
					}
					continue
				}
			}
			AppState::Running => {
				// save config
				// start minecraft server with java
				return;
			}
			AppState::Exited => {}
			AppState::EditingConfiguration => {
				let editorState = machine.editorState.clone().expect("");
				println!("{:?}", editorState);
				match editorState {
					EditorState::SelectOnOff => {}
					EditorState::NumberInput => {}
					EditorState::TextInput => {}
					EditorState::SelectValueOrNone => {}
				}
			}
		}
	}
}
