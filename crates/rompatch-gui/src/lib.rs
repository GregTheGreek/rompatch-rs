mod commands;
mod error;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_patch_format,
            commands::describe_patch,
            commands::detect_rom_header,
            commands::compute_hashes,
            commands::apply_patch,
            commands::default_output_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
