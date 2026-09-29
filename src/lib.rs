#![allow(unused)]
mod types;
mod fetcher;
mod site;


#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use wreq::Client;
    use wreq_util::Emulation;

    use crate::fetcher::Fetcher;

    #[test]
    fn it_works() {
        let client = Client::builder()
            .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
            .emulation(Emulation::Chrome137)
            .build()
            .expect("Client failed");

        let fetcher = Fetcher::new(client);


    }
}
