pub mod subscene;

pub trait Downloadable {
    fn name(&self) -> &str;
    fn lang(&self) -> &str;
    fn download(&self) -> Vec<u8>;

    fn dbg(&self);
}

pub trait Provider {
    fn search(&self, name: &str, lang: &str) -> Vec<Box<Downloadable>>;
    fn accepts_whole_name(&self) -> bool { true }
}