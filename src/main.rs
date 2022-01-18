use std::{env::set_current_dir, fs, process::Stdio};

use clap::{App, Arg};
use console::{style, Term};
use dialoguer;
use execute::command;
use ron;

mod machine;
use enumflags2::make_bitflags;
use machine::*;

fn get_default_configuration_path() -> String {
    if cfg!(windows) {
        let appdata = std::env::var("APPDATA").expect("No path to the Minecraft server folder was specified and the APPDATA environment variable does not exist.");
        return format!("{}\\.minecraft\\server\\", appdata);
    } else {
        return "~/.minecraft/server/".to_string();
    }
}

fn save_configuration(configuration: MinecraftServerConfiguration) -> MinecraftServerConfiguration {
    let configuration_ron = ron::to_string(&configuration).unwrap();
    fs::write("msc-configuration.ron", configuration_ron)
        .expect("Unable to write to msc-configuration.ron");
    configuration
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

    let result = fs::read_to_string("msc-configuration.ron");
    if let Ok(configuration_string) = result {
        if let Ok(configuration) =
            ron::from_str::<MinecraftServerConfiguration>(&configuration_string)
        {
            return configuration;
        } else {
            println!("Unable to successfully parse msc-configuration.ron, falling back to default configuration.");
        }
    } else {
        println!(
            "Unable to read the msc-configuration.ron file, falling back to default configuration."
        );
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
			description: "The folder name of the universe that you want to use that contains all of your worlds.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Option | String}),
		},
		ConfigurationOption {
			property: "world".to_string(),
			name: "World name".to_string(),
			description: "The folder name for the world you want to run.".to_string(),
			r#type: make_bitflags!(ConfigurationOptionTypeFlag::{Option | String}),
		}
	]
}

fn get_names(
    config_option_info: Vec<ConfigurationOption>,
    configuration: MinecraftServerConfiguration,
) -> Vec<String> {
    let configuration_option_names: Vec<String> = config_option_info
        .into_iter()
        .map(|option_information| {
            let value: String = match configuration.get(option_information.property) {
                ConfigurationOptionType::Bool(value) => {
                    if value {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    }
                }
                ConfigurationOptionType::OptionString(value) => {
                    value.unwrap_or("default".to_string())
                }
                ConfigurationOptionType::OptionU16(value) => match value {
                    Some(value) => value.to_string(),
                    None => "default".to_string(),
                },
            };
            format!("{} ({})", option_information.name, value)
        })
        .collect();
    vec![
        vec!["Start server now".to_string(), "Exit".to_string()],
        configuration_option_names,
    ]
    .concat()
}

fn run_server(
    configuration: MinecraftServerConfiguration,
    jar_filename: String,
    terminal: Term,
) -> (MinecraftServerConfiguration, Term) {
    let mut command_string = format!("java -jar {}", jar_filename);

    for option in get_config_option_info() {
        let value = configuration.get(option.property.clone());
        let cli_flag = match (option.property.clone().as_str(), value) {
            ("gui", ConfigurationOptionType::Bool(value)) => {
                if value {
                    None
                } else {
                    Some("--nogui".to_string())
                }
            }
            (property, ConfigurationOptionType::Bool(value)) => {
                if value {
                    Some(format!("--{}", property))
                } else {
                    None
                }
            }
            (property, ConfigurationOptionType::OptionU16(value)) => match value {
                Some(value) => Some(format!("--{} {}", property, value.to_string())),
                None => None,
            },
            (property, ConfigurationOptionType::OptionString(value)) => match value {
                Some(value) => Some(format!("--{} {}", property, value)),
                None => None,
            },
        };

        if let Some(cli_flag) = cli_flag {
            command_string = format!("{} {}", command_string, cli_flag);
        }
    }

    terminal
        .write_line(
            style("Starting your Minecraft Server")
                .green()
                .to_string()
                .as_str(),
        )
        .unwrap();
    terminal.write_line(command_string.as_str()).unwrap();

    let mut command = command(command_string);
    command.stdin(Stdio::inherit());
    command.stdout(Stdio::inherit());

    let output = command.output();
    match output {
        Ok(output) => {
            let code = output.status.code().unwrap();
            if code == 0 {
                terminal.write_line("It was a success!").unwrap();
            } else {
                terminal
                    .write_line("Something went wrong! Do you have Java installed?")
                    .unwrap();
            }
        }
        Err(error) => {
            terminal
                .write_line(style("An error ocurred!").red().to_string().as_str())
                .unwrap();
            terminal.write_line(error.to_string().as_str()).unwrap();
        }
    }
    return (configuration, terminal);
}

fn main() {
    let matches = App::new("minecraft-server-cli")
		.version("0.1.0")
    .author("Andria Brown <andria_girl@pm.me>")
    .about("A command-line interface used to edit and persist your Minecraft server command-line settings and start your Minecraft server.")
		.arg(Arg::with_name("jar_filename")
			.index(1)
			.help("The name you saved your Minecraft server .jar file under.")
			.required(true)
    	.takes_value(true)
		).arg(Arg::with_name("server_directory")
			.index(2)
			.help("The relative or absolute path to the directory containing your Minecraft server. Defaults to %AppData%\\.minecraft\\server\\ on Windows and ~/.minecraft/server/ on Unix-based systems. Please end your value with a forward slash (Unix-based /) or backslash (Windows \\).")
		).get_matches();

    let jar_filename = matches.value_of("jar_filename").unwrap().to_string();
    let server_directory = matches
        .value_of("server_directory")
        .unwrap_or(get_default_configuration_path().as_str())
        .to_string();
    set_current_dir(server_directory).expect(
        "Expected to be able to change the current directory to your Minecraft Server's directory.",
    );

    let mut machine = Machine {
        state: AppState::ChoiceMenu,
        editor_state: None,
        selected_configuration_option: None,
        configuration: get_configuration(),
    };
    let config_option_info = get_config_option_info();
    let mut terminal = Term::stdout();

    while machine.state != AppState::Exited {
        terminal
            .clear_screen()
            .expect("Expected to be able to clear the terminal.");

        match machine.state.clone() {
            AppState::ChoiceMenu => {
                terminal.set_title("Minecraft Server CLI — Choice Menu");
                let select_options =
                    get_names(config_option_info.clone(), machine.configuration.clone());
                let result = dialoguer::Select::new()
                    .with_prompt("Please select the value you wish to change")
                    .items(&select_options)
                    .interact_on_opt(&terminal)
                    .unwrap()
                    .unwrap();

                if result == 0 {
                    machine.dispatch(Event::AppEvent(AppEvent::StartServer), None);
                } else if result == 1 {
                    machine.dispatch(Event::AppEvent(AppEvent::Exit), None);
                } else {
                    match config_option_info.get(result - 2) {
                        Some(option) => {
                            let payload = Some(Payload::ConfigurationOption(option.clone()));
                            machine.dispatch(Event::AppEvent(AppEvent::SelectedOption), payload);
                        }
                        None => {
                            terminal
                                .write_line(
                                    "You didn't select one of the options. Please try again.",
                                )
                                .unwrap();
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                    }
                }
            }
            AppState::Running => {
                terminal.set_title("Minecraft Server");

                let vars = run_server(
                    machine.configuration.clone(),
                    jar_filename.clone(),
                    terminal.clone(),
                );
                machine.configuration = vars.0;
                terminal = vars.1;

                machine.dispatch(Event::AppEvent(AppEvent::Exit), None);
            }
            AppState::Exited => {}
            AppState::EditingConfiguration => {
                terminal.set_title("Minecraft Server CLI — Editing Configuration");
                let editor_state = machine.editor_state.clone().expect(
                    "Expected to have an editor state while in EditingConfiguration app state.",
                );
                let option = machine.selected_configuration_option.as_ref().expect("Expected a configuration option to have been chosen before editing the configuration.").clone();
                terminal
                    .write_line(
                        format!(
                            "Editing Configuration > {}",
                            style(option.name).bold().to_string()
                        )
                        .as_str(),
                    )
                    .unwrap();
                terminal.write_line(option.description.as_str()).unwrap();

                match editor_state {
                    EditorState::SelectOnOff => {
                        let result = dialoguer::Select::new()
                            .items(&vec!["Enable", "Disable"])
                            .interact_on_opt(&terminal)
                            .unwrap()
                            .unwrap();

                        machine.dispatch(
                            Event::EditorEvent(EditorEvent::SubmitValue),
                            Some(Payload::ConfigurationOptionType(
                                ConfigurationOptionType::Bool(result == 0),
                            )),
                        );
                        machine.configuration = save_configuration(machine.configuration);
                    }
                    EditorState::NumberInput => {
                        let result: String =
                            dialoguer::Input::new().interact_text_on(&terminal).unwrap();

                        match result.parse::<u16>() {
                            Ok(result) => {
                                if result == 0 {
                                    terminal.write_line("You entered an invalid port number of \"0\". Please try again.").unwrap();
                                    std::thread::sleep(std::time::Duration::from_secs(1));
                                } else {
                                    machine.dispatch(
                                        Event::EditorEvent(EditorEvent::SubmitValue),
                                        Some(Payload::ConfigurationOptionType(
                                            ConfigurationOptionType::OptionU16(Some(result)),
                                        )),
                                    );
                                    machine.configuration =
                                        save_configuration(machine.configuration);
                                }
                            }
                            Err(_) => {
                                terminal.write_line(format!("You entered an invalid port number of \"{}\". Please try again.", result).as_str()).unwrap();
                                std::thread::sleep(std::time::Duration::from_secs(1));
                            }
                        }
                    }
                    EditorState::TextInput => {
                        let result: String =
                            dialoguer::Input::new().interact_text_on(&terminal).unwrap();

                        machine.dispatch(
                            Event::EditorEvent(EditorEvent::SubmitValue),
                            Some(Payload::ConfigurationOptionType(
                                ConfigurationOptionType::OptionString(Some(result)),
                            )),
                        );
                        machine.configuration = save_configuration(machine.configuration);
                    }
                    EditorState::SelectValueOrNone => {
                        let result = dialoguer::Select::new()
                            .items(&vec!["Enter a value", "Disable"])
                            .interact_on_opt(&terminal)
                            .unwrap()
                            .unwrap();
                        if result == 0 {
                            machine.dispatch(Event::EditorEvent(EditorEvent::SelectedValue), None);
                        } else {
                            machine.dispatch(Event::EditorEvent(EditorEvent::SelectedNone), None);
                            machine.configuration = save_configuration(machine.configuration);
                        }
                    }
                }
            }
        }
    }
}
