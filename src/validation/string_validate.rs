pub struct StringValidator {
    pub min_length: u16,
    pub max_length: Option<u16>
}

impl StringValidator {
    pub fn new(min_length: u16, max_length: Option<u16>) -> StringValidator {
        StringValidator { min_length, max_length }
    }

    pub fn validate(&self, value: &str) -> Result<(), String> {
        let length = value.len() as u16;
        if length < self.min_length {
            return Result::Err(String::from("Too short"))
        }

        if let Some(max) = self.max_length {
            if length > max {
                return Err(String::from("Too long"))
            }
        }

        Ok(())
    }
}

pub const DEFAULT_INPUT_FIELD_STRING_VALIDATOR: StringValidator = StringValidator { min_length: 1, max_length: Some(80) };