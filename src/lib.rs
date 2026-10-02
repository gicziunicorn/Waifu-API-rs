#![allow(unused)]
mod types;
mod fetcher;
mod site;


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use image::{load_from_memory};
    use wreq::Client;
    use wreq_util::Emulation;

    use crate::{fetcher::Fetcher, site::{NekosMoe, Site}};

    #[tokio::test]
    async fn main_test() {
        static APP_USER_AGENT: &str = "Waifu-API-rs (https://github.com/gicziunicorn/Waifu-API-rs)";

        println!("{}", APP_USER_AGENT);

        panic!("exit");

        let client = Client::builder()
            .local_addresses(Ipv4Addr::UNSPECIFIED, None)
            .user_agent(APP_USER_AGENT)
            .build()
            .expect("Client failed");

        let fetcher = Fetcher::new(client);

        let nekosmoe = NekosMoe::new();

        let image = nekosmoe.get_random_image(&fetcher).await.expect("Failed to get random");

        load_from_memory(&image).expect("Failed to convert image").save_with_format("image.jpg", image::ImageFormat::Jpeg).expect("Failed to save image");
    }
}
