use blaze_explorer_lib::command::command_helpers::match_popup_call;
use blaze_explorer_lib::{action::Action, app::App, command::Command};

use crate::TelescopeWindow;
//Plugin functions
pub fn open_sfs(app: &mut App) -> Option<Action> {
    let ctx = app.get_app_context();
    let plugin = match app.plugins.get("Telescope") {
        None => {
            return Some(Action::AppAct(
                blaze_explorer_lib::action::AppAction::DisplayMessage(
                    "Failed to fetch the Telescope plugin when trying to open the popup"
                        .to_string(),
                ),
            ));
        }
        Some(plugin) => plugin,
    };
    let popup_keymap = plugin.get_popup_keymap();
    let popup = Box::new(TelescopeWindow::new_sfs(ctx, popup_keymap));
    app.attach_popup(popup);

    None
}

//Popup functions
#[derive(Clone, PartialEq, Debug)]
pub struct TelescopeUpdateSearchQuery {
    query: String,
}

impl TelescopeUpdateSearchQuery {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}
impl Command for TelescopeUpdateSearchQuery {
    fn execute(&mut self, app: &mut App) -> Option<Action> {
        match &mut app.popup {
            None => None,
            Some(ref mut popup) => {
                popup.update_search_query(self.query.clone());
                None
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TelescopeEraseText {}

impl TelescopeEraseText {
    pub fn new() -> Self {
        Self {}
    }
}
impl Command for TelescopeEraseText {
    fn execute(&mut self, app: &mut App) -> Option<Action> {
        match_popup_call!(app, erase_text)
    }
}

#[cfg(test)]
mod tests {
    use blaze_explorer_lib::action::AppAction;

    use super::*;

    #[test]
    fn test_open_sfs_fails() {
        let mut app = App::new().unwrap();
        let result = open_sfs(&mut app);
        let expected_result = Some(Action::AppAct(AppAction::DisplayMessage(
            "Failed to fetch the Telescope plugin when trying to open the popup".to_string(),
        )));

        assert_eq!(result, expected_result);
    }
}
