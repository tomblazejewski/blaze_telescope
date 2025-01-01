pub mod sfs_telescope;
pub mod telescope_backend;
pub mod telescope_commands;
pub mod telescope_query;

use blaze_explorer_lib::{create_plugin_action, insert_binding};

use ratatui::crossterm::event::KeyCode;
use telescope_commands::{
    open_sfs, TelescopeConfirmResult, TelescopeDropSearchChar, TelescopeNextResult,
    TelescopePreviousResult, TelescopePushSearchChar, TelescopeQuit,
};

use color_eyre::eyre::Result;
use ratatui::{crossterm::event::KeyEvent, layout::Rect, widgets::Clear, Frame};
use std::collections::HashMap;
use telescope_backend::TelescopeBackend;

use blaze_explorer_lib::plugin::Plugin;

use blaze_explorer_lib::{
    action::{Action, AppAction},
    app::App,
    app_context::AppContext,
    custom_action,
    input_machine::{
        input_machine_helpers::convert_str_to_events, InputMachine, KeyProcessingResult,
    },
    line_entry::LineEntry,
    mode::Mode,
    plugin::plugin_action::PluginAction,
    plugin::plugin_popup::PluginPopUp,
};

//The plugin consists of the following parts
//Struct Telescope - defines functionalities available at the app level. The app can bind any of
//these actions to a keymap to use it. Telescope implements Plugin.
//Struct TelescopeWindow (implementing PluginPopUp) - this is spawned upon calling one of the
//plugin's functionalities and takes control of incoming KeyEvents
//

//Create types for simplicity
type CustomAction = fn(&mut App) -> Option<Action>;
type BoxedAction = Box<CustomAction>;

//Plugin defaults:

//Plugin getter
#[no_mangle]
pub extern "Rust" fn get_plugin(
    bindings_map: HashMap<(Mode, Vec<KeyEvent>), String>,
) -> Box<dyn Plugin> {
    Box::new(Telescope::new(bindings_map))
}

//Default Popup Action
pub fn default_popup_action(key_event: KeyEvent) -> Option<Action> {
    match key_event.code {
        KeyCode::Char(ch) => Some(create_plugin_action!(TelescopePushSearchChar, ch)),
        _ => None,
    }
}

//Functionalities offered by the plugin
pub fn get_functionalities() -> HashMap<String, Action> {
    let mut functionality_map = HashMap::new();
    functionality_map.insert("OpenSFS".to_string(), custom_action!(open_sfs));
    functionality_map.insert(
        "TelescopeQuit".to_string(),
        create_plugin_action!(TelescopeQuit),
    );
    functionality_map.insert(
        "TelescopeNextResult".to_string(),
        create_plugin_action!(TelescopeNextResult),
    );
    functionality_map.insert(
        "TelescopePreviousResult".to_string(),
        create_plugin_action!(TelescopePreviousResult),
    );
    functionality_map.insert(
        "TelescopeDropSearchChar".to_string(),
        create_plugin_action!(TelescopeDropSearchChar),
    );
    functionality_map.insert(
        "TelescopeConfirmResult".to_string(),
        create_plugin_action!(TelescopeConfirmResult),
    );
    functionality_map.insert("OpenSFS".to_string(), custom_action!(open_sfs));

    functionality_map
}

//Default bindings
pub fn get_default_bindings() -> HashMap<(Mode, Vec<KeyEvent>), String> {
    let mut bindings_map = HashMap::new();
    insert_binding!(bindings_map, Mode::Normal, " sg", "OpenSFS");
    insert_binding!(bindings_map, Mode::PopUp, "<Esc>", "TelescopeQuit");

    insert_binding!(bindings_map, Mode::PopUp, "<C-n>", "TelescopeNextResult");

    insert_binding!(
        bindings_map,
        Mode::PopUp,
        "<C-p>",
        "TelescopePreviousResult"
    );

    insert_binding!(bindings_map, Mode::PopUp, "<BS>", "TelescopeDropSearchChar");

    insert_binding!(bindings_map, Mode::PopUp, "<CR>", "TelescopeConfirmResult");
    bindings_map
}

#[derive(Debug, Clone)]
pub struct Telescope {
    plugin_bindings: HashMap<(Mode, Vec<KeyEvent>), String>,
    popup_bindings: HashMap<(Mode, Vec<KeyEvent>), String>,
    functionality_map: HashMap<String, Action>,
}

impl Telescope {
    pub fn new(custom_bindings_map: HashMap<(Mode, Vec<KeyEvent>), String>) -> Self {
        let functionality_map = get_functionalities();
        let mut bindings_map = get_default_bindings();
        bindings_map.extend(custom_bindings_map);

        let mut plugin_bindings = HashMap::new();
        let mut popup_bindings = HashMap::new();

        for ((mode, events), string_repr) in bindings_map.iter() {
            match mode {
                Mode::PopUp => {
                    popup_bindings.insert((mode.clone(), events.clone()), string_repr.clone());
                }
                _ => {
                    plugin_bindings.insert((mode.clone(), events.clone()), string_repr.clone());
                }
            }
        }
        Self {
            plugin_bindings,
            popup_bindings,
            functionality_map,
        }
    }
}

impl Plugin for Telescope {
    fn display_details(&self) -> String {
        "Telescope".to_string()
    }

    fn get_plugin_bindings(&self) -> HashMap<(Mode, Vec<KeyEvent>), String> {
        self.plugin_bindings.clone()
    }
    fn get_popup_bindings(&self) -> HashMap<(Mode, Vec<KeyEvent>), String> {
        self.popup_bindings.clone()
    }

    fn get_functionality_map(&self) -> HashMap<String, Action> {
        self.functionality_map.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelescopeWindow {
    keymap: HashMap<(Mode, Vec<KeyEvent>), Action>,
    telescope_backend: TelescopeBackend,
    current_sequence: Vec<KeyEvent>,
    pub should_quit: bool,
}

impl TelescopeWindow {
    pub fn new_sfs(ctx: AppContext, keymap: HashMap<(Mode, Vec<KeyEvent>), Action>) -> Self {
        TelescopeWindow {
            keymap,
            telescope_backend: TelescopeBackend::new_sfs(ctx),
            current_sequence: Vec::new(),
            should_quit: false,
        }
    }

    fn update_self_query(&mut self) {
        let query = self.get_search_query();
        self.update_search_query(query);
    }
}
impl PluginPopUp for TelescopeWindow {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(Clear, area);
        self.telescope_backend.draw(frame, area)?;
        Ok(())
    }

    fn confirm_result(&mut self) -> Option<Action> {
        self.telescope_backend.confirm_result()
    }

    fn next_result(&mut self) {
        self.telescope_backend.next_result();
    }

    fn previous_result(&mut self) {
        self.telescope_backend.previous_result();
    }

    fn update_search_query(&mut self, query: String) {
        self.telescope_backend.update_search_query(query);
    }

    fn push_search_char(&mut self, ch: char) {
        self.telescope_backend.query.append_char(ch);
        self.update_self_query();
    }

    fn drop_search_char(&mut self) {
        self.telescope_backend.query.drop_char();
        self.update_self_query();
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }

    fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn erase_text(&mut self) {
        self.telescope_backend.query.clear_contents();
        self.update_self_query();
    }

    fn get_search_query(&self) -> String {
        self.telescope_backend.query.get_contents()
    }

    fn display_details(&self) -> String {
        "Telescope".to_string()
    }

    fn get_default_action(&self) -> Box<fn(KeyEvent) -> Option<Action>> {
        Box::new(default_popup_action)
    }
    fn get_own_keymap(&self) -> HashMap<(Mode, Vec<KeyEvent>), Action> {
        self.keymap.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use super::*;

    #[test]
    fn test_new_telescope() {
        let mut custom_bindings = HashMap::new();
        insert_binding!(custom_bindings, Mode::PopUp, "abcd", "TelescopeQuit");
        let telescope = Telescope::new(custom_bindings.clone());
        let obtained_bindings = telescope.get_all_bindings();
        let mut expected_bindings = get_default_bindings();
        expected_bindings.extend(custom_bindings);
        assert_eq!(obtained_bindings, expected_bindings);
    }

    #[test]
    fn test_confirm_result() {
        let mut app = App::new().unwrap();
        let ctx = app.get_app_context();
        let mut sfs = TelescopeBackend::new_sfs(ctx);
        sfs.update_search_query("folder".to_string());
        sfs.table_state.select(Some(1));
        let resulting_action = sfs.confirm_result();
        //Get the root folder
        let current_path = env::current_dir().unwrap();
        let expected_path = current_path.join("tests/folder_2");
        let expected_action = Some(Action::AppAct(AppAction::ShowInFolder(expected_path)));
        assert_eq!(resulting_action, expected_action);
    }
}
