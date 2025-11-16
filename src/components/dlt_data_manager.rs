use rand::random;
use regex::Regex;
use std::sync::Mutex;

use crate::types::{FrontDltAppIdItem, FrontDltEcuItem};

#[derive(Debug, Clone)]
pub struct DltDataChartItem{
    pub id: usize,
    pub x_label: String,
    pub y_label: String,
    pub description: String,
    pub data_points: Vec<DltDataChartPointItem>,
}

#[derive(Debug, Clone)]
pub struct DltDataChartPointItem{
    pub x_value: f32,
    pub y_value: f32,
}

#[derive(Debug, Clone)]
pub struct DltDataGattChartItem{
    pub id: usize,
    pub label: String,
    pub description: String,
    pub point_items: Vec<DltDataGattChartPointItem>,
}

#[derive(Debug, Clone)]
pub struct DltDataGattChartPointItem{
    start_time: f32,
    end_time: f32,
    color: [f32; 3],
}

#[derive(Debug, Clone)]
pub enum DltDataModuleItem{
    Chart(DltDataChartItem),
    GattChart(DltDataGattChartItem),
}

#[derive(Debug, Clone)]
pub struct DltDataRegexItem{
    pub id: usize,
    pub regex: String,
    pub description: String,
    pub item_type: DltDataModuleItem,
}

impl DltDataRegexItem {
    pub fn get_chart_item(&self) -> Option<DltDataChartItem> {
        match &self.item_type {
            DltDataModuleItem::Chart(chart) => Some(chart.clone()),
            DltDataModuleItem::GattChart(_) => None,
        }
    }
    
    pub fn get_gatt_chart_item(&self) -> Option<DltDataGattChartItem> {
        match &self.item_type {
            DltDataModuleItem::GattChart(gatt_chart) => Some(gatt_chart.clone()),
            DltDataModuleItem::Chart(_) => None,
        }
    }
    
    pub fn get_item(&self) -> &DltDataModuleItem {
        &self.item_type
    }
}


pub static DLT_DATA_REGEX_STORE: Mutex<Vec<DltDataRegexItem>> = Mutex::new(Vec::new());

pub fn analzye_dlt_data_regex(dlt_payload: String) {
    let mut store = DLT_DATA_REGEX_STORE.lock().unwrap();

    for regex_item in store.iter_mut() {
        let regex = Regex::new(&regex_item.regex).unwrap();

        if regex.is_match(&dlt_payload) {
            match &mut regex_item.item_type {
                DltDataModuleItem::Chart(chart_item) => {
                    // extract x,y data and process
                    let captures = regex.captures(&dlt_payload).unwrap();
                    let x_value: f32 = captures.get(1).unwrap().as_str().parse().unwrap();
                    let y_value: f32 = captures.get(2).unwrap().as_str().parse().unwrap();

                    chart_item.data_points.push(DltDataChartPointItem {
                        x_value,
                        y_value,
                    });
                },
                DltDataModuleItem::GattChart(gatt_chart_item) => {
                    let captures = regex.captures(&dlt_payload).unwrap();
                    let start_time: f32 = captures.get(1).unwrap().as_str().parse().unwrap();
                    let end_time: f32 = captures.get(2).unwrap().as_str().parse().unwrap();
                    let color = random::<[f32; 3]>();

                    gatt_chart_item.point_items.push(DltDataGattChartPointItem {
                        start_time,
                        end_time,
                        color,
                    });
                },
            }
        }
    }
}

pub fn add_dlt_data_regex_item(regex: String, description: String, item_type: DltDataModuleItem) {
    let mut store = DLT_DATA_REGEX_STORE.lock().unwrap();
    let new_id = store.len() + 1;

    let new_item = DltDataRegexItem {
        id: new_id,
        regex,
        description,
        item_type,
    };

    store.push(new_item);
}

pub fn apply_app_update(ecuid: String, ecu_list: &mut Vec<FrontDltEcuItem>, app_info: &FrontDltAppIdItem) -> Result<(), String> {
    let ecu_apps = ecu_list
        .iter_mut()  // Changed from .iter() to .iter_mut()
        .find(|ecu| ecu.ecuid == ecuid)
        .ok_or_else(|| format!("ECU {} not found", ecuid))?
        .app_ids
        .iter_mut()  // Changed from .iter() to .iter_mut()
        .find(|app| app.apid == app_info.apid)
        .ok_or_else(|| format!("App ID {} not found for ECU {}", app_info.apid, ecuid))?;
    
    // Update the app info
    *ecu_apps = app_info.clone();
    
    Ok(())
}

