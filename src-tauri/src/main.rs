use image::GenericImageView;
use std::path::Path;
use tauri::Manager;
use image::GenericImageView;

#[tauri::command]
fn crop_image(path: String, x: u32, y: u32, w: u32, h: u32) -> Result<(), String> {
    match image::open(&Path::new(&path)) {
        Ok(mut img) => {
            let img_w = img.width();
            let img_h = img.height();
            let w = w.min(img_w);
            let h = h.min(img_h);
            let x = x.min(img_w - 1);
            let y = y.min(img_h - 1);

            let cropped = img.crop_imm(x, y, w, h);
            cropped.save(&path).map_err(|e| format!("Failed to save: {}", e))?;
            Ok(())
        }
        Err(e) => Err(format!("Failed to open image: {}", e))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![crop_image])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
