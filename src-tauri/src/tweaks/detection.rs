use super::detectors::detect_tweak_state;
use super::metadata::DetectedState;
use super::model::TweakDefinition;

pub fn detect_tweak(def: &TweakDefinition) -> DetectedState {
    detect_tweak_state(def)
}
