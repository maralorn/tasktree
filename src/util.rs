use std;

pub type Result<T> = std::result::Result<T, Box<std::error::Error>>;

pub fn run<F>(func: F)
where
    F: FnOnce() -> Result<()>,
{
    if let Err(err) = func() {
        println!(
            "There was the following error while Running a Closure: {}",
            err.description()
        )
    };
}
