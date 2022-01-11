use log::Level;

pub fn setup_logger(level: Level) {
    env_logger::builder()
        .filter_level(level.to_level_filter())
        .format_timestamp(None)
        .format_target(false)
        .init()
}