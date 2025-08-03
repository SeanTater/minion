use minion::game_logic::errors::{MinionError, MinionResult};
use minion::terrain_generation::{TerrainAlgorithm, TerrainGenerator, get_terrain_preset};

pub struct TerrainBuilder {
    terrain_type: String,
    seed: Option<u32>,
    amplitude: f32,
    frequency: f32,
    octaves: u32,
}

impl TerrainBuilder {
    pub fn new(terrain_type: String) -> Self {
        Self {
            terrain_type,
            seed: None,
            amplitude: 10.0,
            frequency: 0.1,
            octaves: 4,
        }
    }

    pub fn seed(mut self, seed: Option<u32>) -> Self {
        self.seed = seed;
        self
    }

    pub fn amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves;
        self
    }

    /// Check if any manual parameters differ from defaults
    fn has_manual_parameters(&self) -> bool {
        self.amplitude != 10.0 || self.frequency != 0.1 || self.octaves != 4
    }

    pub fn build(self) -> MinionResult<TerrainGenerator> {
        let seed = self.seed.unwrap_or_else(rand::random);
        let manual_params = self.has_manual_parameters();
        
        // Try preset first, then fall back to custom algorithm
        if let Some(mut generator) = get_terrain_preset(&self.terrain_type, self.seed) {
            if manual_params {
                generator.algorithm = self.override_preset_params(generator.algorithm)?;
            }
            return Ok(generator);
        }

        // Create custom algorithm
        let algorithm = match self.terrain_type.as_str() {
            "perlin" => TerrainAlgorithm::Perlin {
                amplitude: self.amplitude,
                frequency: self.frequency,
                octaves: self.octaves,
            },
            "ridged" => TerrainAlgorithm::Ridged {
                amplitude: self.amplitude,
                frequency: self.frequency,
                octaves: self.octaves,
            },
            _ => {
                return Err(MinionError::InvalidMapData {
                    reason: format!(
                        "Unknown terrain type: '{}'. Available presets: flat, hills, mountains, valleys. Custom algorithms: perlin, ridged",
                        self.terrain_type
                    ),
                });
            }
        };

        Ok(TerrainGenerator::new(seed, algorithm))
    }

    fn override_preset_params(&self, algorithm: TerrainAlgorithm) -> MinionResult<TerrainAlgorithm> {
        match algorithm {
            TerrainAlgorithm::Flat { height } => {
                println!("Warning: Manual terrain parameters (amplitude, frequency, octaves) are ignored for 'flat' terrain type");
                Ok(TerrainAlgorithm::Flat { height })
            },
            TerrainAlgorithm::Perlin { .. } => {
                println!("Using custom parameters with '{}' terrain type: amplitude={}, frequency={}, octaves={}", 
                    self.terrain_type, self.amplitude, self.frequency, self.octaves);
                Ok(TerrainAlgorithm::Perlin {
                    amplitude: self.amplitude,
                    frequency: self.frequency,
                    octaves: self.octaves,
                })
            },
            TerrainAlgorithm::Ridged { .. } => {
                println!("Using custom parameters with '{}' terrain type: amplitude={}, frequency={}, octaves={}", 
                    self.terrain_type, self.amplitude, self.frequency, self.octaves);
                Ok(TerrainAlgorithm::Ridged {
                    amplitude: self.amplitude,
                    frequency: self.frequency,
                    octaves: self.octaves,
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_builder_default() {
        let builder = TerrainBuilder::new("hills".to_string())
            .seed(Some(12345));
        
        let generator = builder.build().unwrap();
        assert_eq!(generator.seed, 12345);
    }

    #[test]
    fn test_terrain_builder_with_manual_params() {
        let builder = TerrainBuilder::new("hills".to_string())
            .seed(Some(12345))
            .amplitude(100.0)
            .frequency(0.2)
            .octaves(6);
        
        let generator = builder.build().unwrap();
        assert_eq!(generator.seed, 12345);

        match generator.algorithm {
            TerrainAlgorithm::Perlin { amplitude, frequency, octaves } => {
                assert_eq!(amplitude, 100.0);
                assert_eq!(frequency, 0.2);
                assert_eq!(octaves, 6);
            },
            _ => panic!("Expected Perlin algorithm"),
        }
    }

    #[test]
    fn test_terrain_builder_custom_algorithm() {
        let builder = TerrainBuilder::new("perlin".to_string())
            .seed(Some(12345))
            .amplitude(25.0)
            .frequency(0.05)
            .octaves(8);
        
        let generator = builder.build().unwrap();
        assert_eq!(generator.seed, 12345);

        match generator.algorithm {
            TerrainAlgorithm::Perlin { amplitude, frequency, octaves } => {
                assert_eq!(amplitude, 25.0);
                assert_eq!(frequency, 0.05);
                assert_eq!(octaves, 8);
            },
            _ => panic!("Expected Perlin algorithm"),
        }
    }

    #[test]
    fn test_terrain_builder_unknown_type() {
        let builder = TerrainBuilder::new("unknown".to_string());
        assert!(builder.build().is_err());
    }
}