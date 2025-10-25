use std::fs::{create_dir_all, write};
use image::{DynamicImage, ImageFormat};
use serde::Serialize;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use crate::models::pack_list::PackList;
use crate::models::image_data::ImageData;
use crate::core::cropper;

// Load in the default icon to bianary so the file is contained in the executable
const DEFAULT_ICON: &[u8] = include_bytes!("../../assets/icon.png");

// The Painting struct is now private to this module.
#[derive(Serialize)]
struct Painting {
    id:         String,
    filename:   String,
    name:       String,
    artist:     String,
    width:      u32,
    height:     u32
}

/*
This creates Base64 previews from a Vec<DynamicImage> for the Tauri frontend.
The images are passed in directly and are not retrieved from app state.
*/
pub fn generate_base64_previews(image_list: &Vec<DynamicImage>) -> Vec<String> {
    let mut base64_images = Vec::new(); // Create a vector to store the Base64 strings

    for preview_image in image_list {
        let mut image_buffer: Vec<u8> = Vec::new();

        // Write the image's PNG data into our in-memory buffer
        preview_image.write_to(
            &mut Cursor::new(&mut image_buffer),
            ImageFormat::Png,
        ).expect("Failed to write image to buffer");
        
        // Encode the binary data into a Base64 string
        let base64_string = general_purpose::STANDARD.encode(&image_buffer);
        
        // Format the string as a Data URI and add it to our vector
        base64_images.push(format!("data:image/png;base64,{}", base64_string));
    }

    base64_images // Return the list of Data URIs
}

fn write_icon(export_path: &str) {
    write(format!("{}/icon.png", export_path), DEFAULT_ICON).expect("Failed to write default icon");
}
fn write_json (painting_list: &PackList<Painting>, export_path: &str) {
    let json_data = serde_json::to_string_pretty(painting_list).expect("Failed to serialize painting list");
    write(format!("{}/custompaintings.json", export_path), json_data).expect("Failed to write painting list JSON file");
}

// This new struct is used to package all necessary data for a single exportable image.
pub struct ExportItem {
    pub source_path: String,
    pub data: ImageData,
}


fn write_images(painting_list: &mut PackList<Painting>, image_list: Vec<ExportItem>, export_path: &str) {
    
    let images_dir = format!("{}/images", export_path);
    create_dir_all(&images_dir).expect("Failed to create images directory");

    for item in image_list {
        // Re-create the image from the source path on-demand for export and make it mutable.
        let mut painting = cropper::crop_single_image(&item.source_path, &item.data.image_size)
            .expect("Failed to re-crop image for export.");

        if painting.width() > 1024 {
            painting = painting.thumbnail(1024, u32::MAX);
        }

        for (width, height) in item.data.get_sizes() {

            let sanitized_id = item.data.id.as_ref().unwrap().replace(' ', "_");
            let sanitized_filename = item.data.filename.as_ref().unwrap().replace(' ', "_");

            let id: String = format!("{}_{}x{}", &sanitized_id, &width, &height);
            let base_filename: String = format!("{}_{}x{}", &sanitized_filename, &width, &height);
            
            let save_path = format!("{}/{}.png", &images_dir, &base_filename);
            painting.save(save_path).expect("This shouldnt fail");

            let painting_meta: Painting = Painting {
                id,
                filename: format!("{}.png", base_filename),
                name: item.data.name.clone().unwrap(),
                artist: item.data.artist.clone().unwrap(), 
                width: *width, 
                height: *height, 
            };
            painting_list.add_painting(painting_meta);
        };
    }
}


/*
This is the final export call. It now accepts the raw metadata components
and is responsible for creating the PackList<Painting> internally.
*/
pub fn export(
    pack_name: String,
    version: String,
    id: String,
    description: String,
    items_to_export: Vec<ExportItem>,
    export_path: &str,
) {
    // --- NEW: Sanitize Pack Name and ID ---
    // Sanitize the pack name for use in the directory path.
    let sanitized_pack_name = pack_name.replace(' ', "_");
    let pack_dir = format!("{}/{}", export_path, &sanitized_pack_name);

    let sanitized_pack_id: String = id
        .to_lowercase()
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
        .collect();

    let mut painting_list = PackList::new(
        pack_name,
        version,
        sanitized_pack_id,
        description,
    );

    write_images(&mut painting_list, items_to_export, &pack_dir);
    write_json(&painting_list, &pack_dir);
    write_icon(&pack_dir);
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::image_size::ImageSize;
    use image::RgbaImage;
    use std::{env, fs, path::PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    // --- Test Helper: TestImage ---
    // Creates a dummy image file for testing
    struct TestImage {
        path: PathBuf,
    }

    impl TestImage {
        fn new(path: &PathBuf) -> Self {
            let img = RgbaImage::new(800, 600); // 4:3 ratio
            img.save(path).expect("Failed to save test image");
            Self { path: path.clone() }
        }

        fn path_str(&self) -> String {
            self.path.to_str().expect("Path is not valid UTF-8").to_string()
        }
    }

    // --- Test Helper: Temporary Directory ---
    // Creates a unique temp directory for an integration test
    // and cleans it up when it goes out of scope.
    struct TempExportDir {
        path: PathBuf,
    }

    impl TempExportDir {
        fn new() -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_nanos();
            let rand_string = format!("{:x}", nanos);
            let path = env::temp_dir().join(format!("test_export_{}", rand_string));
            fs::create_dir_all(&path).expect("Failed to create temp dir");
            Self { path }
        }

        fn path_str(&self) -> String {
            self.path.to_str().expect("Path is not valid UTF-8").to_string()
        }
    }

    impl Drop for TempExportDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path); // Ignore cleanup errors
        }
    }

    #[test]
    fn test_generate_base64_previews() {
        let mut images: Vec<DynamicImage> = Vec::new();
        images.push(DynamicImage::ImageRgba8(RgbaImage::new(10, 10)));
        images.push(DynamicImage::ImageRgba8(RgbaImage::new(20, 20)));

        let previews = generate_base64_previews(&images);

        assert_eq!(previews.len(), 2);
        assert!(previews[0].starts_with("data:image/png;base64,"));
        assert!(previews[1].starts_with("data:image/png;base64,"));
        assert_ne!(previews[0], previews[1]); // Different images should have different base64
    }

    // --- Integration Test for export() ---

    #[test]
    fn test_export_integration() {
        // 1. Setup: Create temp directory and a test source image
        let temp_dir = TempExportDir::new();
        let test_img_path = temp_dir.path.join("source_image.png");
        let test_img = TestImage::new(&test_img_path);

        // 2. Setup: Create export metadata
        let pack_name = "My Test Pack".to_string();
        let version = "1.0.0".to_string();
        let id = "My Test ID!".to_string(); // Will be sanitized
        let description = "A pack for testing".to_string();
        
        let mut items_to_export: Vec<ExportItem> = Vec::new();

        // Create metadata for one "Square" crop
        let mut square_data = ImageData::new(ImageSize::Square);
        square_data.id = Some("Test Square".to_string());
        square_data.filename = Some("test_square_file".to_string());
        square_data.name = Some("My Square Painting".to_string());
        square_data.artist = Some("The Artist".to_string());
        
        items_to_export.push(ExportItem {
            source_path: test_img.path_str(),
            data: square_data,
        });

        // 3. Act: Call the export function
        export(
            pack_name.clone(),
            version.clone(),
            id.clone(),
            description.clone(),
            items_to_export,
            &temp_dir.path_str(),
        );

        // 4. Assert: Check if files and directories were created correctly
        let pack_dir = temp_dir.path.join("My_Test_Pack"); // Sanitized pack name
        assert!(pack_dir.exists() && pack_dir.is_dir());

        // Check for icon
        let icon_path = pack_dir.join("icon.png");
        assert!(icon_path.exists() && icon_path.is_file());

        // Check for images directory
        let images_dir = pack_dir.join("images");
        assert!(images_dir.exists() && images_dir.is_dir());

        // Check for generated images (Square has 4 sizes: 1x1, 2x2, 3x3, 4x4)
        // Example: test_square_file_1x1.png
        assert!(images_dir.join("test_square_file_1x1.png").exists());
        assert!(images_dir.join("test_square_file_2x2.png").exists());
        assert!(images_dir.join("test_square_file_3x3.png").exists());
        assert!(images_dir.join("test_square_file_4x4.png").exists());

        // Check for JSON file
        let json_path = pack_dir.join("custompaintings.json");
        assert!(json_path.exists() && json_path.is_file());

        // 5. Assert: Check JSON content
        let json_content = fs::read_to_string(json_path).expect("Failed to read JSON");
        
        // Simple string contains checks for key metadata
        assert!(json_content.contains(r#""name": "My Test Pack""#));
        assert!(json_content.contains(r#""version": "1.0.0""#));
        assert!(json_content.contains(r#""id": "my_test_id""#)); // Sanitized ID
        assert!(json_content.contains(r#""description": "A pack for testing""#));

        // Check for painting entries
        assert!(json_content.contains(r#""id": "Test_Square_1x1""#));
        assert!(json_content.contains(r#""filename": "test_square_file_1x1.png""#));
        assert!(json_content.contains(r#""name": "My Square Painting""#));
        assert!(json_content.contains(r#""artist": "The Artist""#));
        assert!(json_content.contains(r#""width": 1"#));
        assert!(json_content.contains(r#""height": 1"#));

        assert!(json_content.contains(r#""id": "Test_Square_4x4""#));
        assert!(json_content.contains(r#""width": 4"#));
        assert!(json_content.contains(r#""height": 4"#));

        // 6. Cleanup is handled by TempExportDir's Drop impl
    }
}