use serde::{Serialize};
use std::slice::Iter;

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
pub enum ImageSize {
    Square,
    Wide, 
    LongRectangle,
    Tall,
    TallRectangle,
}

impl ImageSize {

    pub fn iter() -> Iter<'static, ImageSize> {
        static PAINTING_SIZES: [ImageSize; 5] = [
            ImageSize::Square,
            ImageSize::Wide,
            ImageSize::LongRectangle,
            ImageSize::Tall,
            ImageSize::TallRectangle,
        ];
        PAINTING_SIZES.iter()
    }

    pub fn get_size(&self) -> &'static [(u32, u32)] {
    match self {
        ImageSize::Square => &[(1, 1), (2, 2), (3, 3), (4, 4)],
        ImageSize::Wide => &[(2, 1), (4, 2)],
        ImageSize::LongRectangle => &[(4, 3)],
        ImageSize::Tall => &[(1, 2), (2, 4)],
        ImageSize::TallRectangle => &[(3, 4)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_returns_five_variants() {
        let variants: Vec<&ImageSize> = ImageSize::iter().collect();
        assert_eq!(variants.len(), 5);
        assert_eq!(variants[0], &ImageSize::Square);
        assert_eq!(variants[1], &ImageSize::Wide);
        assert_eq!(variants[2], &ImageSize::LongRectangle);
        assert_eq!(variants[3], &ImageSize::Tall);
        assert_eq!(variants[4], &ImageSize::TallRectangle);
    }

    #[test]
    fn test_get_size_for_all_variants() {
        assert_eq!(ImageSize::Square.get_size(), &[(1, 1), (2, 2), (3, 3), (4, 4)]);
        assert_eq!(ImageSize::Wide.get_size(), &[(2, 1), (4, 2)]);
        assert_eq!(ImageSize::LongRectangle.get_size(), &[(4, 3)]);
        assert_eq!(ImageSize::Tall.get_size(), &[(1, 2), (2, 4)]);
        assert_eq!(ImageSize::TallRectangle.get_size(), &[(3, 4)]);
    }
}
