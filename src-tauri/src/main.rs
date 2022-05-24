use std::env;

use tauri::{
  AboutMetadata, CustomMenuItem, Menu, MenuEntry, MenuItem, Submenu, WindowBuilder, WindowUrl,
};

// the payload type must implement `Serialize`.
#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

fn main() {
  let ctx = tauri::generate_context!();
  let menu = Menu::new()
    .add_submenu(Submenu::new(
      &ctx.package_info().name,
      Menu::with_items([
        MenuItem::About(ctx.package_info().name.clone(), AboutMetadata::new()).into(),
        MenuItem::Separator.into(),
        MenuItem::Services.into(),
        MenuItem::Separator.into(),
        MenuItem::Hide.into(),
        MenuItem::HideOthers.into(),
        MenuItem::ShowAll.into(),
        MenuItem::Separator.into(),
        MenuItem::Quit.into(),
      ]),
    ))
    .add_submenu(Submenu::new(
      "File",
      Menu::with_items([
        CustomMenuItem::new("open", "Open...")
          .accelerator("CmdOrControl+O")
          .into(),
        CustomMenuItem::new("print", "Print...")
          .accelerator("CmdOrControl+P")
          .into(),
      ]),
    ))
    .add_submenu(Submenu::new(
      "Edit",
      Menu::with_items([
        MenuItem::Undo.into(),
        MenuItem::Redo.into(),
        MenuItem::Separator.into(),
        MenuItem::Cut.into(),
        MenuItem::Copy.into(),
        MenuItem::Paste.into(),
        #[cfg(not(target_os = "macos"))]
        MenuItem::Separator.into(),
        MenuItem::SelectAll.into(),
      ]),
    ))
    .add_submenu(Submenu::new(
      "View",
      Menu::with_items([MenuItem::EnterFullScreen.into()]),
    ))
    .add_submenu(Submenu::new(
      "Window",
      Menu::with_items([
        MenuItem::Minimize.into(),
        MenuItem::Zoom.into(),
        MenuItem::CloseWindow.into(),
      ]),
    ))
    .add_submenu(Submenu::new(
      // You should always have a Help menu on macOS because it will automatically
      // show a menu search field
      "Help",
      Menu::with_items([CustomMenuItem::new("learn_more", "Learn More").into()]),
    ));
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![])
    .plugin(tauri_plugin_window_state::Builder::default().build())
    .menu(menu)
    .setup(|app| {
      WindowBuilder::new(app, "main", WindowUrl::default())
        // .resizable(true)
        // .decorations(true)
        // .always_on_top(false)
        // .inner_size(1400.0, 850.0)
        .min_inner_size(400.0, 200.0)
        // .skip_taskbar(false)
        // .fullscreen(false)
        .build()?;
      Ok(())
    })
    .run(ctx)
    .expect("error while running tauri application");
}
