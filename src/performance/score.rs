use std::str::FromStr;

use json::JsonValue;

pub struct Score
{
    red:  u8,
    blue: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Team
{
    Red,
    Blue,
}

impl Score
{
    pub fn new(red: u8, blue: u8) -> Self { Self { red, blue } }

    pub fn from_json(json: &JsonValue) -> Self
    {
        let red = json["Red"]["score"].as_u8().unwrap();
        let blue = json["Blue"]["score"].as_u8().unwrap();

        Self { red, blue }
    }

    pub fn get_score(&self, team: Team) -> u8
    {
        match team {
            Team::Red => self.red,
            Team::Blue => self.blue,
        }
    }
}

impl Team
{
    pub fn other(self) -> Self
    {
        match self {
            Self::Red => Self::Blue,
            Self::Blue => Self::Red,
        }
    }
}

impl FromStr for Team
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        match s.trim().to_lowercase().as_str() {
            "red" => Ok(Self::Red),
            "blue" => Ok(Self::Blue),
            _ => Err(()),
        }
    }
}
