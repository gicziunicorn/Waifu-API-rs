#![allow(unused)]
mod types;
mod fetcher;
mod site;


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use image::{load_from_memory};
    use tokio::{join, spawn};
use wreq::Client;
    use wreq_util::Emulation;
    use crate::{fetcher::Fetcher, site::{NekosBest, NekosMoe}};

    // Create a user agent that can be sent to the API.
    // Most sites require or at least recommend this.
    static APP_USER_AGENT: &str = "Waifu-API-rs (https://github.com/gicziunicorn/Waifu-API-rs)";

    #[tokio::test]
    async fn main_test() {
        // Create the client for the app
        let client = Client::builder()
            .local_addresses(Ipv4Addr::UNSPECIFIED, None)
            .user_agent(APP_USER_AGENT)
            .build()
            .expect("Client failed");

        // Initialize the fetcher
        let fetcher = Fetcher::new(client);

        spawn( test_nekosmoe(fetcher.clone()) ).await;
        spawn( test_nekosbest(fetcher.clone()) ).await;
    }

    async fn test_nekosmoe(fetcher: Fetcher) {
        println!("Nekos.moe test started");

        // Create an instance of the site we want to use
        let nekosmoe = NekosMoe::new(fetcher);

        // Get an image from the site
        let image = nekosmoe.get_random_image().await.expect("Failed to get random");

        println!("Nekos.moe test done!");
    }

    async fn test_nekosbest(fetcher: Fetcher) {
        println!("Nekosbest test started");

        let site = NekosBest::new(fetcher);

        // Get an image from the site
        let (image, chosen_category) = site.get_random_image().await.expect("Failed to get random");

        println!("Chosen category: {}", chosen_category);

        // Save the image
        load_from_memory(&image).expect("Failed to convert image")
            .save_with_format("image.jpg", image::ImageFormat::Jpeg).expect("Failed to save image");

        println!("Nekosbest test done!");
    }
}
