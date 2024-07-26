pub mod tests {
    use crate::models::{UserDTO, UserDTOSanitized};


    #[test]
    fn test_user_sanitation_and_serialization() {
        let user = UserDTO {
            password: String::from("PasswordTest"),
            public_key: vec![],
            username: String::from("UsernameTest")
        };

        let sanitized = user.sanitize_and_serialize().expect("Can not fail lol");

        let sanitized_expect = UserDTOSanitized {
            public_key: String::from(""),
            username: String::from("UsernameTest")
        };

        assert_eq!(sanitized, sanitized_expect)
    } 
}