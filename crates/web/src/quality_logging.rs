//! Shared native logging boundary for quality decisions.
use js::Unknown;
use react::webapi::console;
use std::rc::Rc;
pub type QualityLogger = Rc<dyn Fn(&str, &Unknown)>;
pub fn default_quality_logger() -> QualityLogger {
    Rc::new(|message, details| console::info((message, details)))
}
