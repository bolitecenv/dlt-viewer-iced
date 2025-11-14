

#[derive(Debug, Clone, PartialEq)]
pub struct FrontDltEcuItem {
    pub ecuid: String,
    pub description: String,
    pub app_ids: Vec<FrontDltAppIdItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrontDltAppIdItem {
    pub apid: String,
    pub description: String,
    pub ctx_ids: Vec<FrontDltCtxIdItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrontDltCtxIdItem {
    pub context_id: String,
    pub description: String,
    pub log_level: i8,
    pub trace_status: i8,
}