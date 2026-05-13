mod commands;
mod error;
mod library;

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
            commands::library_root,
            commands::library_set_root,
            commands::library_list,
            commands::library_list_roms,
            commands::library_import_rom,
            commands::library_rom_path,
            commands::library_record,
            commands::library_verify,
            commands::library_reapply,
            commands::library_reveal,
            commands::library_delete_entry,
            commands::library_delete_rom,
            commands::library_lookup_by_patch_hash,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
