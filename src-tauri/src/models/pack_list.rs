use serde::Serialize;
use rand::Rng;

#[derive(Serialize, Debug)]
pub struct PackList<T> {
    #[serde(rename = "name")]
    pub pack_name: String,
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String, 
    pub id: String, 
    pub description: String,
    paintings: Vec<T>,
}

impl<T> Default for PackList<T> {
    fn default() -> Self {

        let mut rng = rand::rng();
        let random_int: i32 = rng.random_range(56000..=128000);
        let random_id = format!("{}", random_int);
        
        PackList {
            pack_name: String::from("Default"),
            schema: String::from("http://json-schema.org/draft-07/schema#"),
            version: String::from("1.0.0"),
            id: random_id, 
            description: String::from("A list of paintings in the gallery"),
            paintings: Vec::new(),
        }
    }
}

fn check_no_input(input: &str) -> Option<String> {
    if input.trim().is_empty() {
        None
    } else {
         Some(input.to_string())
    }
}

impl<T> PackList<T> {

    pub fn new(pack_name: String, version: String, id: String, description: String) -> Self {
        PackList {
            pack_name,
            schema: String::from("http://json-schema.org/draft-07/schema#"),
            version,
            id,
            description,
            paintings: Vec::new(),
        }
    }

    pub fn set_pack_name(&mut self, pack_name: &str) {
        match check_no_input(pack_name) {
            Some(valid_pack_name) => self.pack_name = valid_pack_name,
            None => {},
        }
    }

    // unused, nobody is setting a schema round these parts
    pub fn _set_schema(&mut self, schema: &str) {
        match check_no_input(schema) {
            Some(valid_schema) => self.schema = valid_schema,
            None => {},
        }
    }

    pub fn set_version(&mut self, version: &str) {
        match check_no_input(version) {
            Some(valid_version) => self.version = valid_version,
            None => {},
        }
    }

    pub fn set_id(&mut self, id: &str) {
        match check_no_input(id) {
            Some(valid_id) => self.id = valid_id,
            None => {},
        }
    }

    pub fn set_description(&mut self, description: &str) {
        match check_no_input(description) {
            Some(valid_description) => self.description = valid_description,
            None => {},
        }
    }

    pub fn add_painting(&mut self, painting: T) {
        self.paintings.push(painting);
    }

    pub fn painting_count(&self) -> usize {
        self.paintings.len()
    }

    pub fn separate_paintings<U>(self) -> (PackList<U>, Vec<T>) {
        
        // 1. Create the new struct with a new, empty `paintings` vector.
        //    This moves all the metadata fields (schema, id, etc.).
        let new_list = PackList {
            pack_name: self.pack_name,
            schema: self.schema,
            version: self.version,
            id: self.id,
            description: self.description,
            paintings: Vec::new(),
        };

        // 2. The only thing left in `self` is the original `paintings` vector.
        //    We can now move it out.
        let original_paintings = self.paintings;

        // 3. Return both new pieces as a tuple.
        (new_list, original_paintings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tests for check_no_input helper ---

    #[test]
    fn test_check_no_input_valid() {
        let input = "  Valid Input  ";
        assert_eq!(check_no_input(input), Some("  Valid Input  ".to_string()));
    }

    #[test]
    fn test_check_no_input_empty() {
        let input = "";
        assert_eq!(check_no_input(input), None);
    }

    #[test]
    fn test_check_no_input_whitespace_only() {
        let input = "    ";
        assert_eq!(check_no_input(input), None);
    }

    // --- Tests for PackList methods ---

    #[test]
    fn test_packlist_new() {
        let list: PackList<i32> = PackList::new(
            "Test Pack".to_string(),
            "1.1.0".to_string(),
            "test_id".to_string(),
            "A test description".to_string(),
        );
        assert_eq!(list.pack_name, "Test Pack");
        assert_eq!(list.version, "1.1.0");
        assert_eq!(list.id, "test_id");
        assert_eq!(list.description, "A test description");
        assert_eq!(list.painting_count(), 0);
    }

    #[test]
    fn test_packlist_default() {
        let list: PackList<i32> = PackList::default();
        assert_eq!(list.pack_name, "Default");
        assert_eq!(list.version, "1.0.0");
        assert_eq!(list.description, "A list of paintings in the gallery");
        assert!(!list.id.is_empty()); // ID should be a random number string
        assert_eq!(list.painting_count(), 0);
    }

    #[test]
    fn test_set_pack_name() {
        let mut list: PackList<i32> = PackList::default();
        assert_eq!(list.pack_name, "Default");
        
        // Test valid update
        list.set_pack_name("New Name");
        assert_eq!(list.pack_name, "New Name");

        // Test invalid (empty) update
        list.set_pack_name("   ");
        assert_eq!(list.pack_name, "New Name"); // Should not change
    }

    #[test]
    fn test_set_version() {
        let mut list: PackList<i32> = PackList::default();
        assert_eq!(list.version, "1.0.0");
        
        // Test valid update
        list.set_version("2.0.0");
        assert_eq!(list.version, "2.0.0");

        // Test invalid (empty) update
        list.set_version("");
        assert_eq!(list.version, "2.0.0"); // Should not change
    }

    #[test]
    fn test_set_id() {
        let mut list: PackList<i32> = PackList::default();
        let original_id = list.id.clone();
        
        // Test valid update
        list.set_id("new_custom_id");
        assert_eq!(list.id, "new_custom_id");

        // Test invalid (empty) update
        list.set_id("  ");
        assert_eq!(list.id, "new_custom_id"); // Should not change
    }

    #[test]
    fn test_set_description() {
        let mut list: PackList<i32> = PackList::default();
        assert_eq!(list.description, "A list of paintings in the gallery");
        
        // Test valid update
        list.set_description("A new description.");
        assert_eq!(list.description, "A new description.");

        // Test invalid (empty) update
        list.set_description(" ");
        assert_eq!(list.description, "A new description."); // Should not change
    }

    #[test]
    fn test_add_painting() {
        let mut list: PackList<i32> = PackList::default();
        assert_eq!(list.painting_count(), 0);
        
        list.add_painting(100);
        assert_eq!(list.painting_count(), 1);

        list.add_painting(200);
        assert_eq!(list.painting_count(), 2);
    }

    #[test]
    fn test_separate_paintings() {
        let mut list: PackList<i32> = PackList::new(
            "Original Pack".to_string(),
            "1.0".to_string(),
            "original_id".to_string(),
            "Original Desc".to_string(),
        );
        list.add_painting(1);
        list.add_painting(2);

        // Perform the separation
        // We are separating a PackList<i32> into a PackList<String> and a Vec<i32>
        let (new_list, original_paintings_vec): (PackList<String>, Vec<i32>) = list.separate_paintings();

        // Check the new list (metadata)
        assert_eq!(new_list.pack_name, "Original Pack");
        assert_eq!(new_list.id, "original_id");
        assert_eq!(new_list.painting_count(), 0); // New list should have no paintings

        // Check the extracted vector
        assert_eq!(original_paintings_vec.len(), 2);
        assert_eq!(original_paintings_vec[0], 1);
        assert_eq!(original_paintings_vec[1], 2);
    }
}