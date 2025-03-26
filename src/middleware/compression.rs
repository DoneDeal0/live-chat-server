use tower_http::compression::CompressionLayer;

pub fn compress_responses() -> CompressionLayer {
    CompressionLayer::new()
}
