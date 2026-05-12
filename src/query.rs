use sea_query;

#[derive(sea_query::Iden, Clone, Copy)]
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

#[derive(sea_query::Iden, Clone, Copy)]
pub enum StackTrace {
    #[iden = "stacktrace"]
    Table,
    #[iden = "id"]
    ID,
}

#[derive(sea_query::Iden, Clone, Copy)]
pub enum StackTraceElements {
    #[iden = "stacktrace_elements"]
    Table,
    #[iden = "id"]
    ID,
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

#[derive(sea_query::Iden, Clone, Copy)]
pub enum StuckThread {
    #[iden = "stuckthread"]
    Table,
    #[iden = "thread_id"]
    ThreadID,
    #[iden = "start"]
    Start,
    #[iden = "name"]
    Name,
    #[iden = "request"]
    Request,
    #[iden = "active_duration_ms"]
    ActiveDurationMS,
    #[iden = "active_monitor_start"]
    ActiveMonitorStart,
    #[iden = "active_monitor_end"]
    ActiveMonitorEnd,
    #[iden = "stack_id"]
    StackID,
}

#[derive(sea_query::Iden, Clone, Copy)]
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

#[derive(sea_query::Iden, Clone, Copy)]
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
