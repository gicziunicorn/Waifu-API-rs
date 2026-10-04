#![warn(unused)]
pub mod error;
pub mod fetcher;
pub mod site;


#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::{net::Ipv4Addr, time::Instant};
    use image::load_from_memory;
    use wreq::Client;
    use crate::{fetcher::Fetcher, site::{NekosBest, NekosMoe}};

    // Create a user agent that can be sent to the API.
    // Most sites require or at least recommend this.
    static APP_USER_AGENT: &str = "Waifu-API-rs (https://github.com/gicziunicorn/Waifu-API-rs)";


    // Wrapper function to measure the time of a test
    async fn measure<F, Fut>(func: F) -> String
    where F: FnOnce() -> Fut, Fut: Future<Output = &'static str> {
        let start = Instant::now();

        let msg = func().await;

        format!("{}\nTime taken: {:.3}s", msg, start.elapsed().as_secs_f32())
    }

    // Put all function into measure()
    macro_rules! measure_all {
        ($($func:expr),* $(,)?) => {
            futures::future::join_all(vec![
                $(
                    Box::pin(measure(|| $func)) as Pin<Box<dyn Future<Output = _>>>
                ),*
            ])
        };
    }


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

        println!("Starting testing.\n");

        // The functions that'll be tested
        let results = measure_all!(
            test_nekosmoe(fetcher.clone()),
            test_nekosbest(fetcher.clone()),
        ).await;

        results.iter().for_each(|res| {
            println!("------------\n{}\n", res);
        });
    }


    async fn test_nekosmoe(fetcher: Fetcher) -> &'static str {
        // Create an instance of the site we want to use
        let nekosmoe = NekosMoe::new(fetcher);

        // Get an image from the site
        let _image = nekosmoe.get_random_image().await.expect("Failed to get random");

        "Nekos.moe test done."
    }

    async fn test_nekosbest(fetcher: Fetcher) -> &'static str {
        let mut site = NekosBest::new(fetcher);

        // Get an image from the site
        let (image, _chosen_category) = site.get_random_image().await.expect("Failed to get random");

        println!("Remaining: {}, reset: {}", site.ratelimit_remaining, site.ratelimit_reset.unwrap());

        // Save the image
        load_from_memory(&image).expect("Failed to convert image")
            .save_with_format("image.jpg", image::ImageFormat::Jpeg).expect("Failed to save image");

        "Nekosbest test done."
    }
}
