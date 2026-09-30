#![allow(unused)]
mod types;
mod fetcher;
mod site;


#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Write, net::Ipv4Addr};
    use image::{load_from_memory, load_from_memory_with_format};
    use wreq::Client;
    use wreq_util::Emulation;

    use crate::{fetcher::Fetcher, site::{NekosMoe, Site}};

    #[tokio::test]
    async fn it_works() {
        let client = Client::builder()
            .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
            .emulation(Emulation::Chrome137)
            .build()
            .expect("Client failed");

        let fetcher = Fetcher::new(client);

        let nekosmoe = NekosMoe::new();

        let random = nekosmoe.get_random_image(&fetcher).await.expect("Failed to get random");

        let url = format!("https://nekos.moe/image/{}", random[0].id);
        let image = fetcher.fetch_image(&url).await.expect("Failed to get image");

        load_from_memory(&image).expect("Failed to convert image").save_with_format("image.jpg", image::ImageFormat::Jpeg).expect("Failed to save image");
    }
}
