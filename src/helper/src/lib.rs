use std::error::Error;

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page;

pub async fn create_twitter_post_image(url:String)->Result<Vec<u8>, Box<dyn Error>>  {


    let prefix = ["https://x.com","https://twitter.com"];

    assert!(url.to_lowercase().starts_with(&prefix[0])||url.to_lowercase().starts_with(&prefix[1]), "Hostname must be twitter.com or x.com");
    

    let browser = Browser::default()?;

    browser.get_process_id();
    let tab = browser.new_tab()?;

    // Set screen Dimension (in this case mobile)
    let tab = tab.set_bounds(
        headless_chrome::types::Bounds::Normal { 
            left: Some(0), 
            top: Some(0), 
            width: Some(375.0), 
            height: Some(1500.0) 
        })?;

    // Navigate to wikipedia
    tab.navigate_to(&url)?;

    tab.wait_until_navigated()?;

    // Take a screenshot a cropped view of the browser window
    let jpeg_data = tab.capture_screenshot(
        Page::CaptureScreenshotFormatOption::Jpeg,
        Some(100),
        Some(Page::Viewport { x: 75.0, y: 50.0, width: 275.0, height: 1000.0, scale: 2.0 }),
        true)?;

    Ok(jpeg_data)
}



#[cfg(test)]
mod tests {    
    use super::*;
    use tokio;
    use headless_chrome;

    #[tokio::test]
    async fn test_url_starts_with_x_com() {
        let url = "https://X.com/some/path".to_string();
        // This should pass as the URL starts with "https://x.com", case-insensitively.
        let _ = create_twitter_post_image(url).await;
    }

    #[tokio::test]
    async fn test_url_starts_with_twitter_com() {

        println!("{:?}", headless_chrome::browser::default_executable());
        let url = "https://x.com/10015io/status/1495069139811061764".to_string();
        // This should pass as the URL starts with "https://twitter.com", case-insensitively.
        match create_twitter_post_image(url).await {
            Ok(bytes) => {
                assert!(bytes.len() > 0, "Image bytes should not be empty");
                    // Save the screenshot to disc
                    let _ = std::fs::write("./screenshot.jpeg", bytes);
            }
            Err(e) => panic!("Expected Ok, but got Err: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_url_starts_with_other_prefix() {
        let url = "https://example.com/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = std::panic::catch_unwind(|| {
            let _ = futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    async fn test_url_starts_with_other_prefix_case_insensitive() {
        let url = "https://EXAMPLE.COM/some/path".to_string();
        // This should fail as the URL does not start with "https://x.com" or "https://twitter.com".
        let result = std::panic::catch_unwind(|| {
            let _ =  futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }

    #[tokio::test]
    async fn test_empty_url() {
        let url = "".to_string();
        // This should fail as the URL is empty and does not start with either prefix.
        let result = std::panic::catch_unwind(|| {
            let _ =   futures::executor::block_on(create_twitter_post_image(url));
        });
        assert!(result.is_err(), "The URL was expected to be invalid");
    }
}

