pub mod ascii_image;
pub mod basic;
pub mod output;
pub mod asciifier;
pub mod font_handler;
pub mod error;


trait VoidError<V> {
    fn void(self) -> Result<V, ()>;
}

impl<V, E> VoidError<V> for Result<V, E> {
    fn void(self) -> Result<V, ()> {
        self.map_err(|_|())
    }
}
