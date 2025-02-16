pub fn to_upper_camel_case(s: &str) -> String {
    let mut name = to_lower_camel_case(s).to_string();

    name.replace_range(
        0..1,
        name.chars()
            .next()
            .unwrap()
            .to_uppercase()
            .to_string()
            .as_str(),
    );

    remove_plural(name)
}

pub fn to_lower_camel_case(s: &str) -> String {
    let mut all_upper = true;

    for c in s.chars() {
        if c.is_lowercase() {
            all_upper = false;
            break;
        }
    }

    let mut name = if all_upper {
        s.to_lowercase()
    } else {
        s.to_string()
    };

    let mut i = 0;

    while i < name.len() - 1 {
        let c = name.chars().nth(i).unwrap();

        if c == '_' {
            name.replace_range(
                i..i + 2,
                name.chars()
                    .nth(i + 1)
                    .unwrap()
                    .to_uppercase()
                    .to_string()
                    .as_str(),
            );
        }

        i += 1;
    }

    name = name.replace("_", "");

    name.replace_range(
        0..1,
        name.chars()
            .next()
            .unwrap()
            .to_lowercase()
            .to_string()
            .as_str(),
    );

    remove_plural(name)
}

fn remove_plural(s: String) -> String {
    if s.ends_with("s") && !s.ends_with("ss") {
        return s[..s.len() - 1].to_string();
    }

    s
}

#[cfg(test)]
mod test {
    use crate::naming::{to_lower_camel_case, to_upper_camel_case};

    #[tokio::test]
    async fn test_to_upper_camel_case() {
        assert_eq!(to_upper_camel_case("users"), "User");
        assert_eq!(to_upper_camel_case("user_address"), "UserAddress");
        assert_eq!(to_upper_camel_case("USERS_ADDRESS"), "UsersAddress");
        assert_eq!(to_upper_camel_case("FAMILIAR"), "Familiar");
        assert_eq!(to_upper_camel_case("UserAddress"), "UserAddress");
        assert_eq!(to_upper_camel_case("UserAddress_"), "UserAddress");
        assert_eq!(to_upper_camel_case("_A"), "A");
        assert_eq!(to_upper_camel_case("_Abc_Def"), "AbcDef");
    }

    #[tokio::test]
    async fn test_to_lower_camel_case() {
        assert_eq!(to_lower_camel_case("user"), "user");
        assert_eq!(to_lower_camel_case("user_address"), "userAddress");
        assert_eq!(to_lower_camel_case("USERS_ADDRESS"), "usersAddress");
        assert_eq!(to_lower_camel_case("UserAddress"), "userAddress");
        assert_eq!(to_lower_camel_case("UserAddress_"), "userAddress");
        assert_eq!(to_lower_camel_case("_A"), "a");
        assert_eq!(to_lower_camel_case("_Abc_Def"), "abcDef");
    }
}