mod render_action;

#[cfg(test)]
mod tests;

use std::{
	collections::{HashMap, VecDeque},
	mem,
};

pub(crate) use self::render_action::RenderAction;
use crate::view::{LineSegment, ScrollPosition, ViewData, ViewLine, ViewLines};

#[derive(Debug)]
pub(crate) struct RenderSlice {
	actions: VecDeque<RenderAction>,
	height: usize,
	lines: ViewLines,
	lines_count: usize,
	lines_leading_count: usize,
	lines_trailing_count: usize,
	padding_height: usize,
	scroll_position: ScrollPosition,
	scroll_position_cache: HashMap<String, ScrollPosition>,
	should_show_scrollbar: bool,
	show_help: bool,
	show_title: bool,
	version: u32,
	view_data_name: String,
	view_data_version: u32,
	width: usize,
}

impl RenderSlice {
	pub(crate) fn new() -> Self {
		Self {
			actions: VecDeque::new(),
			height: 0,
			lines: ViewLines::new(),
			lines_count: 0,
			lines_leading_count: 0,
			lines_trailing_count: 0,
			padding_height: 0,
			scroll_position: ScrollPosition::new(),
			scroll_position_cache: HashMap::new(),
			should_show_scrollbar: false,
			show_help: false,
			show_title: false,
			version: 0,
			view_data_name: String::new(),
			view_data_version: 0,
			width: 0,
		}
	}

	pub(crate) fn record_scroll_up(&mut self) {
		self.actions.push_back(RenderAction::ScrollUp);
	}

	pub(crate) fn record_scroll_down(&mut self) {
		self.actions.push_back(RenderAction::ScrollDown);
	}

	pub(crate) fn record_page_up(&mut self) {
		self.actions.push_back(RenderAction::PageUp);
	}

	pub(crate) fn record_page_down(&mut self) {
		self.actions.push_back(RenderAction::PageDown);
	}

	pub(crate) fn record_scroll_left(&mut self) {
		self.actions.push_back(RenderAction::ScrollLeft);
	}

	pub(crate) fn record_scroll_right(&mut self) {
		self.actions.push_back(RenderAction::ScrollRight);
	}

	pub(crate) fn record_scroll_top(&mut self) {
		self.actions.push_back(RenderAction::ScrollTop);
	}

	pub(crate) fn record_scroll_bottom(&mut self) {
		self.actions.push_back(RenderAction::ScrollBottom);
	}

	pub(crate) fn record_resize(&mut self, width: usize, height: usize) {
		self.actions.push_back(RenderAction::Resize(width, height));
	}

	pub(crate) fn sync_view_data(&mut self, view_data: &ViewData) {
		let cache_expired = self.cache_expired(view_data);
		// scroll position depends on padding, so if the view has changed it needs to be updated early
		if cache_expired {
			self.set_padding_height(view_data);
		}
		self.set_active_scroll_position(view_data);
		let has_actions = !self.actions.is_empty();
		while let Some(action) = self.actions.pop_front() {
			match action {
				RenderAction::ScrollDown => self.scroll_position.scroll_down(),
				RenderAction::ScrollUp => self.scroll_position.scroll_up(),
				RenderAction::ScrollRight => self.scroll_position.scroll_right(),
				RenderAction::ScrollLeft => self.scroll_position.scroll_left(),
				RenderAction::ScrollTop => self.scroll_position.scroll_top(),
				RenderAction::ScrollBottom => self.scroll_position.scroll_bottom(),
				RenderAction::PageUp => self.scroll_position.page_up(),
				RenderAction::PageDown => self.scroll_position.page_down(),
				RenderAction::Resize(width, height) => self.set_size(width, height),
			}
		}
		if has_actions || cache_expired {
			self.rebuild(view_data);
		}
	}

	pub(crate) const fn should_show_scroll_bar(&self) -> bool {
		self.should_show_scrollbar
	}

	#[expect(
		clippy::cast_precision_loss,
		clippy::cast_possible_truncation,
		clippy::cast_sign_loss,
		reason = "Integer values expected."
	)]
	pub(crate) fn get_scroll_index(&self) -> usize {
		if self.lines_count == 0 || self.scroll_position.get_top_position() == 0 {
			return 0;
		}

		let view_height = self.height.saturating_sub(self.padding_height);

		if view_height <= 1 || view_height > self.lines_count {
			return 0;
		}

		// view_height >= 2
		// lines.len() > 2

		// if at bottom of list
		if self.scroll_position.get_top_position() >= self.lines_count - view_height {
			return view_height - 1;
		}

		if view_height <= 2 {
			return 0;
		}

		// 0 input range, if first and last item are pinned, so scroll center
		if self.lines_count - view_height <= 2 {
			return (0.5 * view_height as f64).round() as usize;
		}

		// linear range map from scroll range to view range. This only maps the range between the
		// first and last items, since those items are always returned as 0 or view_height
		let value = self.scroll_position.get_top_position() as f64;
		let input_start = 1.0;
		let input_end = (self.lines_count - view_height) as f64 - 1.0;
		let output_start = 1.0;
		let output_end = view_height as f64 - 2.0;
		let input_range = input_end - input_start;
		let output_range = output_end - output_start;
		let slope = output_range / input_range;
		slope.mul_add(value - input_start, output_start).round() as usize
	}

	pub(crate) const fn show_title(&self) -> bool {
		self.show_title
	}

	pub(crate) const fn show_help(&self) -> bool {
		self.show_help
	}

	pub(crate) const fn get_leading_lines_count(&self) -> usize {
		self.lines_leading_count
	}

	pub(crate) const fn get_trailing_lines_count(&self) -> usize {
		self.lines_trailing_count
	}

	pub(crate) const fn view_lines(&self) -> &ViewLines {
		&self.lines
	}

	pub(crate) const fn get_version(&self) -> u32 {
		self.version
	}

	#[cfg(test)]
	pub(crate) const fn get_actions(&self) -> &VecDeque<RenderAction> {
		&self.actions
	}

	fn cache_expired(&self, view_data: &ViewData) -> bool {
		self.view_data_name != view_data.get_name() || self.view_data_version != view_data.get_version()
	}

	fn set_size(&mut self, view_width: usize, view_height: usize) {
		if self.height != view_height || self.width != view_width {
			self.height = view_height;
			self.width = view_width;
			self.update_scroll_position_size();
		}
	}

	fn set_padding_height(&mut self, view_data: &ViewData) {
		let padding_height = if view_data.show_title() { 1 } else { 0 }
			+ view_data.leading_lines().count() as usize
			+ view_data.trailing_lines().count() as usize;

		if self.padding_height != padding_height {
			self.padding_height = padding_height;
			self.update_scroll_position_size();
		}
	}

	fn set_active_scroll_position(&mut self, view_data: &ViewData) {
		let name = view_data.get_name();
		if name != self.view_data_name {
			let previous_scroll_position = mem::replace(
				&mut self.scroll_position,
				self.scroll_position_cache
					.remove(&String::from(name))
					.unwrap_or_else(ScrollPosition::new),
			);
			_ = self
				.scroll_position_cache
				.insert(String::from(self.view_data_name.as_str()), previous_scroll_position);
			let version = view_data.get_scroll_version();
			if self.scroll_position.get_version() != version || !view_data.retain_scroll_position() {
				self.scroll_position.reset();
				self.scroll_position.set_version(version);
			}
			self.update_scroll_position_size();
		}
	}

	fn update_scroll_position_size(&mut self) {
		self.scroll_position.resize(
			if self.height == 0 || self.padding_height > self.height {
				0
			}
			else {
				self.height - self.padding_height
			},
			if self.width == 0 {
				0
			}
			else {
				self.width - if self.should_show_scroll_bar() { 1 } else { 0 }
			},
		);
	}

	fn rebuild(&mut self, view_data: &ViewData) {
		let leading_lines_length = view_data.leading_lines().count() as usize;
		let trailing_lines_length = view_data.trailing_lines().count() as usize;
		let lines_length = view_data.lines().count() as usize;

		self.version += 1;
		self.view_data_name = String::from(view_data.get_name());
		self.view_data_version = view_data.get_version();
		self.show_title = view_data.show_title();
		self.show_help = view_data.show_help();
		self.should_show_scrollbar =
			self.padding_height < self.height && lines_length > (self.height - self.padding_height);

		self.scroll_position.set_lines_length(lines_length);
		for row in view_data.visible_rows() {
			self.scroll_position.ensure_line_visible(*row);
		}

		let (leading_lines_end, max_leading_line_length) = if leading_lines_length == 0 {
			(0, 0)
		}
		else {
			// trailing lines have precedence over leading lines, title always has precedence
			let padding_height = if self.show_title { 1 } else { 0 } + trailing_lines_length;
			let available_height = self.height.saturating_sub(padding_height);

			let leading_lines_end = if leading_lines_length < available_height {
				leading_lines_length
			}
			else {
				available_height
			};

			(
				leading_lines_end,
				Self::calculate_max_line_length(view_data.leading_lines(), 0, leading_lines_end),
			)
		};

		let (trailing_lines_end, max_trailing_line_length) = if trailing_lines_length == 0 {
			(0, 0)
		}
		else {
			// title always has precedence
			let padding_height = if self.show_title { 1 } else { 0 };
			let available_height = self.height.saturating_sub(padding_height);

			let trailing_lines_end = if trailing_lines_length < available_height {
				trailing_lines_length
			}
			else {
				available_height
			};

			(
				trailing_lines_end,
				Self::calculate_max_line_length(view_data.trailing_lines(), 0, trailing_lines_end),
			)
		};

		let (lines_start, lines_end, max_line_length) = if lines_length == 0 {
			(0, 0, 0)
		}
		else {
			// all other lines take precedence over regular lines
			let available_height = self.height.saturating_sub(self.padding_height);

			let lines_start = self.scroll_position.get_top_position();

			let lines_end = if lines_length < available_height {
				lines_length
			}
			else {
				available_height
			};

			let max_line_length = Self::calculate_max_line_length(view_data.lines(), lines_start, lines_end);
			(
				lines_start,
				lines_end,
				max_line_length + if self.should_show_scrollbar { 1 } else { 0 },
			)
		};

		self.lines_leading_count = leading_lines_end;
		self.lines_trailing_count = trailing_lines_end;
		self.lines_count = lines_length;
		self.scroll_position.set_max_line_length(
			max_line_length
				.max(max_leading_line_length)
				.max(max_trailing_line_length),
		);
		if let Some(column) = view_data.get_visible_column() {
			self.scroll_position.ensure_column_visible(column);
		}

		self.lines.clear();
		self.push_lines(view_data.leading_lines(), 0, leading_lines_end, false);
		self.push_lines(view_data.lines(), lines_start, lines_end, self.should_show_scrollbar);
		self.push_lines(view_data.trailing_lines(), 0, trailing_lines_end, false);
	}

	fn calculate_max_line_length(view_lines: &ViewLines, start: usize, length: usize) -> usize {
		view_lines
			.iter()
			.skip(start)
			.take(length)
			.fold(0, |longest, line| -> usize {
				if line.get_segments().len() <= line.get_number_of_pinned_segment() {
					longest
				}
				else {
					let sum = line.get_segments().iter().fold(0, |s, l| s + l.get_length());

					if sum > longest { sum } else { longest }
				}
			})
	}

	fn push_lines(&mut self, view_lines: &ViewLines, start: usize, end: usize, scroll_bar: bool) {
		let window_width = if scroll_bar && self.width > 0 {
			self.width - 1
		}
		else {
			self.width
		};
		let left = self.scroll_position.get_left_position();

		view_lines.iter().skip(start).take(end).for_each(|line| {
			let mut cursor = 0;
			let mut left_start = 0;
			let mut segments = vec![];
			// window width can be zero when there is a scrollbar and a view width of 1
			if window_width > 0 {
				for (i, segment) in line.get_segments().iter().enumerate() {
					// set left on first non-pinned segment
					if i == line.get_number_of_pinned_segment() {
						left_start = left;
					}

					let partial = segment.get_partial_segment(left_start, window_width - cursor);

					if partial.get_length() > 0 {
						segments.push(LineSegment::new_copy_style(partial.get_content(), segment));

						cursor += partial.get_length();
						if cursor >= window_width {
							break;
						}
						left_start = 0;
					}
					else {
						left_start -= segment.get_length();
					}
				}

				if cursor < window_width {
					if let Some(padding) = line.get_padding() {
						segments.push(LineSegment::new_copy_style(
							padding.get_content().repeat(window_width - cursor).as_str(),
							padding,
						));
					}
				}
			}

			self.lines.push(
				ViewLine::new_with_pinned_segments(segments, line.get_number_of_pinned_segment())
					.set_selected(line.get_selected()),
			);
		});
	}
}
