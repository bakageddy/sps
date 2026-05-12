use sea_query;

#[derive(sea_query::Iden)]
pub enum Object {
    #[iden = "object"]
    Table,
    #[iden = "id"]
    ID,
    #[iden = "class"]
    Class,
    #[iden = "identity"]
    Identity,
}

#[derive(sea_query::Iden)]
pub enum StackTrace {
    #[iden = "stacktrace"]
    Table,

    #[iden = "id"]
    ID,
}

#[derive(sea_query::Iden)]
pub enum StackTraceElements {
    #[iden = "stacktrace_elements"]
    Table,
    #[iden = "frame_idx"]
    FrameIndex,
    #[iden = "method"]
    Method,
    #[iden = "frame_source"]
    FrameSource,
    #[iden = "line_number"]
    LineNumber,
    #[iden = "object_id"]
    ObjectID,
}

#[derive(sea_query::Iden)]
pub enum StuckThread {
    #[iden = "stuckthread"]
    Table,
    #[iden = "start"]
    Start,
    #[iden = "name"]
    Name,
    #[iden = "active_duration_ms"]
    ActiveDurationMS,
    #[iden = "active_monitor_start"]
    ActiveMonitorStart,
    #[iden = "active_monitor_end"]
    ActiveMonitorEnd,
    #[iden = "stack_id"]
    StackID,
}

#[derive(sea_query::Iden)]
pub enum ThreadDump {
    #[iden = "threaddump"]
    Table,
    #[iden = "id"]
    ID,
    #[iden = "snapshot"]
    Snapshot,
    #[iden = "triggered_unix_ms"]
    TriggeredUnixMS,
}

#[derive(sea_query::Iden)]
pub enum Thread {
    #[iden = "thread"]
    Table,
    #[iden = "id"]
    ID,
    #[iden = "name"]
    Name,
    #[iden = "state"]
    State,
    #[iden = "wait_object_id"]
    WaitObjectID,

    #[iden = "owner_id"]
    OwnerID,
    #[iden = "owner_name"]
    OwnerName,
    #[iden = "lock_object_id"]
    LockObjectID,

    #[iden = "stack_id"]
    StackID,
    #[iden = "dump_id"]
    DumpID,
}
