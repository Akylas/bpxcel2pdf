#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{
  menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
  Emitter,
};

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_window_state::Builder::new().build())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      let handle = app.handle();

      #[allow(unused_mut)]
      let mut menu_builder = MenuBuilder::new(app);

      #[cfg(target_os = "macos")]
      {
        use tauri::menu::AboutMetadataBuilder;
        let app_menu = SubmenuBuilder::new(handle, &handle.package_info().name)
          .about(Some(AboutMetadataBuilder::new().build()))
          .separator()
          .services()
          .separator()
          .hide()
          .hide_others()
          .show_all()
          .separator()
          .quit()
          .build()?;
        menu_builder = menu_builder.item(&app_menu);
      }

      let open_item = MenuItemBuilder::with_id("open", "Open...")
        .accelerator("CmdOrControl+O")
        .build(handle)?;
      let print_item = MenuItemBuilder::with_id("print", "Print...")
        .accelerator("CmdOrControl+P")
        .build(handle)?;
      let file_menu = SubmenuBuilder::new(handle, "File")
        .item(&open_item)
        .item(&print_item)
        .build()?;

      // undo/redo are only natively handled on macOS; skip them on other platforms
      // to avoid blank or no-op menu entries (muda GTK/Windows don't implement them)
      #[cfg(target_os = "macos")]
      let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .build()?;
      #[cfg(not(target_os = "macos"))]
      let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .build()?;

      // fullscreen() predefined item is macOS-only in muda; use a custom item on all
      // platforms so the View menu is never empty and the action always works.
      let fullscreen_accel = if cfg!(target_os = "macos") {
        "Ctrl+Meta+F"
      } else {
        "F11"
      };
      let fullscreen_item = MenuItemBuilder::with_id("fullscreen", "Toggle Full Screen")
        .accelerator(fullscreen_accel)
        .build(handle)?;
      let view_menu = SubmenuBuilder::new(handle, "View")
        .item(&fullscreen_item)
        .build()?;

      // minimize/maximize/close_window predefined items are not supported on Linux (GTK)
      // and fullscreen is not handled on Windows — use custom items on all platforms so
      // the Window menu always has visible, working entries.
      let minimize_item = MenuItemBuilder::with_id("minimize", "Minimize")
        .accelerator("CmdOrControl+M")
        .build(handle)?;
      let maximize_label = if cfg!(target_os = "macos") {
        "Zoom"
      } else {
        "Maximize"
      };
      let maximize_item =
        MenuItemBuilder::with_id("maximize", maximize_label).build(handle)?;
      let close_item = MenuItemBuilder::with_id("close_window", "Close Window")
        .accelerator("CmdOrControl+W")
        .build(handle)?;
      let window_menu = SubmenuBuilder::new(handle, "Window")
        .item(&minimize_item)
        .item(&maximize_item)
        .item(&close_item)
        .build()?;

      let learn_more_item =
        MenuItemBuilder::with_id("learn_more", "Learn More").build(handle)?;
      let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(&learn_more_item)
        .build()?;

      let menu = menu_builder
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&window_menu)
        .item(&help_menu)
        .build()?;

      app.set_menu(menu)?;

      app.on_menu_event(move |app, event| {
        let _ = app.emit("menu", event.id().as_ref());
      });

      tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::default())
        .min_inner_size(400.0, 200.0)
        .build()?;

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
