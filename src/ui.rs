use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GBox, Button, DropDown, Label,
    ListBox, ListBoxRow, Notebook, Orientation, PolicyType, ScrolledWindow,
    SelectionMode, Separator, StringList, StringObject,
};
use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ctl;

// ── Shared log helper ─────────────────────────────────────────────────────────

// We store the TextBuffer directly (not behind RefCell) since GObjects are
// already reference-counted internally. Rc<TextBuffer> lets us clone cheaply.
type LogBuf = Rc<gtk4::TextBuffer>;

fn log(buf: &LogBuf, msg: &str) {
    let mut end = buf.end_iter();
    buf.insert(&mut end, &format!("{}\n", msg));
}

// ── Library page ──────────────────────────────────────────────────────────────

fn build_library_page(log_buf: LogBuf) -> GBox {
    let vbox = GBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let heading = Label::new(Some("Available Prototypes"));
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    vbox.append(&heading);

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("boxed-list");

    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));
    vbox.append(&scroll);

    let status = Label::new(Some(""));
    status.set_xalign(0.0);
    status.add_css_class("dim-label");
    vbox.append(&status);

    let btn_bar = GBox::new(Orientation::Horizontal, 8);
    let refresh_btn = Button::with_label("Refresh");
    let spawn_50_btn = Button::with_label("Spawn 50 Random");
    let spawn_random_btn = Button::with_label("Spawn Random");
    spawn_random_btn.add_css_class("suggested-action");
    btn_bar.append(&refresh_btn);
    btn_bar.append(&spawn_50_btn);
    btn_bar.append(&spawn_random_btn);
    vbox.append(&btn_bar);

    // Build the refresh closure — clones of Rc are cheap
    let do_refresh: Rc<dyn Fn()> = Rc::new({
        let list = list.clone();
        let status = status.clone();
        let log_buf = log_buf.clone();

        move || {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            log(&log_buf, "$ shimejictl prototypes list");

            match ctl::list_prototypes() {
                Err(e) => status.set_text(&format!("Error: {}", e)),
                Ok(names) if names.is_empty() => {
                    status.set_text("No prototypes found. Import a .wlshm first.");
                }
                Ok(names) => {
                    status.set_text(&format!("{} prototype(s) available", names.len()));
                    log(&log_buf, &format!("  → {} prototype(s)", names.len()));
                    for name in &names {
                        let row = library_row(name, log_buf.clone());
                        list.append(&row);
                    }
                }
            }
        }
    });

    {
        let do_refresh = do_refresh.clone();
        refresh_btn.connect_clicked(move |_| do_refresh());
    }
    {
        let do_refresh = do_refresh.clone();
        vbox.connect_map(move |_| do_refresh());
    }
    {
        let log_buf = log_buf.clone();
        spawn_random_btn.connect_clicked(move |_| {
            match ctl::list_prototypes() {
                Ok(names) if !names.is_empty() => {
                    let mut rng = rand::thread_rng();
                    if let Some(name) = names.choose(&mut rng) {
                        log(&log_buf, &format!("$ shimejictl mascot spawn {}", name));
                        match ctl::spawn_mascot(name) {
                            Ok(()) => log(&log_buf, &format!("  → spawned {}", name)),
                            Err(e) => {
                                log(&log_buf, &format!("  ✗ {}", e));
                                eprintln!("spawn error: {}", e);
                            }
                        }
                    }
                }
                Ok(_) => log(&log_buf, "  ✗ no prototypes available"),
                Err(e) => {
                    log(&log_buf, &format!("  ✗ {}", e));
                    eprintln!("list error: {}", e);
                }
            }
        });
    }
    {
        let log_buf = log_buf.clone();
        spawn_50_btn.connect_clicked(move |_| {
            match ctl::list_prototypes() {
                Ok(names) if !names.is_empty() => {
                    let mut rng = rand::thread_rng();
                    log(&log_buf, "$ spawning 50 random prototypes");
                    let mut spawned = 0;
                    let mut failed = 0;
                    for _ in 0..50 {
                        if let Some(name) = names.choose(&mut rng) {
                            match ctl::spawn_mascot(name) {
                                Ok(()) => spawned += 1,
                                Err(e) => {
                                    failed += 1;
                                    eprintln!("spawn error: {}", e);
                                }
                            }
                        }
                    }
                    log(&log_buf, &format!("  → spawned {} (failed {})", spawned, failed));
                }
                Ok(_) => log(&log_buf, "  ✗ no prototypes available"),
                Err(e) => {
                    log(&log_buf, &format!("  ✗ {}", e));
                    eprintln!("list error: {}", e);
                }
            }
        });
    }

    vbox
}

fn library_row(name: &str, log_buf: LogBuf) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_activatable(false);

    let hbox = GBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);

    let lbl = Label::new(Some(name));
    lbl.set_xalign(0.0);
    lbl.set_hexpand(true);
    hbox.append(&lbl);

    let spawn_50_btn = Button::with_label("Spawn 50");
    {
        let log_buf = log_buf.clone();
        let name_owned = name.to_string();
        spawn_50_btn.connect_clicked(move |_| {
            log(&log_buf, &format!("$ spawning 50x {}", name_owned));
            let mut spawned = 0;
            let mut failed = 0;
            for _ in 0..50 {
                match ctl::spawn_mascot(&name_owned) {
                    Ok(()) => spawned += 1,
                    Err(e) => {
                        failed += 1;
                        eprintln!("spawn error: {}", e);
                    }
                }
            }
            log(&log_buf, &format!("  → spawned {} (failed {})", spawned, failed));
        });
    }
    hbox.append(&spawn_50_btn);

    let btn = Button::with_label("Spawn");
    btn.add_css_class("suggested-action");

    let name_owned = name.to_string();
    btn.connect_clicked(move |_| {
        log(&log_buf, &format!("$ shimejictl mascot spawn {}", name_owned));
        match ctl::spawn_mascot(&name_owned) {
            Ok(()) => {
                log(&log_buf, &format!("  → spawned {}", name_owned));
            }
            Err(e) => {
                log(&log_buf, &format!("  ✗ {}", e));
                eprintln!("spawn error: {}", e);
            }
        }
    });

    hbox.append(&btn);
    row.set_child(Some(&hbox));
    row
}

// ── Active page ───────────────────────────────────────────────────────────────

fn build_active_page(log_buf: LogBuf) -> GBox {
    let vbox = GBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let heading = Label::new(Some("Active mascots"));
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    vbox.append(&heading);

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("boxed-list");

    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));
    vbox.append(&scroll);

    let status = Label::new(Some(""));
    status.set_xalign(0.0);
    status.add_css_class("dim-label");
    vbox.append(&status);

    let btn_bar = GBox::new(Orientation::Horizontal, 8);
    let refresh_btn = Button::with_label("Refresh");
    let dismiss_all_btn = Button::with_label("Dismiss all");
    dismiss_all_btn.add_css_class("destructive-action");
    btn_bar.append(&refresh_btn);
    btn_bar.append(&dismiss_all_btn);
    vbox.append(&btn_bar);

    let do_refresh: Rc<dyn Fn()> = Rc::new({
        let list = list.clone();
        let status = status.clone();
        let log_buf = log_buf.clone();

        move || {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            log(&log_buf, "$ shimejictl mascot list");

            match ctl::list_active() {
                Err(e) => status.set_text(&format!("Error: {}", e)),
                Ok(mascots) if mascots.is_empty() => {
                    status.set_text("No mascots active.");
                }
                Ok(mascots) => {
                    status.set_text(&format!("{} mascot(s) active", mascots.len()));
                    for m in &mascots {
                        let row = active_row(m, log_buf.clone());
                        list.append(&row);
                    }
                }
            }
        }
    });

    {
        let do_refresh = do_refresh.clone();
        refresh_btn.connect_clicked(move |_| do_refresh());
    }
    {
        let do_refresh = do_refresh.clone();
        let log_buf = log_buf.clone();
        dismiss_all_btn.connect_clicked(move |_| {
            log(&log_buf, "$ shimejictl mascot dismiss --all");
            match ctl::dismiss_all() {
                Ok(()) => { log(&log_buf, "  → all dismissed"); do_refresh(); }
                Err(e) => { log(&log_buf, &format!("  ✗ {}", e)); eprintln!("{}", e); }
            }
        });
    }
    {
        let do_refresh = do_refresh.clone();
        vbox.connect_map(move |_| do_refresh());
    }

    vbox
}

fn active_row(mascot: &ctl::ActiveMascot, log_buf: LogBuf) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_activatable(false);

    let hbox = GBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(8);
    hbox.set_margin_bottom(8);
    hbox.set_margin_start(8);
    hbox.set_margin_end(8);

    let id_lbl = Label::new(Some(&format!("#{}", mascot.id)));
    id_lbl.add_css_class("dim-label");
    hbox.append(&id_lbl);

    let name_lbl = Label::new(Some(&mascot.name));
    name_lbl.set_xalign(0.0);
    name_lbl.set_hexpand(true);
    hbox.append(&name_lbl);

    let btn = Button::with_label("Dismiss");
    btn.add_css_class("destructive-action");

    let id = mascot.id;
    let name = mascot.name.clone();
    let row_ref = row.clone();

    btn.connect_clicked(move |_| {
        log(&log_buf, &format!("$ shimejictl mascot dismiss --id {}", id));
        match ctl::dismiss_mascot(id) {
            Ok(()) => {
                log(&log_buf, &format!("  → dismissed {} #{}", name, id));
                if let Some(parent) = row_ref.parent() {
                    if let Ok(lb) = parent.downcast::<ListBox>() {
                        lb.remove(&row_ref);
                    }
                }
            }
            Err(e) => {
                log(&log_buf, &format!("  ✗ {}", e));
                eprintln!("dismiss error: {}", e);
            }
        }
    });

    hbox.append(&btn);
    row.set_child(Some(&hbox));
    row
}

// ── Spawn page ────────────────────────────────────────────────────────────────

fn build_spawn_page(log_buf: LogBuf) -> GBox {
    let vbox = GBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(24);
    vbox.set_margin_bottom(24);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);
    vbox.set_valign(gtk4::Align::Center);
    vbox.set_vexpand(true);

    let heading = Label::new(Some("Spawn a shimeji"));
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    vbox.append(&heading);

    let desc = Label::new(Some("Choose a prototype and click Spawn."));
    desc.set_xalign(0.0);
    desc.add_css_class("dim-label");
    vbox.append(&desc);

    vbox.append(&Separator::new(Orientation::Horizontal));

    // Slot that holds the current DropDown widget
    let dropdown_slot = GBox::new(Orientation::Vertical, 0);
    dropdown_slot.set_hexpand(true);
    vbox.append(&dropdown_slot);

    // Shared handle to the current DropDown so the spawn button can read it
    let current_dropdown: Rc<RefCell<Option<DropDown>>> = Rc::new(RefCell::new(None));

    let btn_bar = GBox::new(Orientation::Horizontal, 8);
    btn_bar.set_halign(gtk4::Align::End);
    let spawn_btn = Button::with_label("Spawn");
    spawn_btn.add_css_class("suggested-action");
    btn_bar.append(&spawn_btn);
    vbox.append(&btn_bar);

    // Closure that (re)builds the dropdown from installed prototypes
    let rebuild = {
        let dropdown_slot = dropdown_slot.clone();
        let current_dropdown = current_dropdown.clone();
        move || {
            while let Some(child) = dropdown_slot.first_child() {
                dropdown_slot.remove(&child);
            }
            let names: Vec<String> = ctl::list_prototypes().unwrap_or_default();
            let items: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            let string_list = StringList::new(&items);
            let dd = DropDown::new(Some(string_list), gtk4::Expression::NONE);
            dd.set_hexpand(true);
            dropdown_slot.append(&dd);
            *current_dropdown.borrow_mut() = Some(dd);
        }
    };

    // Build once now, rebuild whenever tab becomes visible
    rebuild();
    vbox.connect_map(move |_| rebuild());

    spawn_btn.connect_clicked(move |_| {
        let dd_opt = current_dropdown.borrow();
        let dd = match dd_opt.as_ref() {
            Some(d) => d.clone(),
            None => return,
        };
        drop(dd_opt);

        let name = match dd
            .selected_item()
            .and_then(|o| o.downcast::<StringObject>().ok())
        {
            Some(s) => s.string().to_string(),
            None => { eprintln!("no prototype selected"); return; }
        };

        log(&log_buf, &format!("$ shimejictl mascot spawn {}", name));
        match ctl::spawn_mascot(&name) {
            Ok(()) => {
                log(&log_buf, &format!("  → spawned {}", name));
            }
            Err(e) => {
                log(&log_buf, &format!("  ✗ {}", e));
                eprintln!("spawn error: {}", e);
            }
        }
    });

    vbox
}

// ── Log page ──────────────────────────────────────────────────────────────────

fn build_log_page(log_buf: LogBuf) -> GBox {
    let vbox = GBox::new(Orientation::Vertical, 8);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);

    let heading = Label::new(Some("Command log"));
    heading.set_xalign(0.0);
    heading.add_css_class("heading");
    vbox.append(&heading);

    // TextBuffer is already an Rc-like GObject — pass directly, no borrow needed
    let text_view = gtk4::TextView::with_buffer(&*log_buf);
    text_view.set_editable(false);
    text_view.set_monospace(true);
    text_view.set_wrap_mode(gtk4::WrapMode::WordChar);

    let scroll = ScrolledWindow::new();
    scroll.set_policy(PolicyType::Never, PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&text_view));
    vbox.append(&scroll);

    // Auto-scroll to bottom on new log entries
    {
        let text_view = text_view.clone();
        log_buf.connect_changed(move |buf| {
            let mut end = buf.end_iter();
            text_view.scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
        });
    }

    let clear_btn = Button::with_label("Clear");
    {
        let log_buf = log_buf.clone();
        clear_btn.connect_clicked(move |_| log_buf.set_text(""));
    }
    vbox.append(&clear_btn);

    vbox
}

// ── Window ────────────────────────────────────────────────────────────────────

pub fn build_window(app: &Application) {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("Shimeji Manager")
        .default_width(520)
        .default_height(520)
        .build();

    // TextBuffer is a GObject — Rc wrapping it is safe and clone is O(1)
    let log_buf: LogBuf = Rc::new(gtk4::TextBuffer::new(None));
    let notebook = Rc::new(Notebook::new());
    notebook.set_tab_pos(gtk4::PositionType::Top);

    let lib_page = build_library_page(log_buf.clone());
    notebook.append_page(&lib_page, Some(&Label::new(Some("Library"))));

    let active_page = build_active_page(log_buf.clone());
    notebook.append_page(&active_page, Some(&Label::new(Some("Active"))));

    let spawn_page = build_spawn_page(log_buf.clone());
    notebook.append_page(&spawn_page, Some(&Label::new(Some("Spawn"))));

    let log_page = build_log_page(log_buf.clone());
    notebook.append_page(&log_page, Some(&Label::new(Some("Log"))));

    log(&log_buf, "Shimeji Manager ready.");
    log(&log_buf, "Make sure wl_shimeji is running before spawning.");

    win.set_child(Some(notebook.as_ref()));
    win.present();
}