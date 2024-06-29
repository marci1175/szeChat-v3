use chrono::{Duration, Utc};
use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/main.ico");
        res.compile().unwrap();
    }

    //This will always make build_info.matthias_build update, regardless if it has been compiled (because of cargo test)
    let date = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap_or_default()
        .format("%Y.%m.%d. %H:%M");
    if let Err(err) = fs::write("build_info.Matthias_build", date.to_string()) {
        println!("{err}")
    };

    generate_emoji_header()?;

    Ok(())
}

fn generate_emoji_header() -> Result<(), Box<dyn std::error::Error>> {
    let path_to_output = PathBuf::from(format!("{}/emoji_header.rs", std::env::var("OUT_DIR")?));

    //This will get written to the output file
    let mut content = String::new();

    //Emoji types
    let mut emoji_types: Vec<std::ffi::OsString> = Vec::new();

    //Emoji type directories
    let mut emoji_type_dir: Vec<PathBuf> = Vec::new();

    let read_dir = fs::read_dir(PathBuf::from("icons/emojis"))?;

    for entry in read_dir {
        let dir_entry = entry?;

        if dir_entry.metadata()?.is_dir() {
            let dir_name = dir_entry.file_name();

            //Push back path
            emoji_type_dir.push(dir_entry.path());

            //Push back type
            emoji_types.push(dir_name.clone());

            //Push back structs
            content.push_str(&format!(
                "#[allow(dead_code)]\n#[derive(Debug, Clone)]\npub struct {} {{ pub name: &'static str, pub bytes: &'static [u8] }}\n",
                dir_name.into_string().unwrap()
            ));
        };
    }

    //This will be the body of the enum generated by this file
    let mut enum_inner_fields = String::new();

    for (idx, emoji_type) in emoji_types.iter().enumerate() {
        enum_inner_fields.push_str(&format!(
            "\t{}([{}; {}]),\n",
            emoji_type.to_string_lossy(),
            emoji_type.to_string_lossy(),
            fs::read_dir(emoji_type_dir[idx].clone())?.count()
        ));
    }

    //Push back final enum
    content.push_str(&format!(
        "#[allow(dead_code)]\n#[derive(Debug, Clone, strum_macros::EnumDiscriminants)]\npub enum EmojiTypes {{
{enum_inner_fields}
}}\n"
    ));

    //Push back final struct, which contains all of the structs
    content.push_str(&format!(
        "#[allow(dead_code)]\npub struct Emoji {{
    pub emoji_types: [EmojiTypes; {}]
}}\n",
        emoji_types.len()
    ));

    //Create constants
    //Create constant body
    let mut constant_body = String::new();

    for (idx, emoji_type) in emoji_types.iter().enumerate() {
        let emoji_type_string = emoji_type.clone().into_string().unwrap();

        let mut emoji_type_body = String::new();

        //Read the contents of the emoji type's folder
        for folder_entry in fs::read_dir(emoji_type_dir[idx].clone())? {
            //Catch error
            let folder_entry = folder_entry?;

            //Get file (image) name
            let file_name = folder_entry.file_name();

            //Get file path
            let file_path = folder_entry.path();

            emoji_type_body.push_str(&format!(r#"{emoji_type_string}{{name: "{}", bytes: include_bytes!(r"..\\..\\..\..\\..\\{}")}},
            "#, file_name.to_string_lossy().split('.').next().unwrap(), file_path.to_string_lossy().replace('\\', r"\\").replace('/', r"\\")));
        }

        constant_body.push_str(&format!(
            "EmojiTypes::{emoji_type_string}([\n\t\t\t{emoji_type_body}\t\t]),\n
        "
        ));
    }

    //Create main constant
    content.push_str(&format!(
        "#[allow(dead_code)]\npub const EMOJIS: Emoji = Emoji {{
    emoji_types: [\n\t\t{constant_body}\n]
}};"
    ));

    //Write the contents to the file
    fs::write(path_to_output, content)?;

    Ok(())
}
