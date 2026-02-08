use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::GlobalKeyExtractor;

pub fn get_login_rate_limit_config() -> GovernorConfig<GlobalKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    // GLOBAL TEST: Kim olursa olsun 2 denemeden sonra 30 saniye bloklanır.
    GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .per_second(30) 
        .burst_size(2)
        .finish()
        .unwrap()
}
