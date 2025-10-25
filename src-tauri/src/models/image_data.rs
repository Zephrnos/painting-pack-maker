use crate::models::image_size::ImageSize;

// The `DynamicImage` field has been removed to reduce memory usage.
// This struct now only holds metadata about a potential crop.
#[derive(Debug, Clone)]
pub struct ImageData {
    pub id:         Option<String>,
    pub filename:   Option<String>,
    pub name:       Option<String>,
    pub artist:     Option<String>,
    pub image_size: ImageSize,
    pub selected:   bool,
}

impl ImageData {
    // The constructor no longer takes an image, only the size metadata.
    pub fn new(image_size: ImageSize) -> Self {
        ImageData {
            id:         None,
            filename:   None,
            name:       None,
            artist:     None,
            image_size,
            selected:   true,
        }
    }

    // `get_image()` has been removed as the image data is no longer stored here.

    pub fn get_sizes(&self) -> &[(u32, u32)] {
        self.image_size.get_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::image_size::ImageSize;

    #[test]
    fn test_new_constructor_sets_defaults() {
        let image_size = ImageSize::Square;
        let image_data = ImageData::new(image_size);

        // Check that all optional fields are None
        assert!(image_data.id.is_none());
        assert!(image_data.filename.is_none());
        assert!(image_data.name.is_none());
        assert!(image_data.artist.is_none());

        // Check that 'selected' defaults to true
        assert_eq!(image_data.selected, true);

        // Check that image_size is set correctly
        assert!(matches!(image_data.image_size, ImageSize::Square));
        assert_eq!(image_data.get_sizes(), &[(1, 1), (2, 2), (3, 3), (4, 4)]);
    }
}
