#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl Rotation {
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Rotation::North),
            1 => Some(Rotation::NorthEast),
            2 => Some(Rotation::East),
            3 => Some(Rotation::SouthEast),
            4 => Some(Rotation::South),
            5 => Some(Rotation::SouthWest),
            6 => Some(Rotation::West),
            7 => Some(Rotation::NorthWest),
            _ => None,
        }
    }
    
    pub fn to_value(&self) -> i32 {
        *self as i32
    }
}