#![warn(unused)]
pub mod error;
pub mod fetcher;
pub mod sites;


#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::{net::Ipv4Addr, time::Instant};
    use image::load_from_memory;
    use wreq::Client;

    use crate::sites::Post;
    use crate::sites::nekosbest::NekosBest;
    use crate::sites::nekosmoe::NekosMoe;
    use crate::sites::waifuim::WaifuIm;
use crate::{fetcher::Fetcher};

    // Create a user agent that can be sent to the API.
    // Most sites require or at least recommend this.
    static APP_USER_AGENT: &str = "Waifu-API-rs (https://github.com/gicziunicorn/Waifu-API-rs)";

    // Wrapper function to measure the time of a test
    async fn measure<F, Fut>(func: F) -> String
    where F: FnOnce() -> Fut, Fut: Future<Output = String> {
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

        println!("--- Starting testing ---\n");

        // The functions that'll be tested
        let results = measure_all!(
            test_nekosmoe(fetcher.clone()),
            test_nekosbest(fetcher.clone()),
            test_waifuim(fetcher.clone()),
        ).await;

        // Print the results
        println!();
        results.iter().for_each(|res| {
            println!("------------------------\n{}", res);
        });
        println!("------------------------\n");
    }


    async fn test_nekosmoe(fetcher: Fetcher) -> String {
        // Create an instance of the site we want to use
        let mut nekosmoe = NekosMoe::new(fetcher);

        // Get a post
        let mut posts = nekosmoe.get_random_posts(1)
            .await.expect("Failed to fetch post");
        let post = posts.images.remove(0);
        let gpost: Post = post.into();

        // Get an image from the site
        let _image = nekosmoe.get_random_image().await.expect("Failed to get random");

        format!("Nekos.moe test done.\n{:#?}", gpost)
    }

    async fn test_nekosbest(fetcher: Fetcher) -> String {
        let mut site = NekosBest::new(fetcher);

        let mut posts = site.get_random_posts_with_category(crate::sites::nekosbest::NekosbestImageCategories::Kitsune, 1)
            .await.expect("Failed to fetch post");
        let post = posts.results.remove(0);
        let gpost: Post = post.into();

        // Get an image from the site
        let (image, _chosen_category) = site.get_random_image().await.expect("Failed to get random");

        println!("Remaining: {}, reset: {}", site.ratelimit_remaining, site.ratelimit_reset.unwrap());

        // Save the image
        load_from_memory(&image).expect("Failed to convert image")
            .save_with_format("image.jpg", image::ImageFormat::Jpeg).expect("Failed to save image");

        format!("Nekosbest test done.\n{:#?}", gpost)
    }

    async fn test_waifuim(fetcher: Fetcher) -> String {
        // Create an instance of the site we want to use
        let mut site = WaifuIm::new(fetcher);

        // Get a post
        let mut posts = site.get_random_posts(1)
            .await.expect("Failed to fetch post");
        let post = posts.items.remove(0);
        let gpost: Post = post.into();

        // Get an image from the site
        let _image = site.get_random_image().await.expect("Failed to get random");

        format!("Waifu.im test done.\n{:#?}", gpost)
    }
}
