mod insert_state;
mod line_type;

#[cfg(all(unix, test))]
mod tests;

use display::DisplayColor;
use input::{Event, InputOptions};
use todo_file::{Line, TodoFile};
use view::{LineSegment, RenderContext, ViewData, ViewDataUpdater, ViewLine, ViewSender};

use self::{insert_state::InsertState, line_type::LineType};
use crate::{
	components::{
		choice::{Choice, INPUT_OPTIONS as CHOICE_INPUT_OPTIONS},
		edit::{Edit, INPUT_OPTIONS as EDIT_INPUT_OPTIONS},
	},
	module::{Module, ProcessResult, State},
};

pub(crate) struct Insert {
	action_choices: Choice<LineType>,
	edit: Edit,
	line_type: LineType,
	state: InsertState,
}

impl Module for Insert {
	fn activate(&mut self, _: &TodoFile, _: State) -> ProcessResult {
		self.state = InsertState::Prompt;
		self.edit.clear();
		ProcessResult::new()
	}

	fn build_view_data(&mut self, _: &RenderContext, _: &TodoFile) -> &ViewData {
		match self.state {
			InsertState::Prompt => self.action_choices.get_view_data(),
			InsertState::Edit => {
				self.edit.build_view_data(
					|updater: &mut ViewDataUpdater<'_>| {
						updater.push_leading_line(ViewLine::from(vec![LineSegment::new_with_color(
							"Enter contents of the new line. Empty content cancels creation of a new line.",
							DisplayColor::IndicatorColor,
						)]));
						updater.push_leading_line(ViewLine::new_empty_line());
					},
					|_| {},
				)
			},
		}
	}

	fn input_options(&self) -> &InputOptions {
		match self.state {
			InsertState::Prompt => &CHOICE_INPUT_OPTIONS,
			InsertState::Edit => &EDIT_INPUT_OPTIONS,
		}
	}

	fn handle_event(&mut self, event: Event, view_sender: &ViewSender, rebase_todo: &mut TodoFile) -> ProcessResult {
		match self.state {
			InsertState::Prompt => {
				let choice = self.action_choices.handle_event(event, view_sender);
				let mut result = ProcessResult::from(event);
				if let Some(action) = choice {
					if action == &LineType::Cancel {
						result = result.state(State::List);
					}
					else {
						self.line_type = action.clone();
						self.edit.set_label(format!("{} ", action.to_string()).as_str());
						self.state = InsertState::Edit;
					}
				}
				result
			},
			InsertState::Edit => {
				let mut result = ProcessResult::from(event);
				self.edit.handle_event(event);
				if self.edit.is_finished() {
					let content = self.edit.get_content();
					result = result.state(State::List);
					if !content.is_empty() {
						let line = match self.line_type {
							LineType::Exec => Line::new_exec(content.as_str()),
							LineType::Pick => Line::new_pick(content.as_str()),
							LineType::Label => Line::new_label(content.as_str()),
							LineType::Reset => Line::new_reset(content.as_str()),
							LineType::Merge => Line::new_merge(content.as_str()),
							// this should exit in the prompt state and never get here
							LineType::Cancel => unreachable!(),
						};
						let new_line_index = rebase_todo.get_selected_line_index() + 1;
						rebase_todo.add_line(new_line_index, line);
						rebase_todo.set_selected_line_index(new_line_index);
					}
				}
				result
			},
		}
	}
}

impl Insert {
	pub(crate) fn new() -> Self {
		let mut action_choices = Choice::new(vec![
			(LineType::Exec, 'e', String::from("exec <command>")),
			(LineType::Pick, 'p', String::from("pick <hash>")),
			(LineType::Label, 'l', String::from("label <label>")),
			(LineType::Reset, 'r', String::from("reset <label>")),
			(
				LineType::Merge,
				'm',
				String::from("merge [-C <commit> | -c <commit>] <label> [# <oneline>]"),
			),
			(LineType::Cancel, 'q', String::from("Cancel add line")),
		]);
		action_choices.set_prompt(vec![ViewLine::from("Select the type of line to insert:")]);

		Self {
			state: InsertState::Prompt,
			edit: Edit::new(),
			action_choices,
			line_type: LineType::Exec,
		}
	}
}
