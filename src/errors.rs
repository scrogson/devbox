use failure;

pub type Error = failure::Error;
pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Fail, Debug)]
#[fail(display = "Unable to find service {}", _0)]
pub struct ServiceNotFound(pub String);

#[derive(Fail, Debug)]
#[fail(display = "Unimplemented subcommand '{}'; please file a bug", _0)]
pub struct UnimplementedSubcommand(pub String);
