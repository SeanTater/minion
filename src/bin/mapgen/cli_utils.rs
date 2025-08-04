use bevy::prelude::*;
use minion::game_logic::errors::{MinionError, MinionResult};

/// Generic parser for delimited strings that return tuples
pub fn parse_delimited<T, const N: usize>(
    input: &str,
    delimiter: char,
    type_name: &str,
    parser: impl Fn(&str) -> Result<T, std::num::ParseFloatError>,
) -> MinionResult<[T; N]>
where
    T: Copy + Default,
{
    let parts: Vec<&str> = input.split(delimiter).collect();
    if parts.len() != N {
        return Err(MinionError::InvalidMapData {
            reason: format!(
                "Invalid {type_name} format '{input}'. Expected {N} {delimiter}-separated values"
            ),
        });
    }

    let mut result = [T::default(); N];
    for (i, part) in parts.iter().enumerate() {
        result[i] = parser(part).map_err(|_| MinionError::InvalidMapData {
            reason: format!("Invalid {type_name} value: '{part}'"),
        })?;
    }

    Ok(result)
}

/// Parse size string "WIDTHxHEIGHT" with validation
pub fn parse_size(size_str: &str) -> MinionResult<(u32, u32)> {
    let [width, height] = parse_delimited::<f32, 2>(size_str, 'x', "size", |s| s.parse())?;
    let (width, height) = (width as u32, height as u32);

    if width == 0 || height == 0 {
        return Err(MinionError::InvalidMapData {
            reason: "Width and height must be greater than 0".to_string(),
        });
    }

    if width > 2048 || height > 2048 {
        return Err(MinionError::InvalidMapData {
            reason: "Width and height must not exceed 2048".to_string(),
        });
    }

    Ok((width, height))
}

/// Parse position string "X,Y,Z"
pub fn parse_position(pos_str: &str) -> MinionResult<Vec3> {
    let [x, y, z] = parse_delimited::<f32, 3>(pos_str, ',', "position", |s| s.parse())?;
    Ok(Vec3::new(x, y, z))
}

/// Parse scale range string "MIN,MAX" with validation
pub fn parse_scale_range(scale_str: &str) -> MinionResult<(f32, f32)> {
    let [min, max] = parse_delimited::<f32, 2>(scale_str, ',', "scale range", |s| s.parse())?;

    if min <= 0.0 || max <= 0.0 {
        return Err(MinionError::InvalidMapData {
            reason: "Scale values must be positive".to_string(),
        });
    }

    if min > max {
        return Err(MinionError::InvalidMapData {
            reason: "Minimum scale must be less than or equal to maximum scale".to_string(),
        });
    }

    Ok((min, max))
}

/// Parse object types from comma-separated string
pub fn parse_object_types(types_str: &str) -> Vec<String> {
    types_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Validate object density and clamp to valid range
pub fn validate_density(density: f32) -> f32 {
    if !(0.0..=1.0).contains(&density) {
        println!(
            "Warning: Object density {density} is out of range [0.0, 1.0], clamping to valid range"
        );
        density.clamp(0.0, 1.0)
    } else {
        density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("64x64").unwrap(), (64, 64));
        assert_eq!(parse_size("128x256").unwrap(), (128, 256));
        assert_eq!(parse_size("1x1").unwrap(), (1, 1));

        assert!(parse_size("64").is_err());
        assert!(parse_size("0x64").is_err());
        assert!(parse_size("3000x64").is_err());
    }

    #[test]
    fn test_parse_position() {
        assert_eq!(
            parse_position("0.0,1.0,0.0").unwrap(),
            Vec3::new(0.0, 1.0, 0.0)
        );
        assert_eq!(
            parse_position("-5.5,2.3,10.1").unwrap(),
            Vec3::new(-5.5, 2.3, 10.1)
        );

        assert!(parse_position("0.0,1.0").is_err());
        assert!(parse_position("abc,def,ghi").is_err());
    }

    #[test]
    fn test_parse_scale_range() {
        assert_eq!(parse_scale_range("0.8,1.2").unwrap(), (0.8, 1.2));
        assert_eq!(parse_scale_range("1.0,1.0").unwrap(), (1.0, 1.0));

        assert!(parse_scale_range("0.8").is_err());
        assert!(parse_scale_range("0.0,1.2").is_err());
        assert!(parse_scale_range("1.2,0.8").is_err());
    }

    #[test]
    fn test_parse_object_types() {
        assert_eq!(parse_object_types("tree,rock"), vec!["tree", "rock"]);
        assert_eq!(
            parse_object_types("tree, rock, grass"),
            vec!["tree", "rock", "grass"]
        );
        assert_eq!(parse_object_types(""), Vec::<String>::new());
    }

    #[test]
    fn test_validate_density() {
        assert_eq!(validate_density(0.5), 0.5);
        assert_eq!(validate_density(-0.1), 0.0);
        assert_eq!(validate_density(1.5), 1.0);
    }
}
