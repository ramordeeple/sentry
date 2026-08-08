pub mod api {
    pub mod http;
}

pub mod application {
    pub mod bootstrap;
}

pub mod config {
    pub mod settings;
}

pub mod domain {
    pub mod scenario;
}

pub mod errors {
    pub mod api_error;
    pub mod config_error;
    pub mod startup_error;
    pub mod storage_error;
}

pub mod serialization {
    pub mod cbr_xml;
}

pub mod storage {
    pub mod postgres;
}

pub use application::bootstrap::run;
pub use config::settings::Config;
