//! Used to load resources, changes load location based on target. \
//! If target is `"wasm32"`, then load location will be the server. Otherwise
//! it will load from the filesystem.

use cfg_if::cfg_if;

const ASSET_FOLDER: &'static str = "assets";

/// Creates a url that we can use to load resources.
#[cfg(target_arch = "wasm32")]
fn format_url(file_folder: &str, file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with(file_folder) {
        origin = format!("{}/{}", origin, file_folder);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

/// Loads a string. If `is_asset` is true, then it will only load from the `assets` folder.
pub async fn load_string(file_name: &str, is_asset: bool) -> anyhow::Result<String> {
    let mut parent_path = "";
    if is_asset {
        parent_path = ASSET_FOLDER;
    }
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(parent_path, file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(parent_path) // env!("OUT_DIR"))
                // .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

/// Loads a binary. If `is_asset` is true, then it will only load from the `assets` folder.
pub async fn load_binary(file_name: &str, is_asset: bool) -> anyhow::Result<Vec<u8>> {
    let mut parent_path = "";
    if is_asset {
        parent_path = ASSET_FOLDER;
    }

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(parent_path, file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(parent_path)
                // .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

#[cfg(test)]
mod tests {

    use anyhow::anyhow;

    use super::*;

    #[test]
    fn load_string_test() -> anyhow::Result<()> {
        let path = "resources-test-text-file.txt";
        let contents = "test text";
        std::fs::write(path, contents)?;
        let read_data = pollster::block_on(load_string(path, false))?;

        std::fs::remove_file(path)?;

        if contents == read_data {
            return Ok(());
        }

        return Err(anyhow!("Contents did not match read data!"));
    }

    #[test]
    fn load_binary_test() -> anyhow::Result<()> {
        let path = "resources-test-binary-file.png";
        let contents = "test text".as_bytes();
        std::fs::write(path, contents)?;
        let read_data = pollster::block_on(load_binary(path, false))?;

        std::fs::remove_file(path)?;

        if contents == read_data {
            return Ok(());
        }

        return Err(anyhow!("Contents did not match read data!"));
    }
}
