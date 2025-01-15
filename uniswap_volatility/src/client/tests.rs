
#[cfg(test)]
mod tests {
    use super::*;
    use ethers::providers::Provider;
    use std::sync::Arc;
    use crate::client::Http;
    use crate::client::fetch_pool_price;

    #[tokio::test]
    async fn test_fetch_pool_price() {
        let provider = Arc::new(Provider::<Http>::try_from("https://mainnet.infura.io/v3/943fabd894044ec88ccae8613bf6b0b4").unwrap());
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8".parse().unwrap();
        let price = fetch_pool_price(provider, pool_address).await.unwrap();
        assert!(price.is_some());
        println!("******Price: {:?}", price.unwrap());
    }
}
