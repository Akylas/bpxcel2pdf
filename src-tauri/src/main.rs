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

      let view_menu = SubmenuBuilder::new(handle, "View")
        .fullscreen()
        .build()?;

      let window_menu = SubmenuBuilder::new(handle, "Window")
        .minimize()
        .maximize()
        .close_window()
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
