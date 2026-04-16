use crate::error::ComtradeError;
use crate::parser::cfg::ConfigLine;
use crate::FormatRevision;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusConfig {
    pub index: NonZeroUsize,
    pub name: String,
    pub phase: String,
    pub circuit_component_being_monitored: String,
    pub normal_status_value: u8,
}

impl StatusConfig {
    pub fn from_config_row<'a>(
        mut config_line: impl ConfigLine<'a>,
        revision: &FormatRevision,
    ) -> Result<Self, ComtradeError> {
        let status_index = config_line
            .read_value()
            .map_err(|e| e.add_context("Status Channel: Index"))?;
        let name = config_line
            .read_value()
            .map_err(|e| e.add_context("Status Channel: Name"))?;

        let phase;
        let circuit_component_being_monitored;
        let normal_status_value;

        if *revision == FormatRevision::Revision1991 {
            phase = String::new();
            circuit_component_being_monitored = String::new();
            normal_status_value = config_line
                .read_value()
                .map_err(|e| e.add_context("Status Channel: Normal Value"))?;
        } else {
            phase = config_line
                .read_value()
                .map_err(|e| e.add_context("Status Channel: Phase"))?;
            circuit_component_being_monitored = config_line
                .read_value()
                .map_err(|e| e.add_context("Status Channel: Component"))?;
            normal_status_value = config_line
                .read_value()
                .map_err(|e| e.add_context("Status Channel: Normal Value"))?;
        }

        if normal_status_value != 0 && normal_status_value != 1 {
            return Err(ComtradeError::InvalidNormalStatus(status_index));
        }
        Ok(Self {
            index: status_index,
            name,
            phase,
            circuit_component_being_monitored,
            normal_status_value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::cfg::split_cfg_line;

    #[test]
    fn errors_on_invalid_standard_status_value() {
        let line = "3, name, phase, component, 2";
        let result =
            StatusConfig::from_config_row(split_cfg_line(line), &FormatRevision::Revision2013);
        assert!(result.is_err());
    }
}
