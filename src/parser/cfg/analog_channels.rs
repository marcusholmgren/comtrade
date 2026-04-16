use super::ConfigLine;
use crate::error::ComtradeError;
use crate::FormatRevision;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq)]
pub enum AnalogScalingMode {
    Primary,
    Secondary,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AnalogConfig {
    /// 1-indexed counter to determine which channel this is in a COMTRADE record.
    pub index: NonZeroUsize,
    pub name: String,
    pub phase: String,
    pub circuit_component_being_monitored: String,
    pub units: String,
    pub min_value: f64,
    pub max_value: f64,
    /// Use to calculate real values from data points.
    pub multiplier: f64,
    pub offset_adder: f64,
    /// Value in microseconds.
    pub skew: f64,
    /// Used to convert between primary and secondary values in channel.
    pub primary_factor: f64,
    /// Used to convert between primary and secondary values in channel.
    pub secondary_factor: f64,
    pub scaling_mode: AnalogScalingMode,
}

impl AnalogConfig {
    pub fn from_cfg_row<'a>(
        mut config_line: impl ConfigLine<'a>,
        revision: &FormatRevision,
    ) -> Result<Self, ComtradeError> {
        let index = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Index"))?;
        let name = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Name"))?;
        let phase = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Phase"))?;
        let circuit_component_being_monitored = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Component"))?;
        let units = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Units"))?;
        let multiplier = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Multiplier"))?;
        let offset_adder = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Offset Adder"))?;
        let skew = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Skew"))?;
        let min_value = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Min Value"))?;
        let max_value = config_line
            .read_value()
            .map_err(|e| e.add_context("Analog Channel: Max Value"))?;

        let primary_factor;
        let secondary_factor;
        let scaling_mode;

        if *revision == FormatRevision::Revision1991 {
            primary_factor = 1.0;
            secondary_factor = 1.0;
            scaling_mode = AnalogScalingMode::Primary;
        } else {
            primary_factor = config_line
                .read_value()
                .map_err(|e| e.add_context("Analog Channel: Primary Factor"))?;
            secondary_factor = config_line
                .read_value()
                .map_err(|e| e.add_context("Analog Channel: Secondary Factor"))?;
            scaling_mode = config_line
                .read_value()
                .map_err(|e| e.add_context("Analog Channel: Scaling Mode"))?;
        }

        Ok(Self {
            index,
            name,
            phase,
            circuit_component_being_monitored,
            units,
            min_value,
            max_value,
            multiplier,
            offset_adder,
            skew,
            primary_factor,
            secondary_factor,
            scaling_mode,
        })
    }
}
