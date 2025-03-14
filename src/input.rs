//! Git Interactive Rebase Tool - Input Module
//!
//! # Description
//! This module is used to handle working with input events.
//!
//! ## Test Utilities
//! To facilitate testing the usages of this crate, a set of testing utilities are provided. Since
//! these utilities are not tested, and often are optimized for developer experience than
//! performance should only be used in test code.

mod event;
mod event_handler;
mod event_provider;
mod input_options;
mod key_bindings;
mod key_event;
mod map_keybindings;
mod standard_event;
mod thread;

pub(crate) use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

pub(crate) use self::{
	event::Event,
	event_handler::EventHandler,
	event_provider::{EventReaderFn, read_event},
	input_options::InputOptions,
	key_bindings::KeyBindings,
	key_event::KeyEvent,
	map_keybindings::map_keybindings,
	standard_event::StandardEvent,
	thread::{State, THREAD_NAME, Thread},
};
