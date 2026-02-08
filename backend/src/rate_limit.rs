use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use tower_governor::governor::GovernorConfig;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;

pub fn get_login_rate_limit_config() -> GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    // 20 saniyede bir yeni hak verilir (dakikada 3 istek).
    // Başlangıçta 3 isteklik bir patlama (burst) hakkı tanınır.
    // Kullanıcı 3 kere hızlıca deneyebilir, 4. deneme için 20 saniye beklemesi gerekir.
    GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(20)
        .burst_size(3)
        .finish()
        .unwrap()
}