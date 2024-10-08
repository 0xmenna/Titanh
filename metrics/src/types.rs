use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

use titan_api::{capsules_types::GetCapsuleOpts, common_types::ConsistencyLevel};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Copy)]
pub enum BytesSize {
    Kilobytes(u64),
    Megabytes(u64),
    Gigabytes(u64),
}

impl BytesSize {
    // Helper function to convert all sizes to bytes
    pub fn to_bytes(&self) -> u64 {
        match self {
            BytesSize::Kilobytes(kb) => kb * 1024,
            BytesSize::Megabytes(mb) => mb * 1024 * 1024,
            BytesSize::Gigabytes(gb) => gb * 1024 * 1024 * 1024,
        }
    }

    // Helper function to create BytesSize from bytes
    fn from_bytes(bytes: u64) -> BytesSize {
        if bytes >= 1024 * 1024 * 1024 {
            BytesSize::Gigabytes(bytes / (1024 * 1024 * 1024))
        } else if bytes >= 1024 * 1024 {
            BytesSize::Megabytes(bytes / (1024 * 1024))
        } else {
            BytesSize::Kilobytes(bytes / 1024)
        }
    }
}

// Implementing addition
impl Add for BytesSize {
    type Output = BytesSize;

    fn add(self, other: BytesSize) -> BytesSize {
        let total_bytes = self.to_bytes() + other.to_bytes();
        BytesSize::from_bytes(total_bytes)
    }
}

// Implementing subtraction
impl Sub for BytesSize {
    type Output = BytesSize;

    fn sub(self, other: BytesSize) -> BytesSize {
        let total_bytes = self.to_bytes().saturating_sub(other.to_bytes());
        BytesSize::from_bytes(total_bytes)
    }
}

// Implementing multiplication (multiplying by a scalar)
impl Mul<u64> for BytesSize {
    type Output = BytesSize;

    fn mul(self, rhs: u64) -> BytesSize {
        let total_bytes = self.to_bytes() * rhs;
        BytesSize::from_bytes(total_bytes)
    }
}

// Implementing division (dividing by a scalar)
impl Div<u64> for BytesSize {
    type Output = BytesSize;

    fn div(self, rhs: u64) -> BytesSize {
        let total_bytes = self.to_bytes() / rhs;
        BytesSize::from_bytes(total_bytes)
    }
}

impl FromStr for BytesSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("Input string is empty".to_string());
        }

        // Extract the last character to determine the unit
        let unit = s.chars().last().ok_or("Failed to read last character")?;
        // Extract the numeric part of the string
        let number_str = &s[..s.len() - 1];
        let number: u64 = number_str.parse().map_err(|_| "Failed to parse number")?;

        match unit {
            'K' | 'k' => Ok(BytesSize::Kilobytes(number)),
            'M' | 'm' => Ok(BytesSize::Megabytes(number)),
            'G' | 'g' => Ok(BytesSize::Gigabytes(number)),
            _ => Err(format!(
                "Unknown unit '{}'. Expected 'K', 'M', or 'G'.",
                unit
            )),
        }
    }
}

impl fmt::Display for BytesSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytesSize::Kilobytes(val) => write!(f, "{}K", val),
            BytesSize::Megabytes(val) => write!(f, "{}M", val),
            BytesSize::Gigabytes(val) => write!(f, "{}G", val),
        }
    }
}

pub enum Operation {
    Get(GetCapsuleOpts),
    Put(ConsistencyLevel),
    BatchPut(ConsistencyLevel),
}

impl Operation {
    pub fn consistency_level(&self) -> ConsistencyLevel {
        match self {
            Operation::Get(opts) => ConsistencyLevel::from(*opts),
            Operation::Put(level) => *level,
            Operation::BatchPut(level) => *level,
        }
    }
}

impl From<Operation> for OperationType {
    fn from(op: Operation) -> Self {
        match op {
            Operation::Get(_) => OperationType::Get,
            Operation::Put(_) => OperationType::Put,
            Operation::BatchPut(_) => OperationType::BatchPut,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum OperationType {
    Get,
    Put,
    BatchPut,
}
