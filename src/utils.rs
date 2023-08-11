use std::{fs::File, io::{Cursor, copy}, path::Path};

pub async fn download_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::get(url).await?;

    println!("Status: {}", res.status());

    let mut dest = File::create(path)?;
    let mut content = Cursor::new(res.bytes().await?);
    
    copy(&mut content, &mut dest)?;

    println!("Downloaded to: {}", path);

    Ok(())
}

pub fn get_app_data_dir() -> String {
    let home = std::env::var("HOME").unwrap();
    let path = format!("{}/.local/share/speedtest", home);

    if !Path::new(&path).exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    path
}
