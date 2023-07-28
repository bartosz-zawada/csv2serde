use thiserror::Error;

#[derive(Error, Debug)]
enum C2SError {}

type C2SResult<T> = Result<T, C2SError>;

fn main() -> C2SResult<()> {
    Ok(())
}
