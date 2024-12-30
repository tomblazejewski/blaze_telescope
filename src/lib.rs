use color_eyre::eyre::Result;
use ratatui::{crossterm::event::KeyEvent, layout::Rect, widgets::Clear, Frame};
use std::collections::HashMap;

use blaze_explorer_core::plugin::Plugin;

use blaze_explorer_core::{
    action::{Action, AppAction},                        //needs to be a lib
    app::App,                                           //needs to be a lib
    custom_action,                                      //needs to be a lib
    input_machine::{InputMachine, KeyProcessingResult}, //needs to be a lib
    line_entry::LineEntry,                              //needs to be a lib
    mode::Mode,                                         //needs to be a lib
    simple_input_machine::TelescopeInputMachine,        //this itself needs to be part of the plugin
    telescope::{AppContext, PopUpComponent, TelescopeBackend},
};

use blaze_explorer_core::plugin::plugin_popup::PluginPopUp;

//The plugin consists of the following parts
//Struct Telescope - defines functionalities available at the app level. The app can bind any of
//these actions to a keymap to use it. Telescope implements Plugin.
//Struct TelescopeWindow (implementing PluginPopUp) - this is spawned upon calling one of the
//plugin's functionalities and takes control of incoming KeyEvents
//

type CustomAction = fn(&mut App) -> Option<Action>;
type BoxedAction = Box<CustomAction>;
pub fn open_sfs(app: &mut App) -> Option<Action> {
    let ctx = app.get_app_context();
    let popup = Box::new(TelescopeWindow::new_sfs(ctx));
    app.attach_popup(popup);

    None
}
#[derive(Debug, Clone)]
pub struct Telescope {
    bindings_map: HashMap<(Mode, Vec<KeyEvent>), String>,
    functionality_map: HashMap<String, Action>,
}

impl Telescope {
    pub fn new(bindings_map: HashMap<(Mode, Vec<KeyEvent>), String>) -> Self {
        let mut functionality_map = HashMap::new();
        functionality_map.insert("OpenSFS".to_string(), custom_action!(open_sfs));

        Self {
            bindings_map,
            functionality_map,
        }
    }
}

impl Plugin for Telescope {
    fn display_details(&self) -> String {
        "Telescope".to_string()
    }

    fn attach_functionality(&self, _app: &mut App) -> HashMap<String, BoxedAction> {
        let mut map = HashMap::new();
        map.insert("OpenSFS".to_string(), Box::new(open_sfs as CustomAction));
        map
    }

    fn get_bindings(&self) -> HashMap<(Mode, Vec<KeyEvent>), Action> {
        let mut output_map = HashMap::new();
        for (key, value) in &self.bindings_map {
            match self.functionality_map.get(value) {
                Some(action) => {
                    output_map.insert((*key).clone(), action.clone());
                }
                None => {}
            }
        }
        output_map
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelescopeWindow {
    input_machine: TelescopeInputMachine,
    telescope_backend: TelescopeBackend,
    current_sequence: Vec<KeyEvent>,
    pub should_quit: bool,
}

impl TelescopeWindow {
    pub fn new_sfs(ctx: AppContext) -> Self {
        TelescopeWindow {
            input_machine: TelescopeInputMachine::new(),
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
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        let keymap_result =
            self.input_machine
                .process_keys(&Mode::Normal, &mut self.current_sequence, key_event);
        match keymap_result {
            KeyProcessingResult::Complete(action) => {
                return Some(action);
            }
            KeyProcessingResult::Invalid => {
                return self
                    .input_machine
                    .get_default_action(&Mode::Normal, key_event)
            }
            _ => {}
        }
        None
    }

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
}
