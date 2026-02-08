use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub fn get_login_rate_limit_config() -> GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    // 5 yanlış denemeden sonra bloklanır.
    // Her yeni hak için 60 saniye (1 dakika) bekleme süresi.
    GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(60) 
        .burst_size(5)
        .finish()
        .unwrap()
}
