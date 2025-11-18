use rand::random;
use regex::Regex;
use std::sync::Mutex;

use crate::types::{FrontDltAppIdItem, FrontDltEcuItem};

#[derive(Debug, Clone)]
pub struct DltDataChartItem {
    pub id: usize,
    pub x_label: String,
    pub y_label: String,
    pub description: String,
    pub data_points: Vec<DltDataChartPointItem>,
}

#[derive(Debug, Clone)]
pub struct DltDataChartPointItem {
    pub x_value: f32,
    pub y_value: f32,
}

#[derive(Debug, Clone)]
pub struct DltDataGattChartItem {
    pub id: usize,
    pub label: String,
    pub description: String,
    pub point_items: Vec<DltDataGattChartPointItem>,
}

#[derive(Debug, Clone)]
pub struct DltDataGattChartPointItem {
    start_time: f32,
    end_time: f32,
    color: [f32; 3],
}

#[derive(Debug, Clone)]
pub enum DltDataModuleItem {
    Chart(DltDataChartItem),
    GattChart(DltDataGattChartItem),
}

#[derive(Debug, Clone)]
pub struct DltDataRegexItem {
    pub id: usize,
    pub regex: String,
    pub description: String,
}

impl DltDataRegexItem {
}

pub fn apply_app_update(
    ecuid: String,
    ecu_list: &mut Vec<FrontDltEcuItem>,
    app_info: &FrontDltAppIdItem,
) -> Result<(), String> {
    let ecu_apps = ecu_list
        .iter_mut() // Changed from .iter() to .iter_mut()
        .find(|ecu| ecu.ecuid == ecuid)
        .ok_or_else(|| format!("ECU {} not found", ecuid))?
        .app_ids
        .iter_mut() // Changed from .iter() to .iter_mut()
        .find(|app| app.apid == app_info.apid)
        .ok_or_else(|| format!("App ID {} not found for ECU {}", app_info.apid, ecuid))?;

    // Update the app info
    *ecu_apps = app_info.clone();

    Ok(())
}
