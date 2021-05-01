extern crate ini;

#[cfg(test)]
mod tests {
    use ini::Ini;

    #[test]
    fn parse() {
        // Arrange
        let config = r###"# Editor configuration
root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
insert_final_newline = true
trim_trailing_whitespace = true

[*.md]
max_line_length = off
trim_trailing_whitespace = false"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        let star = conf.section(Some("*")).unwrap();
        assert!(star.contains_key("charset"));
        let md = conf.section(Some("*.md")).unwrap();
        assert!(md.contains_key("max_line_length"));

        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_with_duplicate_values() {
        // Arrange
        let config = r###"
[*]
charset = utf-8
charset = utf-7
"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        let star = conf.section(Some("*")).unwrap();
        assert!(star.contains_key("charset"));

        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_complex_section_name() {
        // Arrange
        let config = r###"
[*.{js,jsx,ts,tsx,vue}]
indent_style = space
indent_size = 2
trim_trailing_whitespace = true
insert_final_newline = true
"###;

        // Act
        let conf = Ini::load_from_str(config).unwrap();

        // Assert
        for (sec, prop) in &conf {
            println!("Section: {:?}", sec);
            for (key, value) in prop.iter() {
                println!("{:?}:{:?}", key, value);
            }
        }
    }

    #[test]
    fn parse_invalid_syntax() {
        // Arrange
        let config = r###"
[*
charset = utf-8
charset = utf-7
"###;

        // Act
        let conf = Ini::load_from_str(config);

        // Assert
        assert!(conf.is_err());
    }
}
