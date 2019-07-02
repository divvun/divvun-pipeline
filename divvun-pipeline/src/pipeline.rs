pub struct PipelineCall {
    pub module_name: String,
    pub command_name: String,
}

pub enum PipelineItem {
    Single(PipelineCall),
    Serial(Vec<PipelineItem>),
    Parallel(Vec<PipelineItem>),
}

pub struct Pipeline {
    pub root_item: PipelineItem,
}
