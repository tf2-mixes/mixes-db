use std::fmt;
use std::str::FromStr;

/// All TF2 classes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Class
{
    Demoman,
    Engineer,
    Heavy,
    Medic,
    Pyro,
    Scout,
    Sniper,
    Soldier,
    Spy,
}

impl Class
{
    /// Check if the class is considered a "main" class, which means it is under
    /// consideration for main (most) played class during a game.
    ///
    /// # Returns
    /// `true` if the class is main-classeable, currently that includes
    /// `Demoman`, `Scout`, `Soldier` and `Medic`. `false` for all other
    /// classes.
    pub fn is_main_class(self) -> bool
    {
        match self {
            Self::Demoman | Self::Medic | Self::Scout | Self::Soldier => true,
            _ => false,
        }
    }
}

/// When creating identifying a class from a string, the class may be unknown in
/// case the string does not conform to all lowercase string as it is present in
/// the logs.tf API. In that case, this error is thrown, containing the content
/// of the string.
#[derive(Debug)]
pub struct UnknownClassError
{
    class: String,
}

impl fmt::Display for UnknownClassError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "Unknown class `{}`", &self.class)
    }
}

impl std::error::Error for UnknownClassError {}

impl FromStr for Class
{
    // Returns the name of the class in case the class is unknown
    type Err = UnknownClassError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s {
            "demoman" => Ok(Self::Demoman),
            "engineer" => Ok(Self::Engineer),
            "heavy" | "heavyweapons" => Ok(Self::Heavy),
            "medic" => Ok(Self::Medic),
            "pyro" => Ok(Self::Pyro),
            "scout" => Ok(Self::Scout),
            "sniper" => Ok(Self::Sniper),
            "soldier" => Ok(Self::Soldier),
            "spy" => Ok(Self::Spy),
            unknown => Err(UnknownClassError {
                class: unknown.to_string(),
            }),
        }
    }
}
