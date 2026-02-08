use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub fn get_login_rate_limit_config() -> GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    // Katı limitler:
    // Başlangıçta 3 hak. 4. denemede bloklanır.
    // Her yeni hak için 20 saniye bekleme süresi.
    GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(20) 
        .burst_size(3)
        .finish()
        .unwrap()
}