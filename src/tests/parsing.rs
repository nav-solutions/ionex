use crate::prelude::IONEX;

#[test]
fn gzip_repo_parsing() {
    let prefix = "data/IONEX/V1";

    for file in std::fs::read_dir(prefix).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        let fullpath = &path.to_str().unwrap();

        let filename = file.file_name().to_str().unwrap().to_string();

        if filename.starts_with('.') {
            continue;
        }

        if filename.ends_with(".gz") {
            println!("parsing \"{}\"", fullpath);

            let ionex = IONEX::from_gzip_file(fullpath).unwrap_or_else(|e| {
                panic!("Failed to parse \"{}\": {}", fullpath, e);
            });
        }
    }
}
