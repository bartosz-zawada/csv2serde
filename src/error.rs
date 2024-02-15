#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not parse headers: {0}")]
    CantParseHeaders(#[source] csv::Error),

    #[error("Could not parse record: {0}")]
    CantParseRecord(#[source] csv::Error),

    #[error("Could not generate code: {0}")]
    CantGenerateCode(#[source] syn::Error),
}
