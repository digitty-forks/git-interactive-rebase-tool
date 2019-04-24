#[derive(Debug, PartialEq)]
pub enum Input {
	Abort,
	ActionBreak,
	ActionDrop,
	ActionEdit,
	ActionFixup,
	ActionPick,
	ActionReword,
	ActionSquash,
	Backspace,
	Character(char),
	Edit,
	Enter,
	Delete,
	ForceAbort,
	ForceRebase,
	Help,
	MoveCursorDown,
	MoveCursorLeft,
	MoveCursorPageDown,
	MoveCursorPageUp,
	MoveCursorRight,
	MoveCursorUp,
	OpenInEditor,
	Other,
	Rebase,
	Resize,
	ShowCommit,
	SwapSelectedDown,
	SwapSelectedUp,
	ToggleVisualMode,
}