//! Handling of steam ids. Since there are multiple versions and the logs.tf API
//! uses steamID64 for lookups but has steamID3s in the log files, a safe
//! conversion and type safety between these two is critical.

use std::str::FromStr;

use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

const ACCOUNT_INSTANCE_OFFSET_BITS: u64 = 32;
// Account instance is located at ACCOUNT_INSTANCE_OFFSET_BITS and 20 bits long
const ACCOUNT_INSTANCE_MASK: u64 = 0xfffff << ACCOUNT_INSTANCE_OFFSET_BITS;
const ACCOUNT_TYPE_OFFSET_BITS: u64 = 52;
const UNIVERSE_OFFSET_BITS: u64 = 56;

#[derive(Copy, Clone, Debug)]
pub struct SteamID
{
    id64: u64,
}

impl SteamID
{
    /// Create a steam id from its steamID64 representation.
    ///
    /// The id value is not checked and it is therefore possible to create an
    /// invalid steam id with this.
    pub const unsafe fn new(id64: u64) -> Self { Self { id64 } }

    /// Create a steam id from its steamID64 representation.
    ///
    /// Checks if the id is in a sane format. In case it is not, an `Err(())` is
    /// returned.
    ///
    /// # Warning
    /// It does not actually make a request to check if there is a profile
    /// connected to this steam id, so lookups for the profile may still fail.
    pub fn new_checked(id64: u64) -> Result<Self, ()>
    {
        if Self::try_for_universe(id64).is_some()
            && Self::try_for_account_type(id64).is_some()
            // Check for normal user account, the only one currently supported
            && (id64 & ACCOUNT_INSTANCE_MASK == 1 << ACCOUNT_INSTANCE_OFFSET_BITS)
        {
            Ok(Self { id64 })
        }
        else {
            Err(())
        }
    }

    /// Create a steam id from the parts usually present. The account type will
    /// always be set to a user account.
    pub fn from_parts(universe: Universe, account_type: AccountType, id: u32) -> Self
    {
        let mut id64 = 0;
        id64 |= id as u64;

        // Assume user account
        id64 |= 1 << ACCOUNT_INSTANCE_OFFSET_BITS;

        id64 |= (account_type as u64) << ACCOUNT_TYPE_OFFSET_BITS;
        id64 |= (universe as u64) << UNIVERSE_OFFSET_BITS;

        Self { id64 }
    }

    fn try_for_universe(id64: u64) -> Option<Universe>
    {
        let universe_byte: u8 =
            ((id64 & (0xff << UNIVERSE_OFFSET_BITS)) >> UNIVERSE_OFFSET_BITS) as u8;
        Universe::from_u8(universe_byte)
    }
    fn try_for_account_type(id64: u64) -> Option<AccountType>
    {
        let account_type_nibble: u8 =
            ((id64 & (0xf << ACCOUNT_TYPE_OFFSET_BITS)) >> ACCOUNT_TYPE_OFFSET_BITS) as u8;
        AccountType::from_u8(account_type_nibble)
    }

    /// Get the universe this account is part of.
    ///
    /// # Panics
    /// If there is no valid universe in this steam id, which means that the
    /// internal data is corrupt, which is only possible when creating using the
    /// unsafe `new` method.
    pub fn universe(self) -> Universe
    {
        Self::try_for_universe(self.id64).expect("Corrupted steam id. Check unsafe `new` calls")
    }

    /// Get the type of this account
    ///
    /// # Panics
    /// If the steam id is corrupt. Can only happen with accounts created with
    /// unsafe `new` method.
    pub fn account_type(self) -> AccountType
    {
        Self::try_for_account_type(self.id64).expect("Corrupted steam id. Check unsafe `new` calls")
    }

    pub fn to_id64_string(self) -> String { self.id64.to_string() }

    pub fn to_id3_string(self) -> String
    {
        let mut res = "[".to_owned();
        res.push(self.account_type().into());
        res.push(':');
        let id = self.id64 as u32;
        res += &(id & 1).to_string();
        res.push(':');
        res += &(id >> 1).to_string();
        res.push(']');

        res
    }

    pub fn to_id1_string(self) -> String
    {
        let mut res = "STEAM_".to_owned();
        res += &(self.universe() as u8).to_string();
        res.push(':');
        let id = self.id64 as u32;
        res += &(id & 1).to_string();
        res.push(':');
        res += &(id >> 1).to_string();

        res
    }
}

impl FromStr for SteamID
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        // Try known conversions
        // Starting with steamid64 if it's just a number.
        if let Ok(id64) = s.parse::<u64>() {
            Self::new_checked(id64)
        }
        // Check for ID3
        else if s.starts_with('[') && s.ends_with(']') {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 3 && parts[0].len() == 2 && parts[1].len() == 1 {
                let account_id = {
                    let lowest_bit = parts[1].parse::<u32>().map_err(|_| ())?;
                    if lowest_bit > 1 {
                        return Err(());
                    }

                    let id31upper_bits = parts[2][..s.len() - 1].parse::<u32>().map_err(|_| ())?;
                    id31upper_bits << 1 | lowest_bit
                };
                let account_type: AccountType = parts[0].chars().nth(1).unwrap().try_into()?;
                let universe: Universe = Universe::Public;

                Ok(Self::from_parts(universe, account_type, account_id))
            }
            else {
                Err(())
            }
        }
        // Check for legacy ID format
        else if s.starts_with("STEAM_") {
            todo!()
        }
        // Not a known format
        else {
            Err(())
        }
    }
}

#[derive(Copy, Clone, Debug, Primitive)]
pub enum Universe
{
    Unspecified = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
    RC = 5,
}

#[derive(Copy, Clone, Debug, Primitive)]
pub enum AccountType
{
    Invalid = 0,
    Individual = 1,
    Multiseat = 2,
    GameServer = 3,
    AnonGameServer = 4,
    Pending = 5,
    ContentServer = 6,
    Clan = 7,
    Chat = 8,
    // P2P SuperSeeder ignored
    AnonUser = 10,
}

impl Into<char> for AccountType
{
    fn into(self) -> char
    {
        match self {
            Self::Invalid => 'I',
            Self::Individual => 'U',
            Self::Multiseat => 'M',
            Self::GameServer => 'G',
            Self::AnonGameServer => 'A',
            Self::Pending => 'P',
            Self::ContentServer => 'C',
            Self::Clan => 'g',
            Self::Chat => 'c',
            Self::AnonUser => 'a',
        }
    }
}

impl TryFrom<char> for AccountType
{
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error>
    {
        match value {
            'I' => Ok(Self::Invalid),
            'U' => Ok(Self::Individual),
            'M' => Ok(Self::Multiseat),
            'G' => Ok(Self::GameServer),
            'A' => Ok(Self::AnonGameServer),
            'P' => Ok(Self::Pending),
            'C' => Ok(Self::ContentServer),
            'g' => Ok(Self::Clan),
            'T' | 'L' | 'c' => Ok(Self::Chat),
            'a' => Ok(Self::AnonUser),
            _ => Err(()),
        }
    }
}
