mod export;
mod images;
mod project;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            images::prepare_image,
            project::save_project,
            project::load_project,
            export::export_zine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
