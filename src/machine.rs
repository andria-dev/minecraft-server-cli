use std::mem;

use enumflags2::{BitFlags, bitflags};
use serde::{Serialize, Deserialize};

// Configuration data structure. This is what we edit and persist to the disk.
#[derive(Serialize, Deserialize, Debug)]
pub struct MinecraftServerConfiguration {
	pub bonusChest: bool,
	pub demo: bool,
	pub eraseCache: bool,
	pub forceUpgrade: bool,
	pub initSettings: bool,
	pub gui: bool,
	pub port: Option<u16>, // u16 is the equivalent of 2^16-1 (0–65535). All ports are 1–65535
	pub safeMode: bool,
	pub singleplayer: bool,
	pub universe: Option<String>,
	pub world: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum AppState {
	ChoiceMenu,
	Running,
	Exited,
	EditingConfiguration,
}
#[derive(Debug, Clone, Copy)]
enum AppEvent {
	StartServer,
	Exit,
	SelectedOption,
}

#[derive(Debug, PartialEq)]
enum EditorState {
	SelectOnOff,
	NumberInput,
	TextInput,
	SelectValueOrNone,
}
#[derive(Debug, Clone, Copy)]
enum EditorEvent {
	SubmitValue,
	SelectedValue,
	SelectedNone,
}

#[derive(Debug, Clone, Copy)]
enum Event {
	AppEvent(AppEvent),
	EditorEvent(EditorEvent)
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ConfigurationOptionType {
	Bool,
	U16,
	String,
	Option
}

#[derive(Debug, Clone)]
pub struct ConfigurationOption {
	pub property: String,
	pub name: String,
	pub description: String,
	pub r#type: BitFlags<ConfigurationOptionType>,
}

#[derive(Debug, Clone)]
enum Payload {
	ConfigurationOption(ConfigurationOption)
}

struct Machine {
	state: AppState,
	editorState: Option<EditorState>,
}

impl Machine {
	fn dispatch(&mut self, event: Event, payload: Option<Payload>) -> () {
		let state = self.state.clone();
		match event {
    	Event::AppEvent(event) => { self.state = match (state, event) {
				(AppState::ChoiceMenu, AppEvent::StartServer) => AppState::Running,
				(AppState::ChoiceMenu, AppEvent::Exit) => AppState::Exited,
				(AppState::ChoiceMenu, AppEvent::SelectedOption) => {
					let Payload::ConfigurationOption(payload) = payload.expect("A ConfigurationOption payload was not provided when the SelectedOption event was dispatched from the ChoiceMenu state.");

					self.editorState = Some(if payload.r#type == ConfigurationOptionType::Option {
						EditorState::SelectValueOrNone
					} else if payload.r#type == ConfigurationOptionType::Bool {
						EditorState::SelectOnOff
					} else if payload.r#type == ConfigurationOptionType::U16 {
						EditorState::NumberInput
					} else {
						EditorState::TextInput
					});

					AppState::EditingConfiguration
				},
				_ => state,
			}}
			Event::EditorEvent(event) => {
			}
		}
	}
}
