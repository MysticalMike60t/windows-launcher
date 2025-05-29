use native_windows_gui as nwg;
use serde::Deserialize;
use std::{cell::RefCell, fs, process::Command, rc::Rc};

#[derive(Debug, Deserialize, Clone)]
struct App {
    name: String,
    exec: String,
    icon: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Category {
    name: String,
    icon: Option<String>,
    apps: Vec<App>,
}

#[derive(Debug, Deserialize, Clone)]
struct AppData {
    categories: Vec<Category>,
}

#[derive(Default)]
pub struct LauncherUi {
    window: nwg::Window,
    search_label: nwg::Label,
    search_box: nwg::TextInput,
    category_label: nwg::Label,
    category_list: nwg::ListBox<String>,
    app_label: nwg::Label,
    app_list: nwg::ListBox<String>,
    data: Rc<RefCell<Option<AppData>>>,
    filtered_categories: Rc<RefCell<Vec<Category>>>,
}

impl LauncherUi {
    fn update_categories(&self, filter: &str) {
        let data = self.data.borrow();
        self.category_list.clear();
        self.filtered_categories.borrow_mut().clear();

        if let Some(data) = &*data {
            for cat in &data.categories {
                if filter.is_empty()
                    || cat.name.to_lowercase().contains(filter)
                    || cat
                        .apps
                        .iter()
                        .any(|app| app.name.to_lowercase().contains(filter))
                {
                    self.category_list
                        .insert(self.category_list.len(), cat.name.clone());
                    self.filtered_categories.borrow_mut().push(cat.clone());
                }
            }
        }
        self.category_list.set_selection(Some(0));
        self.update_apps();
    }

    fn update_apps(&self) {
        let idx = self.category_list.selection();
        self.app_list.clear();
        if let Some(idx) = idx {
            let filtered = self.filtered_categories.borrow();
            if let Some(cat) = filtered.get(idx) {
                for app in &cat.apps {
                    self.app_list.insert(self.app_list.len(), app.name.clone());
                }
            }
        }
    }
}

mod events {
    use super::*;
    use nwg::Event as E;

    pub fn handle(ui: &LauncherUi, evt: nwg::Event, handle: &nwg::ControlHandle) {
        match evt {
            E::OnListBoxSelect => {
                if *handle == ui.category_list.handle {
                    ui.update_apps();
                }
            }
            E::OnTextInput => {
                if *handle == ui.search_box.handle {
                    let filter = ui.search_box.text().to_lowercase();
                    ui.update_categories(&filter);
                }
            }
            E::OnListBoxDoubleClick => {
                let cat_idx = ui.category_list.selection();
                let app_idx = ui.app_list.selection();
                if let (Some(cat_idx), Some(app_idx)) = (cat_idx, app_idx) {
                    let filtered = ui.filtered_categories.borrow();
                    if let Some(cat) = filtered.get(cat_idx) {
                        if let Some(app) = cat.apps.get(app_idx) {
                            let _ = Command::new("cmd").args(["/C", &app.exec]).spawn();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").unwrap();

    let mut ui = LauncherUi::default();

    nwg::Window::builder()
        .size((700, 450))
        .position((300, 200))
        .title("Rust Windows Launcher")
        .build(&mut ui.window)
        .unwrap();

    nwg::Label::builder()
        .text("Search:")
        .parent(&ui.window)
        .position((20, 15))
        .size((60, 20))
        .build(&mut ui.search_label)
        .unwrap();

    nwg::TextInput::builder()
        .parent(&ui.window)
        .position((85, 12))
        .size((160, 28))
        .build(&mut ui.search_box)
        .unwrap();

    nwg::Label::builder()
        .text("Categories")
        .parent(&ui.window)
        .position((20, 50))
        .size((120, 20))
        .build(&mut ui.category_label)
        .unwrap();

    nwg::ListBox::builder()
        .parent(&ui.window)
        .position((20, 75))
        .size((180, 320))
        .build(&mut ui.category_list)
        .unwrap();

    nwg::Label::builder()
        .text("Applications")
        .parent(&ui.window)
        .position((220, 50))
        .size((200, 20))
        .build(&mut ui.app_label)
        .unwrap();

    nwg::ListBox::builder()
        .parent(&ui.window)
        .position((220, 75))
        .size((450, 320))
        .build(&mut ui.app_list)
        .unwrap();

    // Load data
    let json = fs::read_to_string("apps.json")
        .or_else(|_| fs::read_to_string("src/apps.json"))
        .unwrap();
    let data: AppData = serde_json::from_str(&json).unwrap();
    ui.data.replace(Some(data.clone()));
    ui.filtered_categories.borrow_mut().clear();
    ui.filtered_categories
        .borrow_mut()
        .extend(data.categories.clone());
    for cat in &data.categories {
        ui.category_list
            .insert(ui.category_list.len(), cat.name.clone());
    }
    ui.category_list.set_selection(Some(0));
    ui.update_apps();

    let ui_rc = Rc::new(ui);

    let ui_events = ui_rc.clone();
    nwg::bind_event_handler(&ui_events.window.handle, &ui_events.window.handle, {
        let ui_events = ui_events.clone();
        move |evt, _evt_data, handle| {
            events::handle(&ui_events, evt, &handle);
        }
    });

    nwg::dispatch_thread_events();
}
