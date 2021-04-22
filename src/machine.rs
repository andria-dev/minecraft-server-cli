use std::{collections::HashMap, mem};

use enumflags2::{BitFlags, bitflags};
use serde::{Serialize, Deserialize};

// Configuration data structure. This is what we edit and persist to the disk.
#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigurationOptionType {
	Bool(bool),
	OptionU16(Option<u16>),
	OptionString(Option<String>),
}
impl MinecraftServerConfiguration {
	fn set(&mut self, property: String, value: ConfigurationOptionType) {
		let property = property.as_str();
		if let ConfigurationOptionType::Bool(value) = value {
			match property {
				"bonusChest" => self.bonusChest = value,
				"demo" => self.demo = value,
				"eraseCache" => self.eraseCache = value,
				"forceUpgrade" => self.forceUpgrade = value,
				"initSettings" => self.initSettings = value,
				"gui" => self.gui = value,
				_ => {},
			}
		} else if let ConfigurationOptionType::OptionU16(value) = value {
			match property {
				"port" => self.port = value,
				_ => {},
			}
		} else if let ConfigurationOptionType::OptionString(value) = value {
			match property {
				"universe" => self.universe = value,
				"world" => self.world = value,
				_ => {},
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AppState {
	ChoiceMenu,
	Running,
	Exited,
	EditingConfiguration,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AppEvent {
	StartServer,
	Exit,
	SelectedOption,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EditorState {
	SelectOnOff,
	NumberInput,
	TextInput,
	SelectValueOrNone,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EditorEvent {
	SubmitValue,
	SelectedValue,
	SelectedNone,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
	AppEvent(AppEvent),
	EditorEvent(EditorEvent)
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConfigurationOptionTypeFlag {
	Bool,
	U16,
	String,
	Option,
}

#[derive(Debug, Clone)]
pub struct ConfigurationOption {
	pub property: String,
	pub name: String,
	pub description: String,
	pub r#type: BitFlags<ConfigurationOptionTypeFlag>,
}

#[derive(Debug, Clone)]
pub enum Payload {
	ConfigurationOption(ConfigurationOption),
	ConfigurationOptionType(ConfigurationOptionType)
}

pub struct Machine {
	pub state: AppState,
	pub editorState: Option<EditorState>,
	pub selectedConfigurationOption: Option<ConfigurationOption>,
	pub configuration: MinecraftServerConfiguration,
}

impl Machine {
	fn set_option_value(&mut self, payload: ConfigurationOptionType) {
		let property = self.selectedConfigurationOption.clone().expect("A configuration option was never selected before attempting to set its value.").property;
		self.configuration.set(property, payload);
		self.selectedConfigurationOption = None;
	}

	pub fn dispatch(&mut self, event: Event, payload: Option<Payload>) -> () {
		let state = self.state.clone();
		match event {
    	Event::AppEvent(event) => { self.state = match (state, event) {
				(AppState::ChoiceMenu, AppEvent::StartServer) => AppState::Running,
				(AppState::ChoiceMenu, AppEvent::Exit) => AppState::Exited,
				(AppState::ChoiceMenu, AppEvent::SelectedOption) => {
					if let Payload::ConfigurationOption(payload) = payload.expect("A ConfigurationOption payload was not provided when the SelectedOption event was dispatched from the ChoiceMenu state.") {
						self.selectedConfigurationOption = Some(payload.clone());
						self.editorState = Some(if payload.r#type == ConfigurationOptionTypeFlag::Option {
							EditorState::SelectValueOrNone
						} else if payload.r#type == ConfigurationOptionTypeFlag::Bool {
							EditorState::SelectOnOff
						} else if payload.r#type == ConfigurationOptionTypeFlag::U16 {
							EditorState::NumberInput
						} else {
							EditorState::TextInput
						});
					}
					AppState::EditingConfiguration
				},
				_ => state,
			}}
			Event::EditorEvent(event) => {
				let optionEditorState = self.editorState.clone();
				let editorState = optionEditorState.expect("An EditorEvent has been dispatched while not in the EditorState");
				let none: Option<EditorState> = None;
				self.editorState = match (editorState, event) {
					(EditorState::SelectOnOff, EditorEvent::SubmitValue) => {
						if let Payload::ConfigurationOptionType(value) = payload.expect("Expected true or false.") {
							self.set_option_value(value);
							self.state = AppState::ChoiceMenu;
							none
						} else {
							optionEditorState
						}
					}
					(EditorState::NumberInput, EditorEvent::SubmitValue) => {
						if let Payload::ConfigurationOptionType(value) = payload.expect("Expected true or false.") {
							self.set_option_value(value);
							self.state = AppState::ChoiceMenu;
							none
						} else {
							optionEditorState
						}
					}
					(EditorState::TextInput, EditorEvent::SubmitValue) => {
						if let Payload::ConfigurationOptionType(value) = payload.expect("Expected true or false.") {
							self.set_option_value(value);
							self.state = AppState::ChoiceMenu;
							none
						} else {
							optionEditorState
						}
					}
					(EditorState::SelectValueOrNone, EditorEvent::SelectedValue) => {
						let selected = self.selectedConfigurationOption.clone().expect("");
						if selected.r#type == ConfigurationOptionTypeFlag::U16 {
							Some(EditorState::NumberInput)
						} else if selected.r#type == ConfigurationOptionTypeFlag::String {
							Some(EditorState::TextInput)
						} else {
							optionEditorState
						}
					}
					(EditorState::SelectValueOrNone, EditorEvent::SelectedNone) => {
						let selected = self.selectedConfigurationOption.clone().expect("");
						if selected.r#type == ConfigurationOptionTypeFlag::U16 {
							let none = ConfigurationOptionType::OptionU16(None);
							self.set_option_value(none);
						} else if selected.r#type == ConfigurationOptionTypeFlag::String {
							let none = ConfigurationOptionType::OptionString(None);
							self.set_option_value(none);
						}
						self.state = AppState::ChoiceMenu;
						None
					}
					_ => optionEditorState,
				}
			}
		}
	}
}
