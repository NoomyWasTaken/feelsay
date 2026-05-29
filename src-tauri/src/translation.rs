pub trait TranslationEngine {
    fn engine_id(&self) -> &'static str;
}
