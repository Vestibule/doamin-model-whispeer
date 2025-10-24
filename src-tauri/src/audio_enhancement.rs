use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Configuration pour l'amélioration audio
#[derive(Debug, Clone)]
pub struct AudioEnhancementConfig {
    /// Force de réduction du bruit (0.0 = aucun, 1.0 = maximum)
    pub noise_reduction: f32,
    /// Activer le high-pass filter (coupe les basses fréquences)
    pub enable_highpass: bool,
    /// Normalisation du volume
    pub normalize: bool,
}

impl Default for AudioEnhancementConfig {
    fn default() -> Self {
        Self {
            noise_reduction: 0.21, // Modéré
            enable_highpass: true,
            normalize: true,
        }
    }
}

/// Module d'amélioration audio utilisant ffmpeg
pub struct AudioEnhancer {
    _config: AudioEnhancementConfig,
}

impl AudioEnhancer {
    /// Crée un nouveau enhancer avec la configuration donnée
    pub fn new(_sample_rate: u32, config: AudioEnhancementConfig) -> Result<Self> {
        // Vérifier que ffmpeg est disponible
        Command::new("ffmpeg")
            .arg("-version")
            .output()
            .context("ffmpeg not found. Please install ffmpeg: brew install ffmpeg")?;

        Ok(Self { _config: config })
    }

    /// Traite un fichier WAV entier et le sauvegarde
    pub fn process_file(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        // Construire la chaîne de filtres ffmpeg
        let mut filters = Vec::new();
        
        // Highpass filter (coupe les basses fréquences < 200Hz)
        if self._config.enable_highpass {
            filters.push("highpass=f=200".to_string());
        }
        
        // Réduction de bruit avec afftdn (FFT Denoiser)
        if self._config.noise_reduction > 0.0 {
            filters.push(format!("afftdn=nr={}", self._config.noise_reduction * 40.0));
        }
        
        // Normalisation du volume
        if self._config.normalize {
            filters.push("dynaudnorm=f=150:g=15".to_string());
        }
        
        let filter_chain = filters.join(",");
        
        // Exécuter ffmpeg avec conversion à 16kHz pour Whisper
        let output = Command::new("ffmpeg")
            .arg("-i").arg(input_path)
            .arg("-af").arg(&filter_chain)
            .arg("-ar").arg("16000") // Resample à 16kHz pour Whisper
            .arg("-ac").arg("1") // Mono
            .arg("-y") // Overwrite output file
            .arg(output_path)
            .output()
            .context("Failed to run ffmpeg")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ffmpeg failed: {}", stderr);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhancer_creation() {
        let config = AudioEnhancementConfig::default();
        let result = AudioEnhancer::new(48000, config);
        assert!(result.is_ok());
    }
}
