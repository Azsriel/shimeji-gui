mod ctl;
mod ui;

use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "com.example.shimeji-gui";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        ui::build_window(app);
    });

    app.run();
}
