use native_windows_gui as nwg;
use serde::Deserialize;
use std::{cell::RefCell, fs, process::Command, rc::Rc};

#[derive(Debug, Deserialize)]
struct App {
    name: String,
    exec: String,
}

#[derive(Debug, Deserialize)]
struct Category {
    name: String,
    icon: Option<String>,
    apps: Vec<App>,
}

#[derive(Debug, Deserialize)]
struct AppData {
    categories: Vec<Category>,
}

#[derive(Default)]
pub struct LauncherUi {
    window: nwg::Window,
    category_label: nwg::Label,
    category_list: nwg::ListBox<String>,
    app_label: nwg::Label,
    app_list: nwg::ListBox<String>,
    launch_btn: nwg::Button,
    data: Rc<RefCell<Option<AppData>>>,
}

impl LauncherUi {
    fn update_apps(&self) {
        let idx = self.category_list.selection();
        self.app_list.clear();
        if let (Some(data), Some(idx)) = (&*self.data.borrow(), idx) {
            if let Some(cat) = data.categories.get(idx) {
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
                // Only update apps if the category list triggered the event
                if *handle == ui.category_list.handle {
                    ui.update_apps();
                }
            }
            E::OnButtonClick | E::OnListBoxDoubleClick => {
                let cat_idx = ui.category_list.selection();
                let app_idx = ui.app_list.selection();
                if let (Some(data), Some(cat_idx), Some(app_idx)) =
                    (&*ui.data.borrow(), cat_idx, app_idx)
                {
                    if let Some(cat) = data.categories.get(cat_idx) {
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
        .size((500, 350))
        .position((300, 300))
        .title("Rust Windows Launcher")
        .build(&mut ui.window)
        .unwrap();

    nwg::Label::builder()
        .text("Categories")
        .parent(&ui.window)
        .position((20, 10))
        .size((120, 20))
        .build(&mut ui.category_label)
        .unwrap();

    nwg::ListBox::builder()
        .parent(&ui.window)
        .position((20, 35))
        .size((180, 220))
        .build(&mut ui.category_list)
        .unwrap();

    nwg::Label::builder()
        .text("Applications")
        .parent(&ui.window)
        .position((220, 10))
        .size((200, 20))
        .build(&mut ui.app_label)
        .unwrap();

    nwg::ListBox::builder()
        .parent(&ui.window)
        .position((220, 35))
        .size((250, 220))
        .build(&mut ui.app_list)
        .unwrap();

    nwg::Button::builder()
        .text("Launch")
        .parent(&ui.window)
        .position((20, 270))
        .size((450, 40))
        .build(&mut ui.launch_btn)
        .unwrap();

    // Load data
    let json = fs::read_to_string("apps.json")
        .or_else(|_| fs::read_to_string("src/apps.json"))
        .unwrap();
    let data: AppData = serde_json::from_str(&json).unwrap();
    for cat in &data.categories {
        ui.category_list
            .insert(ui.category_list.len(), cat.name.clone());
    }
    ui.data.replace(Some(data));
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
