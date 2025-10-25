use crate::models::image_size::ImageSize;
use image::{open, GenericImageView, DynamicImage};

/*
Used as an intermediary function to get proper crop dimensions of a given image. No public use.
*/
fn calculate_crop_dimensions(image_dims: (u32, u32), target_size: (u32, u32)) -> (u32, u32, u32, u32) {
    let (width, height) = image_dims;
    let (img_width, img_height) = target_size;

    // Use cross-multiplication to avoid floating point math and integer division errors. Technically more performant?
    // We cast to u64 to prevent overflow when multiplying dimensions.
    let is_image_wider_than_ratio =
        (width as u64 * img_height as u64) >= (height as u64 * img_width as u64);

    let scale_factor: u32 = match is_image_wider_than_ratio {
        // If the image is wider than the target ratio, the height is the limiting dimension.
        true => height / img_height,
        // If the image is taller, the width is the limiting dimension.
        false => width / img_width,
    };

    let crop_width: u32 = img_width * scale_factor;
    let crop_height: u32 = img_height * scale_factor;
    let width_start: u32 = (width - crop_width) / 2;
    let height_start: u32 = (height - crop_height) / 2;

    (width_start, height_start, crop_width, crop_height)
}

/*
Generates a vector of all 5 cropped image variants from a single source file path.
This is used to create transient images for Base64 preview generation.
These images are NOT stored in the application state to conserve memory.
*/
pub fn generate_cropped_images(path: &str) -> Result<Vec<DynamicImage>, image::ImageError> {
    let mut cropped_images: Vec<DynamicImage> = Vec::new();
    let img = open(path)?;
    let img_dims = img.dimensions();

    for size_variant in ImageSize::iter() {
        let target_size = size_variant.get_size()[0];
        let (width_start, height_start, crop_width, crop_height) =
            calculate_crop_dimensions(img_dims, target_size);

        let crop_view = img.view(width_start, height_start, crop_width, crop_height);
        let crop_preview = DynamicImage::ImageRgba8(crop_view.to_image());

        cropped_images.push(crop_preview);
    }
    Ok(cropped_images)
}

/*
Generates a single cropped image variant from a source file path.
This is used during the final export process to re-generate images on-demand.
*/
pub fn crop_single_image(
    path: &str,
    image_size: &ImageSize,
) -> Result<DynamicImage, image::ImageError> {
    let img = open(path)?;
    let img_dims = img.dimensions();
    let target_size = image_size.get_size()[0];

    let (width_start, height_start, crop_width, crop_height) =
        calculate_crop_dimensions(img_dims, target_size);

    let crop_view = img.view(width_start, height_start, crop_width, crop_height);
    Ok(DynamicImage::ImageRgba8(crop_view.to_image()))
}

#[cfg(test)]
mod tests {
    use super::*; // Import functions from parent module
    use image::{GenericImageView, RgbaImage};
    use std::fs;
    use std::path::PathBuf;

    // Use the real ImageSize from the crate models to ensure types match the functions under test
    use crate::models::image_size::ImageSize;

    // --- Test Image Helper ---
    // This struct creates a dummy image file when created
    // and automatically deletes it when it goes out of scope (using Drop)
    struct TestImage {
        path: PathBuf,
    }

    impl TestImage {
        fn new(path_str: &str, width: u32, height: u32) -> Self {
            let img = RgbaImage::new(width, height);
            let path = PathBuf::from(path_str);
            img.save(&path).expect("Failed to save test image");
            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path.to_str().expect("Path is not valid UTF-8")
        }
    }

    impl Drop for TestImage {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path); // Ignore error on cleanup
        }
    }

    // --- Unit Tests for calculate_crop_dimensions ---

    #[test]
    fn test_calc_crop_wider_image() {
        // 16:9 image (1600x900), 4:3 target
        let img_dims = (1600, 900);
        let target_size = (4, 3);

        // Crop should be based on height
        // scale_factor = 900 / 3 = 300
        // crop_width = 4 * 300 = 1200
        // crop_height = 3 * 300 = 900
        // width_start = (1600 - 1200) / 2 = 200
        // height_start = (900 - 900) / 2 = 0
        let (x, y, w, h) = calculate_crop_dimensions(img_dims, target_size);
        assert_eq!((x, y, w, h), (200, 0, 1200, 900));
    }

    #[test]
    fn test_calc_crop_taller_image() {
        // 9:16 image (900x1600), 4:3 target
        let img_dims = (900, 1600);
        let target_size = (4, 3);

        // Crop should be based on width
        // scale_factor = 900 / 4 = 225
        // crop_width = 4 * 225 = 900
        // crop_height = 3 * 225 = 675
        // width_start = (900 - 900) / 2 = 0
        // height_start = (1600 - 675) / 2 = 462
        let (x, y, w, h) = calculate_crop_dimensions(img_dims, target_size);
        assert_eq!((x, y, w, h), (0, 462, 900, 675));
    }

    #[test]
    fn test_calc_crop_same_ratio() {
        // 4:3 image (800x600), 4:3 target
        let img_dims = (800, 600);
        let target_size = (4, 3);

        // Crop should be the full image
        // scale_factor = 600 / 3 = 200
        // crop_width = 4 * 200 = 800
        // crop_height = 3 * 200 = 600
        // width_start = (800 - 800) / 2 = 0
        // height_start = (600 - 600) / 2 = 0
        let (x, y, w, h) = calculate_crop_dimensions(img_dims, target_size);
        assert_eq!((x, y, w, h), (0, 0, 800, 600));
    }

    #[test]
    fn test_calc_crop_square_target_from_landscape() {
        // 16:9 image (1600x900), 1:1 target
        let img_dims = (1600, 900);
        let target_size = (1, 1);

        // Crop should be based on height
        // scale_factor = 900 / 1 = 900
        // crop_width = 1 * 900 = 900
        // crop_height = 1 * 900 = 900
        // width_start = (1600 - 900) / 2 = 350
        // height_start = (900 - 900) / 2 = 0
        let (x, y, w, h) = calculate_crop_dimensions(img_dims, target_size);
        assert_eq!((x, y, w, h), (350, 0, 900, 900));
    }

    // --- Integration Tests for public functions ---

    #[test]
    fn test_crop_single_image_success() {
        // 800x600 (4:3) image
        let test_img = TestImage::new("test_single.png", 800, 600);
        // 1:1 target (Square)
        let size = ImageSize::Square; 
        
        let result = crop_single_image(test_img.path_str(), &size);
        assert!(result.is_ok());
        let cropped = result.unwrap();

        // Expected: crop based on height (600)
        // scale = 600 / 1 = 600
        // crop_w = 1 * 600 = 600
        // crop_h = 1 * 600 = 600
        // Final image dimensions should be 600x600
        assert_eq!(cropped.dimensions(), (600, 600));
    }

    #[test]
    fn test_generate_cropped_images_success() {
        // 1600x900 (16:9) image
        let test_img = TestImage::new("test_generate.png", 1600, 900);
        
        let result = generate_cropped_images(test_img.path_str());
        assert!(result.is_ok());
        let cropped_vec = result.unwrap();

        // ImageSize::iter() returns 5 variants
        assert_eq!(cropped_vec.len(), 5);

        // Check dimensions for each variant based on 1600x900 source
        // 1. Square (1:1) -> 900x900
        assert_eq!(cropped_vec[0].dimensions(), (900, 900));
        // 2. Wide (2:1) -> 1600x800
        assert_eq!(cropped_vec[1].dimensions(), (1600, 800));
        // 3. LongRectangle (4:3) -> 1200x900
        assert_eq!(cropped_vec[2].dimensions(), (1200, 900));
        // 4. Tall (1:2) -> 450x900
        assert_eq!(cropped_vec[3].dimensions(), (450, 900));
        // 5. TallRectangle (3:4) -> 675x900
        assert_eq!(cropped_vec[4].dimensions(), (675, 900));
    }

    #[test]
    fn test_crop_image_file_not_found() {
        let result = crop_single_image("nonexistent_file.png", &ImageSize::Square);
        assert!(result.is_err());
        // Check that it's an I/O error (which `open` returns for missing files)
        assert!(matches!(result.unwrap_err(), image::ImageError::IoError(_)));
    }

    #[test]
    fn test_generate_images_file_not_found() {
        let result = generate_cropped_images("nonexistent_file.png");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), image::ImageError::IoError(_)));
    }
}