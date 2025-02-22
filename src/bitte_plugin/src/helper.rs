use std::error::Error;
use std::marker::{Send, Sync};

use headless_chrome::protocol::cdp::Page;
use headless_chrome::Browser;
use serde::{Deserialize, Serialize};

pub async fn create_twitter_post_image(
    url: String,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let prefix = ["https://x.com", "https://twitter.com"];

    assert!(
        url.to_lowercase().starts_with(&prefix[0]) || url.to_lowercase().starts_with(&prefix[1]),
        "Hostname must be twitter.com or x.com"
    );

    // let mut builder = headless_chrome::LaunchOptions::default_builder();
    let builder = headless_chrome::LaunchOptions::default_builder();
    // Set headless mode based on whether it's in test mode
    // #[cfg(test)]
    // builder.headless(false); // Headful mode for tests

    // #[cfg(not(test))]
    // builder.headless(true); // Headless mode for non-test

    let browser = Browser::new(builder.build()?)?;

    let tab = browser.new_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(60));
    let _ = tab.set_user_agent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
        None,
        None
    );

    // Set screen Dimension (in this case mobile)
    let tab = tab.set_bounds(headless_chrome::types::Bounds::Normal {
        left: Some(0),
        top: Some(0),
        width: Some(475.0),
        height: Some(2000.0),
    })?;

    // Navigate to wikipedia
    tab.navigate_to(&url)?;

    tab.wait_until_navigated()?;

    let page_error = tab.find_element_by_xpath(
        "/html/body/div/div/div/div[2]/main/div/div/div/div/div/div[3]/div/span",
    );
    match page_error {
        Ok(element) => {
            let data = element.get_inner_text();
            match data {
                Ok(data) => {
                    return Err(format!("PageError Found: {}", data).into());
                }
                Err(_) => {}
            }
        }
        Err(_) => {
            // Proceed with execution for successful case
            println!("Page didn't error out");
        }
    }

    let view_port_selector =
        "#react-root > div > div > div.css-175oi2r.r-1f2l425.r-13qz1uu.r-417010 > main > div > div > div > div > div > section > div > div > div:nth-child(1) > div > div > article > div.css-175oi2r.r-eqz5dr.r-16y2uox.r-1wbh5a2 > div";
    let view_port_element = tab.wait_for_element(view_port_selector).unwrap();
    let view_port = view_port_element
        .get_box_model()
        .unwrap()
        .content_viewport();

    // println!("View Port: {:?}", view_port);

    // Take a screenshot a cropped view of the browser window
    let image_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Png,
        Some(100),
        Some(Page::Viewport {
            x: view_port.x - 10.0,
            y: view_port.y - 10.0,
            width: view_port.width + 20.0,
            height: view_port.height - 146.0 + 20.0,
            scale: 2.0,
        }),
        true,
    )?;

    Ok(image_data)
}

pub async fn create_twitter_post_image_from_id(
    tweet_id: u64,
) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    return create_twitter_post_image(format!("https://x.com/x/status/{}", tweet_id)).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use headless_chrome;

    #[tokio::test]
    async fn test_create_image() {
        println!("{:?}", headless_chrome::browser::default_executable());
        let tweet_id: u64 = 1834071245224308850;
        match create_twitter_post_image_from_id(tweet_id).await {
            Ok(bytes) => {
                assert!(bytes.len() > 0, "Image bytes should not be empty");
                // Save the screenshot to disc
                let _ = std::fs::write("./screenshot.png", bytes);
            }
            Err(e) => panic!("Expected Ok, but got Err: {:?}", e),
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Hostname must be twitter.com or x.com")]
    async fn test_url_starts_with_other_prefix() {
        let url = "https://example.com/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = create_twitter_post_image(url).await;
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    #[should_panic(expected = "Hostname must be twitter.com or x.com")]
    async fn test_url_starts_with_other_prefix_case_insensitive() {
        let url = "https://EXAMPLE.COM/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = create_twitter_post_image(url).await;
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    #[should_panic(expected = "Hostname must be twitter.com or x.com")]
    async fn test_empty_url() {
        let url = "".to_string();
        // This should fail as the URL is empty and does not start with either prefix.
        let result = create_twitter_post_image(url).await;
        assert!(result.is_err(), "The URL was expected to be invalid");
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetMetadata {
    image_url: String,
    owner_account_id: String,
    token_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ZkInputParam {
    /// verify Proof.
    pub proof: String,

    /// meta_data
    pub meta_data: AssetMetadata,
}
