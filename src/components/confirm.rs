mod confirmed;
#[cfg(test)]
mod tests;

use std::sync::LazyLock;

use captur::capture;

pub(crate) use self::confirmed::Confirmed;
use crate::{
	input::{Event, InputOptions, KeyBindings, KeyCode, KeyEvent, StandardEvent},
	view::{ViewData, ViewLine},
};

pub(crate) static INPUT_OPTIONS: LazyLock<InputOptions> =
	LazyLock::new(|| InputOptions::RESIZE | InputOptions::MOVEMENT);

pub(crate) struct Confirm {
	view_data: ViewData,
}

impl Confirm {
	pub(crate) fn new(prompt: &str, confirm_yes: &[String], confirm_no: &[String]) -> Self {
		let view_data = ViewData::new(|updater| {
			capture!(confirm_yes, confirm_no);
			updater.set_show_title(true);
			updater.set_retain_scroll_position(false);
			updater.push_line(ViewLine::from(format!(
				"{prompt} ({}/{})? ",
				confirm_yes.join(","),
				confirm_no.join(",")
			)));
		});
		Self { view_data }
	}

	pub(crate) fn get_view_data(&mut self) -> &ViewData {
		&self.view_data
	}

	pub(crate) fn read_event(event: Event, key_bindings: &KeyBindings) -> Event {
		if let Event::Key(key) = event {
			if let KeyCode::Char(c) = key.code {
				let event_lower_modifiers = key.modifiers;
				let event_lower = Event::Key(KeyEvent::new(
					KeyCode::Char(c.to_ascii_lowercase()),
					event_lower_modifiers,
				));
				let event_upper = Event::Key(KeyEvent::new(KeyCode::Char(c.to_ascii_uppercase()), key.modifiers));

				return if key_bindings.confirm_yes.contains(&event_lower)
					|| key_bindings.confirm_yes.contains(&event_upper)
				{
					Event::from(StandardEvent::Yes)
				}
				else {
					Event::from(StandardEvent::No)
				};
			}
		}
		event
	}

	pub(crate) const fn handle_event(&self, event: Event) -> Confirmed {
		if let Event::Standard(standard_event) = event {
			match standard_event {
				StandardEvent::Yes => Confirmed::Yes,
				StandardEvent::No => Confirmed::No,
				_ => Confirmed::Other,
			}
		}
		else {
			Confirmed::Other
		}
	}
}
