pub mod cpumemstats;
pub mod cpumonitoring;
pub mod stuckthread;
pub mod stuckquery;
pub mod error;
pub mod tokenizer;

pub enum DBKind {
    PGSQL,
    MSSQL,
}

/// SQL Server wait types (sys.dm_os_wait_stats), generated from the
/// Microsoft docs table; descriptions are verbatim from that table.
/// Microsoft ADDS wait types every release, so parsing NEVER fails:
/// names not in this list land in Unknown(String).
#[allow(non_camel_case_types)] // variants mirror the wire strings 1:1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WaitType {
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    ABR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    AM_INDBUILD_ALLOCATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    AM_SCHEMAMGR_UNSHARED_CACHE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    ASSEMBLY_FILTER_HASHTABLE,
    /// Occurs during exclusive access to assembly loading.
    ASSEMBLY_LOAD,
    /// Occurs when there's an attempt to synchronize parallel threads that are performing tasks such as creating or initializing a file.
    ASYNC_DISKPOOL_LOCK,
    /// Occurs when a task is waiting for asynchronous non-data I/Os to finish. Examples include I/O involved in warm standby log shipping, database mirroring, some bulk import related operations.
    ASYNC_IO_COMPLETION,
    /// Occurs on network writes when the task is blocked waiting for the client application to acknowledge that it has processed all the data sent to it. Verify that the client application is processing data from the server as fast as possible or that no network delays exist. Reasons the client application can't consume data fast enough include: application design issues like writing results to a file while the results arrive, waiting for user input, client-side filtering on a large dataset instead of server-side filtering, or an intentional wait introduced. Also the client computer might be experiencing slow response due to issues like low virtual/physical memory, 100% CPU consumption, etc. Network delays can also lead to this wait - typically caused by network adapter driver issues, filter drivers, firewalls, or misconfigured routers.
    ASYNC_NETWORK_IO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    ASYNC_OP_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    ASYNC_OP_CONTEXT_READ,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    ASYNC_OP_CONTEXT_WRITE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    ASYNC_SOCKETDUP_IO,
    /// Occurs when there's a wait on a lock that controls access to a special cache. The cache contains information about which audits are being used to audit each audit action group.
    AUDIT_GROUPCACHE_LOCK,
    /// Occurs when there's a wait on a lock that controls access to a special cache. The cache contains information about which audits are being used to audit login audit action groups.
    AUDIT_LOGINCACHE_LOCK,
    /// Occurs when there's a wait on a lock that is used to ensure single initialization of audit related Extended Event targets.
    AUDIT_ON_DEMAND_TARGET_LOCK,
    /// Occurs when there's a wait on a lock that is used to synchronize the starting and stopping of audit related Extended Events sessions.
    AUDIT_XE_SESSION_MGR,
    /// Occurs when a task is blocked as part of backup processing.
    BACKUP,
    /// Occurs when a backup task is waiting for data, or is waiting for a buffer in which to store data. This type isn't typical, except when a task is waiting for a tape mount.
    BACKUPBUFFER,
    /// Occurs when a backup task is waiting for data, or is waiting for a buffer in which to store data. This type isn't typical, except when a task is waiting for a tape mount.
    BACKUPIO,
    /// Occurs when a task is waiting for a backup task to finish. Wait times might be long, from several minutes to several hours. If the task that is being waited on is in an I/O process, this type doesn't indicate a problem.
    BACKUPTHREAD,
    /// Occurs when a task is waiting for a tape mount. To view the tape status, query sys.dm_io_backup_tapes. If a mount operation isn't pending, this wait type might indicate a hardware problem with the tape drive.
    BACKUP_OPERATOR,
    /// Occurs when the background suspect page logger is trying to avoid running more than every five seconds. Excessive suspect pages cause the logger to run frequently.
    BAD_PAGE_PROCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    BLOB_METADATA,
    /// Occurs with parallel batch-mode plans when synchronizing the allocation of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BMPALLOCATION,
    /// Occurs with parallel batch-mode plans when synchronizing the building of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BMPBUILD,
    /// Occurs with parallel batch-mode plans when synchronizing the repartitioning of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BMPREPARTITION,
    /// Occurs with parallel batch-mode plans when synchronizing the replication of a large bitmap filter across worker threads. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BMPREPLICATION,
    /// Occurs with parallel batch-mode plans when synchronizing the sorting of a dataset across multiple threads. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    BPSORT,
    /// Occurs when waiting for access to receive a message on a connection endpoint. Receive access to the endpoint is serialized.
    BROKER_CONNECTION_RECEIVE_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    BROKER_DISPATCHER,
    /// Occurs when there's contention to access the state of a Service Broker connection endpoint. Access to the state for changes is serialized.
    BROKER_ENDPOINT_STATE_MUTEX,
    /// Occurs when a task is waiting in the primary event handler of the Service Broker. This should occur very briefly.
    BROKER_EVENTHANDLER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    BROKER_FORWARDER,
    /// Occurs when initializing Service Broker in each active database. This should occur infrequently.
    BROKER_INIT,
    /// Occurs when a task is waiting for the primary event handler of the Service Broker to start. This should occur very briefly.
    BROKER_MASTERSTART,
    /// Occurs when the RECEIVE WAITFOR is waiting. This might mean that either no messages are ready to be received in the queue or a lock contention is preventing it from receiving messages from the queue.
    BROKER_RECEIVE_WAITFOR,
    /// Occurs during the initialization of a Service Broker connection endpoint. This should occur very briefly.
    BROKER_REGISTERALLENDPOINTS,
    /// Occurs when the Service Broker destination list that is associated with a target service is updated or reprioritized.
    BROKER_SERVICE,
    /// Occurs when there's a planned shutdown of Service Broker. This should occur very briefly, if at all.
    BROKER_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    BROKER_START,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BROKER_TASK_SHUTDOWN,
    /// Occurs when the Service Broker queue task handler tries to shut down the task. The state check is serialized and must be in a running state beforehand.
    BROKER_TASK_STOP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    BROKER_TASK_SUBMIT,
    /// Occurs when the Service Broker lazy flusher flushes the in-memory transmission objects to a work table.
    BROKER_TO_FLUSH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    BROKER_TRANSMISSION_OBJECT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    BROKER_TRANSMISSION_TABLE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    BROKER_TRANSMISSION_WORK,
    /// Occurs when the Service Broker transmitter is waiting for work. Service Broker has a component known as the Transmitter, which schedules messages from multiple dialogs to be sent across the wire over one or more connection endpoints. The transmitter has two dedicated threads for this purpose. This wait type is charged when these transmitter threads are waiting for dialog messages to be sent using the transport connections. High values of waiting_tasks_count for this wait type point to intermittent work for these transmitter threads and aren't indications of any performance problem. If service broker isn't used at all, waiting_tasks_count should be 2 (for the two transmitter threads), and wait_time_ms should be twice the duration since instance startup. See Service broker wait stats.
    BROKER_TRANSMITTER,
    /// Might occur when the buffer pool scan runs in parallel and the main task waits for the scan to complete. For more information, see Operations that trigger a buffer pool scan may run slowly on large-memory computers.
    ///
    /// Applies to: SQL Server 2022 (16.x) and later versions.
    BUFFERPOOL_SCAN,
    /// Might occur after startup of instance, while internal data structures are initializing. Doesn't recur once data structures have initialized.
    BUILTIN_HASHKEY_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    CHANGE_TRACKING_WAITFORCHANGES,
    /// Occurs while the checkpoint task is waiting for the next checkpoint request.
    CHECKPOINT_QUEUE,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    CHECK_PRINT_RECORD,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    CHECK_SCANNER_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    CHECK_TABLES_INITIALIZATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    CHECK_TABLES_SINGLE_SCAN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    CHECK_TABLES_THREAD_BARRIER,
    /// Occurs at server startup to tell the checkpoint thread that it can start.
    CHKPT,
    /// Occurs during operations that change the state of a database, such as opening or closing a database.
    CLEAR_DB,
    /// Occurs where there's a wait to acquire exclusive access to the CLR-hosting data structures. This wait type occurs while setting up or tearing down the CLR runtime.
    CLRHOST_STATE_ACCESS,
    /// Occurs when a task is currently performing common language runtime (CLR) execution and is waiting for a particular autoevent to be initiated. Long waits are typical, and don't indicate a problem.
    CLR_AUTO_EVENT,
    /// Occurs when a task is currently performing CLR execution, and is waiting to enter a critical section of the task that is currently being used by another task.
    CLR_CRST,
    /// Occurs when a task is currently performing CLR execution, and is waiting for another task to end. This wait state occurs when there's a join between tasks.
    CLR_JOIN,
    /// Occurs when a task is currently performing CLR execution, and is waiting for a specific manual event to be initiated.
    CLR_MANUAL_EVENT,
    /// Occurs during a wait on lock acquisition for a data structure that is used to record all virtual memory allocations that come from CLR. The data structure is locked to maintain its integrity if there's parallel access.
    CLR_MEMORY_SPY,
    /// Occurs when a task is currently performing CLR execution, and is waiting to obtain a lock on the monitor.
    CLR_MONITOR,
    /// Occurs when a task is currently performing CLR execution, and is waiting for a reader lock.
    CLR_RWLOCK_READER,
    /// Occurs when a task is currently performing CLR execution, and is waiting for a writer lock.
    CLR_RWLOCK_WRITER,
    /// Occurs when a task is currently performing CLR execution, and is waiting for a semaphore.
    CLR_SEMAPHORE,
    /// Occurs while waiting for a CLR task to complete startup.
    CLR_TASK_START,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    CMEMPARTITIONED,
    /// Occurs when a task is waiting on a thread-safe memory object. The wait time might increase when there's contention caused by multiple tasks trying to allocate memory from the same memory object.
    CMEMTHREAD,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    COLUMNSTORE_BUILD_THROTTLE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    COLUMNSTORE_COLUMNDATASET_SESSION_LIST,
    /// Internal use only.
    COMMIT_TABLE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    CONNECTION_ENDPOINT_LOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    COUNTRECOVERYMGR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    CREATE_DATINISERVICE,
    /// Occurs with parallel query plans when a consumer thread (parent) waits for a producer thread to send rows. CXCONSUMER waits are caused by an Exchange Iterator that runs out of rows from its producer thread. This is a normal part of parallel query execution.
    ///
    /// Applies to: SQL Server (Starting with SQL Server 2016 (13.x) Service Pack 2, SQL Server 2017 (14.x) CU 3), Azure SQL Database, Azure SQL Managed Instance
    CXCONSUMER,
    /// Occurs with parallel query plans when waiting to synchronize the Query Processor Exchange Iterator, and when producing and consuming rows. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Note: Starting with SQL Server 2016 (13.x) Service Pack 2 and SQL Server 2017 (14.x) CU 3, CXPACKET only refers to waiting to synchronize the Exchange Iterator and producing rows. Threads consuming rows are tracked separately in the CXCONSUMER wait type. If the consumer threads are too slow, the Exchange Iterator buffer might become full and cause CXPACKET waits.
    ///
    /// Note: In SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance, CXPACKET only refers to waiting on threads producing rows. Exchange Iterator synchronization is tracked separately in the CXSYNC_PORT and CXSYNC_CONSUMER wait types. Threads consuming rows are tracked separately in the CXCONSUMER wait type.
    CXPACKET,
    /// Occurs during a parallel range scan.
    CXROWSET_SYNC,
    /// Occurs with parallel query plans when waiting to reach an Exchange Iterator synchronization point among all consumer threads.
    ///
    /// Applies to: SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance
    CXSYNC_CONSUMER,
    /// Occurs with parallel query plans when waiting to open, close, and synchronize Exchange Iterator ports between producer and consumer threads. For example, if a query plan has a long sort operation, CXSYNC_PORT waits might be higher because the sort must complete before the Exchange Iterator port can be synchronized.
    ///
    /// Applies to: SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance
    CXSYNC_PORT,
    /// Occurs while the dedicated administrator connection is initializing.
    DAC_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    DBCC_SCALE_OUT_EXPR_CACHE,
    /// Occurs when a task is waiting for log records to be flushed to disk. This wait state is expected to be held for long periods of time.
    DBMIRRORING_CMD,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    DBMIRROR_DBM_EVENT,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    DBMIRROR_DBM_MUTEX,
    /// Occurs when database mirroring waits for events to process.
    DBMIRROR_EVENTS_QUEUE,
    /// Occurs when a task is waiting for a communications backlog at the network layer to clear to be able to send messages. Indicates that the communications layer is starting to become overloaded and affect the database mirroring data throughput.
    DBMIRROR_SEND,
    /// Indicates that the database mirroring worker task is waiting for more work.
    DBMIRROR_WORKER_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    DBSEEDING_FLOWCONTROL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    DBSEEDING_OPERATION,
    /// Occurs when the deadlock monitor and sys.dm_os_waiting_tasks try to make sure that SQL Server isn't running multiple deadlock searches at the same time.
    DEADLOCK_ENUM_MUTEX,
    /// Large waiting time on this resource indicates that the server is executing queries on top of sys.dm_os_waiting_tasks, and these queries are blocking deadlock monitor from running deadlock search. This wait type is used by deadlock monitor only. Queries on top of sys.dm_os_waiting_tasks use DEADLOCK_ENUM_MUTEX.
    DEADLOCK_TASK_SEARCH,
    /// Occurs during Transact-SQL and CLR debugging for internal synchronization.
    DEBUG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DIRECTLOGCONSUMER_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    DIRTY_PAGE_POLL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    DIRTY_PAGE_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DIRTY_PAGE_TABLE_LOCK,
    /// Occurs when SQL Server polls the version transaction manager to see whether the timestamp of the earliest active transaction is later than the timestamp of when the state started changing. If this is this case, all the snapshot transactions that were started before the ALTER DATABASE statement was run have finished. This wait state is used when SQL Server disables versioning by using the ALTER DATABASE statement.
    DISABLE_VERSIONING,
    /// Occurs when a task is waiting to access a file when an external backup is active. This is reported for each waiting user process. A count larger than five per user process might indicate that the external backup is taking too much time to finish.
    DISKIO_SUSPEND,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    DISPATCHER_PRIORITY_QUEUE_SEMAPHORE,
    /// Occurs when a thread from the dispatcher pool is waiting for more work to process. The wait time for this wait type is expected to increase when the dispatcher is idle.
    DISPATCHER_QUEUE_SEMAPHORE,
    /// Occurs once while waiting for the XML parser DLL to load.
    DLL_LOADING_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DPT_ENTRY_LOCK,
    /// Occurs between attempts to drop a temporary object if the previous attempt failed. The wait duration grows exponentially with each failed drop attempt.
    DROPTEMP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    DROP_DATABASE_TIMER_TASK,
    /// Occurs when a task is waiting on an event that is used to manage state transition. This state controls when the recovery of Microsoft Distributed Transaction Coordinator (MS DTC) transactions occurs after SQL Server receives notification that the MS DTC service has become unavailable.
    DTC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DTCNEW_ENLIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DTCNEW_PREPARE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DTCNEW_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DTCNEW_TM,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    DTCNEW_TRANSACTION_ENLISTMENT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    DTCPNTSYNC,
    /// Occurs in an MSDTC worker session when the session is waiting to take ownership of an MSDTC transaction. After MS DTC owns the transaction, the session can roll back the transaction. Generally, the session waits for another session that is using the transaction.
    DTC_ABORT_REQUEST,
    /// Occurs when a recovery task is waiting for the master database in a cross-database transaction so that the task can query the outcome of the transaction.
    DTC_RESOLVE,
    /// Occurs when a task is waiting on an event that protects changes to the internal MS DTC global state object. This state should be held for very short periods of time.
    DTC_STATE,
    /// Occurs in an MSDTC worker session when SQL Server receives notification that the MS DTC service isn't available. First, the worker waits for the MS DTC recovery process to start. Then, the worker waits to obtain the outcome of the distributed transaction that the worker is working on. This might continue until the connection with the MS DTC service has been reestablished.
    DTC_TMDOWN_REQUEST,
    /// Occurs when recovery tasks wait for MS DTC to become active to enable the resolution of prepared transactions.
    DTC_WAITFOR_OUTCOME,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    DUMPTRIGGER,
    /// Occurs when a main task is waiting for a subtask to generate data. Ordinarily, this state doesn't occur. A long wait indicates an unexpected blockage. The subtask should be investigated.
    DUMP_LOG_COORDINATOR,
    /// Internal use only.
    DUMP_LOG_COORDINATOR_QUEUE,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    EC,
    /// Occurs during synchronization of certain types of memory allocations during statement execution.
    EE_PMOLOCK,
    /// Occurs during synchronization of internal procedure hash table creation. This wait can only occur during the initial accessing of the hash table after the SQL Server instance starts.
    EE_SPECPROC_MAP_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    ENABLE_EMPTY_VERSIONING,
    /// Occurs when SQL Server waits for all update transactions in this database to finish before declaring the database ready to transition to snapshot isolation allowed state. This state is used when SQL Server enables snapshot isolation by using the ALTER DATABASE statement.
    ENABLE_VERSIONING,
    /// Occurs during synchronization of multiple concurrent error log initializations.
    ERROR_REPORTING_MANAGER,
    /// Occurs during synchronization in the query processor exchange iterator during parallel queries.
    EXCHANGE,
    /// Occurs during parallel queries while synchronizing in query processor in areas not related to the exchange iterator. Examples of such areas are bitmaps, large binary objects (LOBs), and the spool iterator. LOBs might frequently use this wait state.
    EXECSYNC,
    /// Occurs during synchronization between producer and consumer parts of batch execution that are submitted through the connection context.
    EXECUTION_PIPE_EVENT_INTERNAL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    EXTERNAL_RG_UPDATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) through current.
    EXTERNAL_SCRIPT_NETWORK_IO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    EXTERNAL_SCRIPT_PREPARE_SERVICE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    EXTERNAL_SCRIPT_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    EXTERNAL_WAIT_ON_LAUNCHER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_HADR_TRANSPORT_CONNECTION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_REPLICA_CONTROLLER_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FAILPOINT,
    /// Occurs when the reads of a snapshot (or a temporary snapshot created by DBCC) sparse file are synchronized.
    FCB_REPLICA_READ,
    /// Occurs when the pushing or pulling of a page to a snapshot (or a temporary snapshot created by DBCC) sparse file is synchronized.
    FCB_REPLICA_WRITE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FEATURE_SWITCHES_UPDATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_DB_KILL_FLAG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_DB_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_FCB,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_FCB_FIND,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_FCB_PARENT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_FCB_RELEASE_CACHED_ENTRIES,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    FFT_NSO_FCB_STATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_FILEOBJECT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NSO_TABLE_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_NTFS_STORE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_RSFX_COMM,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_RSFX_WAIT_FOR_MEMORY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_STARTUP_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_STORE_DB,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_STORE_ROWSET_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FFT_STORE_TABLE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_CACHE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_CHUNKER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_CHUNKER_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_FCB,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_FILE_OBJECT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILESTREAM_WORKITEM_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FILETABLE_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    FILE_VALIDATION_THREADS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) through current.
    FOREIGN_REDO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    FORWARDER_TRANSITION,
    /// Occurs when a FILESTREAM file I/O operation is waiting for a FILESTREAM agent resource that is being used by another file I/O operation.
    FSAGENT,
    /// Occurs when a FILESTREAM file I/O operation needs to bind to the associated transaction, but the transaction is currently owned by another session.
    FSA_FORCE_OWN_XACT,
    /// Occurs when there's a wait for another FILESTREAM feature reconfiguration to be completed.
    FSTR_CONFIG_MUTEX,
    /// Occurs when there's a wait to serialize access to the FILESTREAM configuration parameters.
    FSTR_CONFIG_RWLOCK,
    /// Occurs when there's a wait by the FILESTREAM garbage collector to do either of the following tasks:
    ///
    /// - Disable garbage collection (used by backup and restore).
    /// - Execute one cycle of the FILESTREAM garbage collector.
    FS_FC_RWLOCK,
    /// Occurs when the FILESTREAM garbage collector is waiting for cleanup tasks to be completed.
    FS_GARBAGE_COLLECTOR_SHUTDOWN,
    /// Occurs when there's a wait to acquire access to the FILESTREAM header of a FILESTREAM data container to either read or update contents in the FILESTREAM header file (Filestream.hdr).
    FS_HEADER_RWLOCK,
    /// Occurs when there's a wait to acquire access to FILESTREAM log truncation to do either of the following tasks:
    ///
    /// - Temporarily disable FILESTREAM log (FSLOG) truncation (used by backup and restore).
    /// - Execute one cycle of FSLOG truncation.
    FS_LOGTRUNC_RWLOCK,
    /// Full-text is waiting on fragment metadata operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_COMPROWSET_RWLOCK,
    /// Full-text is waiting on an FDHost control operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_IFTSHC_MUTEX,
    /// Full-text is waiting on communication operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_IFTSISM_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_ASYNC_WRITE_PIPE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_BLOB_HASH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CATEALOG_SOURCE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_COMP_DESC_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CONSUMER_PLUGIN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CRAWL_BATCH_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_CRAWL_CHILDREN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_DOCID_INTERFACE_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_DOCID_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_FP_INFO_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_HOST_CONTROLLER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_MASTER_MERGE_TASK_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_MEMREGPOOL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_MERGE_FRAGMENT_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_NOISE_WORDS_COLLECTION_CACHE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_NOISE_WORDS_RESOURCE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_OCCURRENCE_BUFFER_POOL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_PIPELINE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_PIPELINE_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_PIPELINE_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_PROJECT_FD_INFO_MAP,
    /// Full-text is waiting on internal synchronization. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_IFTS_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_SCHEDULER,
    /// Full-text scheduler sleep wait type. The scheduler is idle.
    FT_IFTS_SCHEDULER_IDLE_WAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_SHARED_MEMORY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_SHUTDOWN_PIPE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_SRCH_FD_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_SRCH_FD_SERVICE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_STOPLIST_CACHE_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_THESAURUS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_VERSION_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2022 (16.x) CU 1 and later versions.
    FT_IFTS_WORK_QUEUE,
    /// Full-text is waiting on master merge operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_MASTER_MERGE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FT_MASTER_MERGE_COORDINATOR,
    /// Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    FT_METADATA_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    FT_PROPERTYLIST_CACHE,
    /// Occurs when a full-text crawl needs to restart from a last known good point to recover from a transient failure. The wait lets the worker tasks currently working on that population to complete or exit the current step.
    FT_RESTART_CRAWL,
    /// Occurs during synchronization of full-text operations.
    /// (Logged as "FULLTEXT GATHERER" - with a space.)
    FULLTEXT_GATHERER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    GDMA_GET_RESOURCE_OWNER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    GHOSTCLEANUPSYNCMGR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    GHOSTCLEANUP_UPDATE_STATS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    GLOBAL_QUERY_CANCEL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    GLOBAL_QUERY_CLOSE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    GLOBAL_QUERY_CONSUMER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    GLOBAL_QUERY_PRODUCER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    GLOBAL_TRAN_CREATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    GLOBAL_TRAN_UCS_SESSION,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    GUARDIAN,
    /// Occurs when an availability group DDL statement or Windows Server Failover Clustering command is waiting for exclusive read/write access to the configuration of an availability group.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_AG_MUTEX,
    /// The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the list of event subscribers. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST,
    /// Occurs when an availability group DDL statement or Windows Server Failover Clustering command is waiting for exclusive read/write access to the runtime state of the local replica of the associated availability group.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_AR_CRITICAL_SECTION_ENTRY,
    /// Occurs when an availability replica shutdown is waiting for startup to complete or an availability replica startup is waiting for shutdown to complete. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_AR_MANAGER_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_AR_UNLOAD_COMPLETED,
    /// The availability group primary database received a backup request from a secondary database and is waiting for the background thread to finish processing the request on acquiring or releasing the BulkOp lock.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_BACKUP_BULK_LOCK,
    /// The backup background thread of the availability group primary database is waiting for a new work request from the secondary database. (Typically, this occurs when the primary database is holding the BulkOp log and is waiting for the secondary database to indicate that the primary database can release the lock).
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_BACKUP_QUEUE,
    /// A SQL Server thread is waiting to switch from non-preemptive mode (scheduled by SQL Server) to preemptive mode (scheduled by the operating system) in order to invoke Windows Server Failover Clustering APIs.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_CLUSAPI_CALL,
    /// Waiting for access to the cache of compressed log blocks that is used to avoid redundant compression of the log blocks sent to multiple secondary databases.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_COMPRESSED_CACHE_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_CONNECTIVITY_INFO,
    /// Waiting for messages to be sent to the partner when the maximum number of queued messages has been reached. Indicates that the log scans are running faster than the network sends. This is an issue only if network sends are slower than expected.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DATABASE_FLOW_CONTROL,
    /// Occurs on the versioning state change of an availability group secondary database. This wait is for internal data structures and usually is very short with no direct effect on data access.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DATABASE_VERSIONING_STATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_DATABASE_WAIT_FOR_RECOVERY,
    /// Waiting for the database to restart under availability group control. Under normal conditions, this isn't a customer issue because waits are expected here.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DATABASE_WAIT_FOR_RESTART,
    /// A query on objects in a readable secondary database of an availability group is blocked on row versioning while waiting for commit or rollback of all transactions that were in-flight when the secondary replica was enabled for read workloads. This wait type guarantees that row versions are available before execution of a query under snapshot isolation.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING,
    /// The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the runtime state of an event subscriber that corresponds to an availability database. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DBR_SUBSCRIBER,
    /// The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the list of event subscribers that correspond to availability databases. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DBR_SUBSCRIBER_FILTER_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HADR_DBSEEDING,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HADR_DBSEEDING_LIST,
    /// Concurrency control wait for updating the internal state of the database replica.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DBSTATECHANGE_SYNC,
    /// Waiting for responses to conversational messages (which require an explicit response from the other side, using the availability group conversational message infrastructure). Many different message types use this wait type.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DB_COMMAND,
    /// Waiting for responses to conversational messages (which require an explicit response from the other side, using the availability group conversational message infrastructure). Many different message types use this wait type.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DB_OP_COMPLETION_SYNC,
    /// An availability group DDL statement or a Windows Server Failover Clustering command is waiting for serialized access to an availability database and its runtime state.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_DB_OP_START_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HADR_FABRIC_CALLBACK,
    /// The FILESTREAM Always On transport manager is waiting until processing of a log block is finished.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_BLOCK_FLUSH,
    /// The FILESTREAM Always On transport manager is waiting until the next FILESTREAM file gets processed and its handle gets closed.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_FILE_CLOSE,
    /// An Always On secondary replica is waiting for the primary replica to send all requested FILESTREAM files during UNDO.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_FILE_REQUEST,
    /// The FILESTREAM Always On transport manager is waiting for R/W lock that protects the FILESTREAM Always On I/O manager during startup or shutdown.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_IOMGR,
    /// The FILESTREAM Always On I/O manager is waiting for I/O completion.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_IOMGR_IOCOMPLETION,
    /// The FILESTREAM Always On transport manager is waiting for the R/W lock that protects the FILESTREAM Always On transport manager during startup or shutdown.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_FILESTREAM_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_FILESTREAM_PREPROC,
    /// Transaction commit processing is waiting to allow a group commit so that multiple commit log records can be put into a single log block. This wait is an expected condition that optimizes the log I/O, capture, and send operations.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_GROUP_COMMIT,
    /// Concurrency control around the log capture or apply object when creating or destroying scans. This is an expected wait when partners change state or connection status.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_LOGCAPTURE_SYNC,
    /// Waiting for log records to become available. Can occur either when waiting for new log records to be generated by connections or for I/O completion when reading log not in the cache. This is an expected wait if the log scan is caught up to the end of log or is reading from disk.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_LOGCAPTURE_WAIT,
    /// Concurrency control wait when updating the log progress status of database replicas.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_LOGPROGRESS_SYNC,
    /// A background task that processes Windows Server Failover Clustering notifications is waiting for the next notification. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_NOTIFICATION_DEQUEUE,
    /// The availability replica manager is waiting for serialized access to the runtime state of a background task that processes Windows Server Failover Clustering notifications. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS,
    /// A background task is waiting for the completion of the startup of a background task that processes Windows Server Failover Clustering notifications. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_NOTIFICATION_WORKER_STARTUP_SYNC,
    /// A background task is waiting for the termination of a background task that processes Windows Server Failover Clustering notifications. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_NOTIFICATION_WORKER_TERMINATION_SYNC,
    /// Concurrency control wait on the partner list.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_PARTNER_SYNC,
    /// Waiting to get read or write access to the list of WSFC networks. Internal use only. Note: The engine keeps a list of WSFC networks that is used in DMVs (such as sys.dm_hadr_cluster_networks) or to validate Always On Transact-SQL statements that reference WSFC network information. This list is updated upon engine startup, WSFC related notifications, and internal Always On restart (for example, losing and regaining of WSFC quorum). Tasks are usually blocked when an update in that list is in progress.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_READ_ALL_NETWORKS,
    /// Waiting for the secondary database to connect to the primary database before running recovery. This is an expected wait, which can lengthen if the connection to the primary is slow to establish.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_RECOVERY_WAIT_FOR_CONNECTION,
    /// Database recovery is waiting for the secondary database to finish the reverting and initializing phase to bring it back to the common log point with the primary database. This is an expected wait after failovers. Undo progress can be tracked through the Windows System Monitor (perfmon.exe) and DMVs.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_RECOVERY_WAIT_FOR_UNDO,
    /// Waiting for concurrency control to update the current replica state.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_REPLICAINFO_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_CANCELLATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_FILE_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_LIMIT_BACKUPS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_SYNC_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_TIMEOUT_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_SEEDING_WAIT_FOR_COMPLETION,
    /// Waiting for transaction commit processing to allow a synchronizing secondary database to catch up to the primary end of the log, in order to transition to the synchronized state. This is an expected wait when a secondary database is catching up.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_SYNCHRONIZING_THROTTLE,
    /// Waiting for a transaction commit processing on the synchronized secondary databases to harden the log. This wait is also reflected by the Transaction Delay performance counter. This wait type is expected for synchronous-commit availability groups, and indicates the time to send, write, and acknowledge log commit to the secondary databases.
    /// For detailed information and troubleshooting HADR_SYNC_COMMIT, refer to this blog post
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_SYNC_COMMIT,
    /// Either the internal Always On system, or the WSFC cluster, requests that listeners are started or stopped. The processing of this request is always asynchronous, and there's a mechanism to remove redundant requests. There are also moments that this process is suspended because of configuration changes. All waits related with this listener synchronization mechanism use this wait type. Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TDS_LISTENER_SYNC,
    /// Used at the end of an Always On Transact-SQL statement that requires starting and/or stopping an availability group listener. Since the start/stop operation is done asynchronously, the user thread blocks using this wait type until the situation of the listener is known.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TDS_LISTENER_SYNC_PROCESSING,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HADR_THROTTLE_LOG_RATE_GOVERNOR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    HADR_THROTTLE_LOG_RATE_LOG_SIZE,
    /// Occurs when a geo-replication secondary is configured with lower compute size (lower SLO) than the primary. A primary database is throttled due to delayed log consumption by the secondary. This is caused by the secondary database having insufficient compute capacity to keep up with the primary database's rate of change.
    ///
    /// Applies to: Azure SQL Database
    HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    HADR_THROTTLE_LOG_RATE_SEEDING,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE,
    /// Waiting to get the lock on the timer task object and is also used for the actual waits between times that work is being performed. For example, for a task that runs every 10 seconds, after one execution, availability groups waits about 10 seconds to reschedule the task, and the wait is included here.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TIMER_TASK,
    /// Waiting for access to the transport layer's database replica list. Used for the spinlock that grants access to it.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TRANSPORT_DBRLIST,
    /// Waiting when the number of outstanding unacknowledged Always On messages is over the out flow control threshold. This is on an availability replica-to-replica basis (not on a database-to-database basis).
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TRANSPORT_FLOW_CONTROL,
    /// Availability groups are waiting while changing or accessing the underlying transport state.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_TRANSPORT_SESSION,
    /// Concurrency control wait on the availability group background work task object.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_WORK_POOL,
    /// Availability group background worker thread waiting for new work to be assigned. This is an expected wait when there are ready workers waiting for new work, which is the normal state.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_WORK_QUEUE,
    /// Accessing (look up, add, and delete) the extended recovery fork stack for an availability database.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HADR_XRF_STACK_ACCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HCCO_CACHE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HKCS_PARALLEL_MIGRATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HKCS_PARALLEL_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    HK_RESTORE_FILEMAP,
    /// Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions, but not Azure SQL Database, Azure SQL Managed Instance with the always-up-to-date update policy, and Azure Synapse Analytics.
    HTBUILD,
    /// Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.
    HTBUILD_AGG,
    /// Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.
    HTBUILD_JOIN,
    /// Occurs with parallel batch-mode plans when synchronizing at the end of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions, but not Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.
    HTDELETE,
    /// Occurs with parallel batch-mode plans when synchronizing at the end of a hash aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.
    HTDELETE_AGG,
    /// Occurs with parallel batch-mode plans when synchronizing at the end of a hash join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.
    HTDELETE_JOIN,
    /// Occurs with parallel batch-mode plans when synchronizing before scanning hash table to output matches / non-matches in hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HTMEMO,
    /// Occurs with parallel batch-mode plans when synchronizing before resetting a hash join/aggregation for the next partial join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HTREINIT,
    /// Occurs with parallel batch-mode plans when synchronizing the repartitioning of the hash table on the input side of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    HTREPARTITION,
    /// Occurs at startup to enumerate the HTTP endpoints to start HTTP.
    HTTP_ENUMERATION,
    /// Occurs when a connection is waiting for HTTP to complete initialization.
    HTTP_START,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    HTTP_STORAGE_CONNECTION,
    /// Occurs when SQL Server waits for a bulkload I/O to finish.
    IMPPROV_IOWAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    INSTANCE_LOG_RATE_GOVERNOR,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    INTERNAL_TESTING,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    IOAFF_RANGE_QUEUE,
    /// Occurs during synchronization of trace event buffers.
    IO_AUDIT_MUTEX,
    /// Occurs while waiting for I/O operations to complete. This wait type generally represents non-data page I/Os. Data page I/O completion waits appear as PAGEIOLATCH_* waits.
    IO_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    IO_QUEUE_LIMIT,
    /// Occurs when an I/O operation such as a read or a write to disk fails because of insufficient resources, and is then retried.
    IO_RETRY,
    /// Used by the service control task while waiting for requests from the Service Control Manager. Long waits are expected and don't indicate a problem.
    KSOURCE_WAKEUP,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    KTM_ENLISTMENT,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    KTM_RECOVERY_MANAGER,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    KTM_RECOVERY_RESOLUTION,
    /// Occurs when waiting for a DT (destroy) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.
    LATCH_DT,
    /// Occurs when waiting for an EX (exclusive) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.
    LATCH_EX,
    /// Occurs when waiting for a KP (keep) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.
    LATCH_KP,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    LATCH_NL,
    /// Occurs when waiting for an SH (share) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.
    LATCH_SH,
    /// Occurs when waiting for an UP (update) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.
    LATCH_UP,
    /// Occurs when lazy writer tasks are suspended. This is a measure of the time spent by background tasks that are waiting. Don't consider this state when you're looking for user stalls.
    LAZYWRITER_SLEEP,
    /// Occurs when a task is waiting to acquire a Bulk Update (BU) lock. For more information, see Bulk Update Locks.
    LCK_M_BU,
    /// Occurs when a task is waiting to acquire a Bulk Update (BU) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Bulk Update Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_BU_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Bulk Update (BU) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Bulk Update Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_BU_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Intent Shared (IS) lock. For more information, see Intent Locks.
    LCK_M_IS,
    /// Occurs when a task is waiting to acquire an Intent Shared (IS) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IS_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Intent Shared (IS) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IS_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Intent Update (IU) lock. For more information, see Intent Locks.
    LCK_M_IU,
    /// Occurs when a task is waiting to acquire an Intent Update (IU) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IU_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Intent Update (IU) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IU_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock. For more information, see Intent Locks.
    LCK_M_IX,
    /// Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IX_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_IX_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a NULL lock on the current key value, and an Insert Range lock between the current and previous key. A NULL lock on the key is an instant release lock.
    LCK_M_RIn_NL,
    /// Occurs when a task is waiting to acquire a NULL lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. A NULL lock on the key is an instant release lock. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_NL_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a NULL lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. A NULL lock on the key is an instant release lock. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_NL_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a shared lock on the current key value, and an Insert Range lock between the current and previous key.
    LCK_M_RIn_S,
    /// Occurs when a task is waiting to acquire a shared lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_S_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a shared lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_S_LOW_PRIORITY,
    /// Task is waiting to acquire an Update lock on the current key value, and an Insert Range lock between the current and previous key.
    LCK_M_RIn_U,
    /// Task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_U_ABORT_BLOCKERS,
    /// Task is waiting to acquire an Update lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_U_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Exclusive lock on the current key value, and an Insert Range lock between the current and previous key.
    LCK_M_RIn_X,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_X_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RIn_X_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared lock on the current key value, and a Shared Range lock between the current and previous key.
    LCK_M_RS_S,
    /// Occurs when a task is waiting to acquire a Shared lock with Abort Blockers on the current key value, and a Shared Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RS_S_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Shared lock with Low Priority on the current key value, and a Shared Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RS_S_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Update lock on the current key value, and an Update Range lock between the current and previous key.
    LCK_M_RS_U,
    /// Occurs when a task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Update Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RS_U_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Update lock with Low Priority on the current key value, and an Update Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RS_U_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared lock on the current key value, and an Exclusive Range lock between the current and previous key.
    LCK_M_RX_S,
    /// Occurs when a task is waiting to acquire a Shared lock with Abort Blockers on the current key value, and an Exclusive Range with Abort Blockers lock between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_S_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Shared lock with Low Priority on the current key value, and an Exclusive Range with Low Priority lock between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_S_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Update lock on the current key value, and an Exclusive range lock between the current and previous key.
    LCK_M_RX_U,
    /// Occurs when a task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Exclusive range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_U_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Update lock with Low Priority on the current key value, and an Exclusive range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_U_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Exclusive lock on the current key value, and an Exclusive Range lock between the current and previous key.
    LCK_M_RX_X,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers on the current key value, and an Exclusive Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_X_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Low Priority on the current key value, and an Exclusive Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_RX_X_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared lock. For more information, see Shared Locks.
    LCK_M_S,
    /// Occurs when a task is waiting to acquire a Schema Modify lock. For more information, see Schema Locks.
    LCK_M_SCH_M,
    /// Occurs when a task is waiting to acquire a Schema Modify lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SCH_M_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Schema Modify lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SCH_M_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Schema Share lock. For more information, see Schema Locks.
    LCK_M_SCH_S,
    /// Occurs when a task is waiting to acquire a Schema Share lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SCH_S_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Schema Share lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SCH_S_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared With Intent Update lock. For more information, see Intent Locks.
    LCK_M_SIU,
    /// Occurs when a task is waiting to acquire a Shared With Intent Update lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SIU_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Shared With Intent Update lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SIU_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock. For more information, see Intent Locks.
    LCK_M_SIX,
    /// Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SIX_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_SIX_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire a Shared lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Shared Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_S_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire a Shared lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Shared Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_S_LOW_PRIORITY,
    /// Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction) wait_resource type, where the read or modify intent can't be inferred.
    LCK_M_S_XACT,
    /// Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction) wait_resource type, with an intent to modify.
    LCK_M_S_XACT_MODIFY,
    /// Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction)wait_resource type, with an intent to read.
    LCK_M_S_XACT_READ,
    /// Occurs when a task is waiting to acquire an Update lock. For more information, see Update Locks.
    LCK_M_U,
    /// Occurs when a task is waiting to acquire an Update With Intent Exclusive lock. For more information, see Intent Locks.
    LCK_M_UIX,
    /// Occurs when a task is waiting to acquire an Update With Intent Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_UIX_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Update With Intent Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_UIX_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Update lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Update Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_U_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Update lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Update Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_U_LOW_PRIORITY,
    /// Occurs when a task is waiting to acquire an Exclusive lock. For more information, see Exclusive Locks.
    LCK_M_X,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Exclusive Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_X_ABORT_BLOCKERS,
    /// Occurs when a task is waiting to acquire an Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Exclusive Locks.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    LCK_M_X_LOW_PRIORITY,
    /// Occurs when a task is waiting for space in the log buffer to store a log record. Consistently high values might indicate that the log devices can't keep up with the amount of log being generated by the server.
    LOGBUFFER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGCAPTURE_LOGPOOLTRUNCPOINT,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    LOGGENERATION,
    /// Occurs when a task is waiting for any outstanding log I/Os to finish before shutting down the log while closing the database.
    LOGMGR,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    LOGMGR_FLUSH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    LOGMGR_PMM_LOG,
    /// Occurs while the log writer task waits for work requests.
    LOGMGR_QUEUE,
    /// Occurs when a task is waiting to see whether log truncation frees up log space to enable the task to write a new log record. Consider increasing the size of the log files for the affected database to reduce this wait.
    LOGMGR_RESERVE_APPEND,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOLREFCOUNTEDOBJECT_REFDONE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_CACHESIZE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_CONSUMER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_CONSUMERSET,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_FREEPOOLS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_MGRSET,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    LOGPOOL_REPLACEMENTSET,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    LOG_POOL_SCAN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    LOG_RATE_GOVERNOR,
    /// Occurs while waiting for memory to be available for use.
    LOWFAIL_MEMMGR_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    MD_AGENT_YIELD,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    MD_LAZYCACHE_RWLOCK,
    /// Occurs while allocating memory from either the internal SQL Server memory pool or the operation system.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    MEMORY_ALLOCATION_EXT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    MEMORY_GRANT_UPDATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    METADATA_LAZYCACHE_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    MIGRATIONBUFFER,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    MISCELLANEOUS,
    /// Occurs when a task is waiting for a distributed query operation to finish. This is used to detect potential Multiple Active Result Set (MARS) application deadlocks. The wait ends when the distributed query call finishes.
    MSQL_DQ,
    /// Occurs when a task is waiting to obtain ownership of the session transaction manager to perform a session level transaction operation.
    MSQL_XACT_MGR_MUTEX,
    /// Occurs during synchronization of transaction usage. A request must acquire the mutex before it can use the transaction.
    MSQL_XACT_MUTEX,
    /// Occurs when a task is waiting for an extended stored procedure to end. SQL Server uses this wait state to detect potential MARS application deadlocks. The wait stops when the extended stored procedure call ends.
    MSQL_XP,
    /// Occurs during Full-Text Search calls. This wait ends when the full-text operation completes. It doesn't indicate contention, but rather the duration of full-text operations.
    MSSEARCH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    NETWORKSXMLMGRLOAD,
    /// Occurs when a connection is waiting for a network packet during a network read.
    NET_WAITFOR_PACKET,
    /// Internal use only.
    NODE_CACHE_MUTEX,
    /// Occurs when SQL Server calls the SNAC OLE DB Provider (SQLNCLI) or the Microsoft OLE DB Driver for SQL Server (MSOLEDBSQL). This wait type isn't used for synchronization. Instead, it indicates the duration of calls to the OLE DB provider.
    OLEDB,
    /// Occurs while a background task waits for high priority system task requests. Long wait times indicate that there have been no high priority requests to process, and shouldn't cause concern.
    ONDEMAND_TASK_QUEUE,
    /// Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Destroy mode. Long waits might indicate problems with the disk subsystem.
    PAGEIOLATCH_DT,
    /// Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Exclusive mode - a mode used when the buffer is being written to disk. Long waits might indicate problems with the disk subsystem.
    ///
    /// For more information, see Slow I/O - SQL Server and disk I/O performance.
    PAGEIOLATCH_EX,
    /// Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Keep mode. Long waits might indicate problems with the disk subsystem.
    PAGEIOLATCH_KP,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PAGEIOLATCH_NL,
    /// Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Shared mode - a mode used when the buffer is being read from disk. Long waits might indicate problems with the disk subsystem.
    ///
    /// For more information, see Slow I/O - SQL Server and disk I/O performance.
    PAGEIOLATCH_SH,
    /// Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Update mode. Long waits might indicate problems with the disk subsystem.
    ///
    /// For more information, see Slow I/O - SQL Server and disk I/O performance.
    PAGEIOLATCH_UP,
    /// Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Destroy mode. Destroy mode must be acquired before deleting contents of a page. For more information, see Latch Modes.
    PAGELATCH_DT,
    /// Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Exclusive mode - it blocks other threads from writing to or reading from the page (buffer).
    ///
    /// A common scenario that leads to this latch is the "last-page insert" buffer latch contention. To understand and resolve this, use Resolve last-page insert PAGELATCH_EX contention and Diagnose and resolve last-page-insert latch contention on SQL Server. Another scenario is Latch contention on small tables with a non-clustered index and random inserts (queue table).
    PAGELATCH_EX,
    /// Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Keep mode, which prevents the page from being destroyed by another thread. For more information, see Latch Modes.
    PAGELATCH_KP,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PAGELATCH_NL,
    /// Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Shared mode, which allows multiple threads to read, but not modify, a buffer (page). For more information, see Latch Modes.
    PAGELATCH_SH,
    /// Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Update mode. Commonly this wait type might be observed when a system page (buffer) like PFS, GAM, SGAM is latched. For more information, see Latch Modes.
    ///
    /// For troubleshooting a common scenario with this latch, refer to Reduce Allocation Contention in SQL Server tempdb database.
    PAGELATCH_UP,
    /// Occurs when serializing output produced by RESTORE HEADERONLY, RESTORE FILELISTONLY, or RESTORE LABELONLY.
    PARALLEL_BACKUP_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_DRAIN_WORKER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_FLOW_CONTROL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_LOG_CACHE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_TRAN_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_TRAN_TURN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_WORKER_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PARALLEL_REDO_WORKER_WAIT_WORK,
    /// Internal use only.
    PERFORMANCE_COUNTERS_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PHYSICAL_SEEDING_DMV,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    POOL_LOG_RATE_GOVERNOR,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PREEMPTIVE_ABR,
    /// Occurs when the SQL Server Operating System (SQLOS) scheduler switches to preemptive mode to write an audit event to the Windows event log.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    PREEMPTIVE_AUDIT_ACCESS_EVENTLOG,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to write an audit event to the Windows Security log.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    PREEMPTIVE_AUDIT_ACCESS_SECLOG,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to close backup media.
    PREEMPTIVE_CLOSEBACKUPMEDIA,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to close a tape backup device.
    PREEMPTIVE_CLOSEBACKUPTAPE,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to close a virtual backup device.
    PREEMPTIVE_CLOSEBACKUPVDIDEVICE,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to perform Windows Server failover cluster operations.
    PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL,
    /// Occurs when the SQLOS scheduler switches to preemptive mode to create a COM object.
    PREEMPTIVE_COM_COCREATEINSTANCE,
    /// Internal use only.
    PREEMPTIVE_COM_COGETCLASSOBJECT,
    /// Internal use only.
    PREEMPTIVE_COM_CREATEACCESSOR,
    /// Internal use only.
    PREEMPTIVE_COM_DELETEROWS,
    /// Internal use only.
    PREEMPTIVE_COM_GETCOMMANDTEXT,
    /// Internal use only.
    PREEMPTIVE_COM_GETDATA,
    /// Internal use only.
    PREEMPTIVE_COM_GETNEXTROWS,
    /// Internal use only.
    PREEMPTIVE_COM_GETRESULT,
    /// Internal use only.
    PREEMPTIVE_COM_GETROWSBYBOOKMARK,
    /// Internal use only.
    PREEMPTIVE_COM_LBFLUSH,
    /// Internal use only.
    PREEMPTIVE_COM_LBLOCKREGION,
    /// Internal use only.
    PREEMPTIVE_COM_LBREADAT,
    /// Internal use only.
    PREEMPTIVE_COM_LBSETSIZE,
    /// Internal use only.
    PREEMPTIVE_COM_LBSTAT,
    /// Internal use only.
    PREEMPTIVE_COM_LBUNLOCKREGION,
    /// Internal use only.
    PREEMPTIVE_COM_LBWRITEAT,
    /// Internal use only.
    PREEMPTIVE_COM_QUERYINTERFACE,
    /// Internal use only.
    PREEMPTIVE_COM_RELEASE,
    /// Internal use only.
    PREEMPTIVE_COM_RELEASEACCESSOR,
    /// Internal use only.
    PREEMPTIVE_COM_RELEASEROWS,
    /// Internal use only.
    PREEMPTIVE_COM_RELEASESESSION,
    /// Internal use only.
    PREEMPTIVE_COM_RESTARTPOSITION,
    /// Internal use only.
    PREEMPTIVE_COM_SEQSTRMREAD,
    /// Internal use only.
    PREEMPTIVE_COM_SEQSTRMREADANDWRITE,
    /// Internal use only.
    PREEMPTIVE_COM_SETDATAFAILURE,
    /// Internal use only.
    PREEMPTIVE_COM_SETPARAMETERINFO,
    /// Internal use only.
    PREEMPTIVE_COM_SETPARAMETERPROPERTIES,
    /// Internal use only.
    PREEMPTIVE_COM_STRMLOCKREGION,
    /// Internal use only.
    PREEMPTIVE_COM_STRMSEEKANDREAD,
    /// Internal use only.
    PREEMPTIVE_COM_STRMSEEKANDWRITE,
    /// Internal use only.
    PREEMPTIVE_COM_STRMSETSIZE,
    /// Internal use only.
    PREEMPTIVE_COM_STRMSTAT,
    /// Internal use only.
    PREEMPTIVE_COM_STRMUNLOCKREGION,
    /// Internal use only.
    PREEMPTIVE_CONSOLEWRITE,
    /// Internal use only.
    PREEMPTIVE_CREATEPARAM,
    /// Internal use only.
    PREEMPTIVE_DEBUG,
    /// Internal use only.
    PREEMPTIVE_DFSADDLINK,
    /// Internal use only.
    PREEMPTIVE_DFSLINKEXISTCHECK,
    /// Internal use only.
    PREEMPTIVE_DFSLINKHEALTHCHECK,
    /// Internal use only.
    PREEMPTIVE_DFSREMOVELINK,
    /// Internal use only.
    PREEMPTIVE_DFSREMOVEROOT,
    /// Internal use only.
    PREEMPTIVE_DFSROOTFOLDERCHECK,
    /// Internal use only.
    PREEMPTIVE_DFSROOTINIT,
    /// Internal use only.
    PREEMPTIVE_DFSROOTSHARECHECK,
    /// Internal use only.
    PREEMPTIVE_DTC_ABORT,
    /// Internal use only.
    PREEMPTIVE_DTC_ABORTREQUESTDONE,
    /// Internal use only.
    PREEMPTIVE_DTC_BEGINTRANSACTION,
    /// Internal use only.
    PREEMPTIVE_DTC_COMMITREQUESTDONE,
    /// Internal use only.
    PREEMPTIVE_DTC_ENLIST,
    /// Internal use only.
    PREEMPTIVE_DTC_PREPAREREQUESTDONE,
    /// Internal use only.
    PREEMPTIVE_FILESIZEGET,
    /// Internal use only.
    PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION,
    /// Internal use only.
    PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION,
    /// Internal use only.
    PREEMPTIVE_FSAOLEDB_STARTTRANSACTION,
    /// Internal use only.
    PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO,
    /// Internal use only.
    PREEMPTIVE_GETRMINFO,
    /// Availability group lease manager scheduling for Microsoft Support diagnostics.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PREEMPTIVE_HADR_LEASE_MECHANISM,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PREEMPTIVE_HTTP_EVENT_WAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PREEMPTIVE_HTTP_REQUEST,
    /// Internal use only.
    PREEMPTIVE_LOCKMONITOR,
    /// Internal use only.
    PREEMPTIVE_MSS_RELEASE,
    /// Internal use only.
    PREEMPTIVE_ODBCOPS,
    /// Internal use only.
    PREEMPTIVE_OLEDBOPS,
    /// Internal use only.
    PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN,
    /// Internal use only.
    PREEMPTIVE_OLEDB_ABORTTRAN,
    /// Internal use only.
    PREEMPTIVE_OLEDB_GETDATASOURCE,
    /// Internal use only.
    PREEMPTIVE_OLEDB_GETLITERALINFO,
    /// Internal use only.
    PREEMPTIVE_OLEDB_GETPROPERTIES,
    /// Internal use only.
    PREEMPTIVE_OLEDB_GETPROPERTYINFO,
    /// Internal use only.
    PREEMPTIVE_OLEDB_GETSCHEMALOCK,
    /// Internal use only.
    PREEMPTIVE_OLEDB_JOINTRANSACTION,
    /// Internal use only.
    PREEMPTIVE_OLEDB_RELEASE,
    /// Internal use only.
    PREEMPTIVE_OLEDB_SETPROPERTIES,
    /// Internal use only.
    PREEMPTIVE_OLE_UNINIT,
    /// Internal use only.
    PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT,
    /// Internal use only.
    PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE,
    /// Internal use only.
    PREEMPTIVE_OS_AUTHENTICATIONOPS,
    /// Internal use only.
    PREEMPTIVE_OS_AUTHORIZATIONOPS,
    /// Internal use only.
    PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT,
    /// Internal use only.
    PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID,
    /// Internal use only.
    PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER,
    /// Internal use only.
    PREEMPTIVE_OS_BACKUPREAD,
    /// Internal use only.
    PREEMPTIVE_OS_CLOSEHANDLE,
    /// Internal use only.
    PREEMPTIVE_OS_CLUSTEROPS,
    /// Internal use only.
    PREEMPTIVE_OS_COMOPS,
    /// Internal use only.
    PREEMPTIVE_OS_COMPLETEAUTHTOKEN,
    /// Internal use only.
    PREEMPTIVE_OS_COPYFILE,
    /// Internal use only.
    PREEMPTIVE_OS_CREATEDIRECTORY,
    /// Internal use only.
    PREEMPTIVE_OS_CREATEFILE,
    /// Internal use only.
    PREEMPTIVE_OS_CRYPTACQUIRECONTEXT,
    /// Internal use only.
    PREEMPTIVE_OS_CRYPTIMPORTKEY,
    /// Internal use only.
    PREEMPTIVE_OS_CRYPTOPS,
    /// Internal use only.
    PREEMPTIVE_OS_DECRYPTMESSAGE,
    /// Internal use only.
    PREEMPTIVE_OS_DELETEFILE,
    /// Internal use only.
    PREEMPTIVE_OS_DELETESECURITYCONTEXT,
    /// Internal use only.
    PREEMPTIVE_OS_DEVICEIOCONTROL,
    /// Internal use only.
    PREEMPTIVE_OS_DEVICEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_DIRSVC_NETWORKOPS,
    /// Internal use only.
    PREEMPTIVE_OS_DISCONNECTNAMEDPIPE,
    /// Internal use only.
    PREEMPTIVE_OS_DOMAINSERVICESOPS,
    /// Internal use only.
    PREEMPTIVE_OS_DSGETDCNAME,
    /// Internal use only.
    PREEMPTIVE_OS_DTCOPS,
    /// Internal use only.
    PREEMPTIVE_OS_ENCRYPTMESSAGE,
    /// Internal use only.
    PREEMPTIVE_OS_FILEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_FINDFILE,
    /// Internal use only.
    PREEMPTIVE_OS_FLUSHFILEBUFFERS,
    /// Internal use only.
    PREEMPTIVE_OS_FORMATMESSAGE,
    /// Internal use only.
    PREEMPTIVE_OS_FREECREDENTIALSHANDLE,
    /// Internal use only.
    PREEMPTIVE_OS_FREELIBRARY,
    /// Internal use only.
    PREEMPTIVE_OS_GENERICOPS,
    /// Internal use only.
    PREEMPTIVE_OS_GETADDRINFO,
    /// Internal use only.
    PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE,
    /// Internal use only.
    PREEMPTIVE_OS_GETDISKFREESPACE,
    /// Internal use only.
    PREEMPTIVE_OS_GETFILEATTRIBUTES,
    /// Internal use only.
    PREEMPTIVE_OS_GETFILESIZE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE,
    /// Internal use only.
    PREEMPTIVE_OS_GETLONGPATHNAME,
    /// Internal use only.
    PREEMPTIVE_OS_GETPROCADDRESS,
    /// Internal use only.
    PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT,
    /// Internal use only.
    PREEMPTIVE_OS_GETVOLUMEPATHNAME,
    /// Internal use only.
    PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT,
    /// Internal use only.
    PREEMPTIVE_OS_LIBRARYOPS,
    /// Internal use only.
    PREEMPTIVE_OS_LOADLIBRARY,
    /// Internal use only.
    PREEMPTIVE_OS_LOGONUSER,
    /// Internal use only.
    PREEMPTIVE_OS_LOOKUPACCOUNTSID,
    /// Internal use only.
    PREEMPTIVE_OS_MESSAGEQUEUEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_MOVEFILE,
    /// Internal use only.
    PREEMPTIVE_OS_NETGROUPGETUSERS,
    /// Internal use only.
    PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS,
    /// Internal use only.
    PREEMPTIVE_OS_NETUSERGETGROUPS,
    /// Internal use only.
    PREEMPTIVE_OS_NETUSERGETLOCALGROUPS,
    /// Internal use only.
    PREEMPTIVE_OS_NETUSERMODALSGET,
    /// Internal use only.
    PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY,
    /// Internal use only.
    PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE,
    /// Internal use only.
    PREEMPTIVE_OS_OPENDIRECTORY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PREEMPTIVE_OS_PDH_WMI_INIT,
    /// Internal use only.
    PREEMPTIVE_OS_PIPEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_PROCESSOPS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES,
    /// Internal use only.
    PREEMPTIVE_OS_QUERYREGISTRY,
    /// Internal use only.
    PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN,
    /// Internal use only.
    PREEMPTIVE_OS_REMOVEDIRECTORY,
    /// Internal use only.
    PREEMPTIVE_OS_REPORTEVENT,
    /// Internal use only.
    PREEMPTIVE_OS_REVERTTOSELF,
    /// Internal use only.
    PREEMPTIVE_OS_RSFXDEVICEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_SECURITYOPS,
    /// Internal use only.
    PREEMPTIVE_OS_SERVICEOPS,
    /// Internal use only.
    PREEMPTIVE_OS_SETENDOFFILE,
    /// Internal use only.
    PREEMPTIVE_OS_SETFILEPOINTER,
    /// Internal use only.
    PREEMPTIVE_OS_SETFILEVALIDDATA,
    /// Internal use only.
    PREEMPTIVE_OS_SETNAMEDSECURITYINFO,
    /// Internal use only.
    PREEMPTIVE_OS_SQLCLROPS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) through SQL Server 2016 (13.x).
    PREEMPTIVE_OS_SQMLAUNCH,
    /// Internal use only.
    PREEMPTIVE_OS_VERIFYSIGNATURE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PREEMPTIVE_OS_VERIFYTRUST,
    /// Internal use only.
    PREEMPTIVE_OS_VSSOPS,
    /// Internal use only.
    PREEMPTIVE_OS_WAITFORSINGLEOBJECT,
    /// Internal use only.
    PREEMPTIVE_OS_WINSOCKOPS,
    /// Internal use only.
    PREEMPTIVE_OS_WRITEFILE,
    /// Internal use only.
    PREEMPTIVE_OS_WRITEFILEGATHER,
    /// Internal use only.
    PREEMPTIVE_OS_WSASETLASTERROR,
    /// Internal use only.
    PREEMPTIVE_REENLIST,
    /// Internal use only.
    PREEMPTIVE_RESIZELOG,
    /// Internal use only.
    PREEMPTIVE_ROLLFORWARDREDO,
    /// Internal use only.
    PREEMPTIVE_ROLLFORWARDUNDO,
    /// Internal use only.
    PREEMPTIVE_SB_STOPENDPOINT,
    /// Internal use only.
    PREEMPTIVE_SERVER_STARTUP,
    /// Internal use only.
    PREEMPTIVE_SETRMINFO,
    /// Internal use only.
    PREEMPTIVE_SHAREDMEM_GETDATA,
    /// Internal use only.
    PREEMPTIVE_SNIOPEN,
    /// Internal use only.
    PREEMPTIVE_SOSHOST,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PREEMPTIVE_SOSTESTING,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PREEMPTIVE_SP_SERVER_DIAGNOSTICS,
    /// Internal use only.
    PREEMPTIVE_STARTRM,
    /// Internal use only.
    PREEMPTIVE_STREAMFCB_CHECKPOINT,
    /// Internal use only.
    PREEMPTIVE_STREAMFCB_RECOVER,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PREEMPTIVE_STRESSDRIVER,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PREEMPTIVE_TESTING,
    /// Internal use only.
    PREEMPTIVE_TRANSIMPORT,
    /// Internal use only.
    PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN,
    /// Internal use only.
    PREEMPTIVE_VSS_CREATESNAPSHOT,
    /// Internal use only.
    PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    PREEMPTIVE_XETESTING,
    /// Internal use only.
    PREEMPTIVE_XE_CALLBACKEXECUTE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PREEMPTIVE_XE_CX_FILE_OPEN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PREEMPTIVE_XE_CX_HTTP_CALL,
    /// Internal use only.
    PREEMPTIVE_XE_DISPATCHER,
    /// Internal use only.
    PREEMPTIVE_XE_ENGINEINIT,
    /// Internal use only.
    PREEMPTIVE_XE_GETTARGETSTATE,
    /// Internal use only.
    PREEMPTIVE_XE_SESSIONCOMMIT,
    /// Internal use only.
    PREEMPTIVE_XE_TARGETFINALIZE,
    /// Internal use only.
    PREEMPTIVE_XE_TARGETINIT,
    /// Internal use only.
    PREEMPTIVE_XE_TIMERRUN,
    /// Used to wait while user processes are ended in a database that has been transitioned by using the ALTER DATABASE termination clause. For more information, see ALTER DATABASE (Transact-SQL).
    PRINT_ROLLBACK_PROGRESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PRU_ROLLBACK_DEFERRED,
    /// Occurs when the persistent version store (PVS) cleanup process is waiting for a lock required to start the cleanup. Might occur when an active transaction is preventing PVS cleanup initiated internally or using the sys.sp_persistent_version_cleanup system stored procedure. For more information, see Monitor and troubleshoot accelerated database recovery.
    ///
    /// Applies to: SQL Server 2019 (15.x) and later versions.
    PVS_CLEANUP_LOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_ALL_COMPONENTS_INITIALIZED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_COOP_SCAN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PWAIT_DIRECTLOGCONSUMER_GETNEXT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_EVENT_SESSION_INIT_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    PWAIT_HADRSIM,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_ACTION_COMPLETED,
    /// Occurs when a background task is waiting for the termination of the background task that receives (via polling) Windows Server Failover Clustering notifications.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC,
    /// An append, replace, and/or remove operation is waiting to grab a write lock on an Always On internal list (such as a list of networks, network addresses, or availability group listeners). Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_CLUSTER_INTEGRATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_FAILOVER_COMPLETED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PWAIT_HADR_JOIN,
    /// A drop availability group operation is waiting for the target availability group to go offline before destroying Windows Server Failover Clustering objects.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_OFFLINE_COMPLETED,
    /// A create or failover availability group operation is waiting for the target availability group to come online.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_ONLINE_COMPLETED,
    /// A drop availability group operation is waiting for the termination of any background task that was scheduled as part of a previous command. For example, there might be a background task that is transitioning availability databases to the primary role. The DROP AVAILABILITY GROUP DDL must wait for this background task to terminate in order to avoid race conditions.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_POST_ONLINE_COMPLETED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_SERVER_READY_CONNECTIONS,
    /// Internal wait by a thread waiting for an async work task to complete. This is an expected wait and is for CSS use.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_HADR_WORKITEM_COMPLETED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PWAIT_LOG_CONSOLIDATION_IO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PWAIT_LOG_CONSOLIDATION_POLL,
    /// Occurs during internal synchronization in metadata on login stats.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_MD_LOGIN_STATS,
    /// Occurs during internal synchronization in metadata on table or index.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_MD_RELATION_CACHE,
    /// Occurs during internal synchronization in metadata on linked servers.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_MD_SERVER_CACHE,
    /// Occurs during internal synchronization in upgrading server wide configurations.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_MD_UPGRADE_CONFIG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    PWAIT_PREEMPTIVE_APP_USAGE_TIMER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_QRY_BPMEMORY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_REPLICA_ONLINE_INIT_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    PWAIT_SBS_FILE_OPERATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PWAIT_XTP_FSSTORAGE_MAINTENANCE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    PWAIT_XTP_HOST_STORAGE_WAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_ASYNC_CHECK_CONSISTENCY_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_ASYNC_PERSIST_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_ASYNC_PERSIST_TASK_START,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    QDS_ASYNC_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_BCKG_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    QDS_BLOOM_FILTER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_CTXS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_DB_DISK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_DYN_VECTOR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    QDS_EXCLUSIVE_ACCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    QDS_HOST_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_LOADDB,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_PERSIST_TASK_MAIN_LOOP_SLEEP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    QDS_QDS_CAPTURE_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_SHUTDOWN_QUEUE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_STMT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_STMT_DISK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_TASK_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QDS_TASK_START,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    QE_WARN_LIST_SYNC,
    /// Indicates that an asynchronous automatic statistics update was canceled by a call to KILL as the update was starting to run. The terminating thread is suspended, waiting for it to start listening for KILL commands. A good value is less than one second.
    QPJOB_KILL,
    /// Indicates that an asynchronous automatic statistics update was canceled by a call to KILL when it was running. The update has now completed but is suspended until the terminating thread message coordination is complete. This is an ordinary but rare state, and should be very short. A good value is less than one second.
    QPJOB_WAITFOR_ABORT,
    /// Occurs when Query Execution memory management tries to control access to static grant information list. This state lists information about the current granted and waiting memory requests. This state is a simple access control state. There should never be a long wait on this state. If this mutex isn't released, all new memory-using queries stop responding.
    QRY_MEM_GRANT_INFO_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    QRY_PARALLEL_THREAD_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    QRY_PROFILE_LIST_MUTEX,
    /// Identified for informational purposes only. Not supported.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    QUERY_ERRHDL_SERVICE_DONE,
    /// Occurs in certain cases when offline create index build is run in parallel, and the different worker threads that are sorting synchronize access to the sort files.
    QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN,
    /// Occurs during synchronization of the garbage collection queue in the Query Notification Manager.
    QUERY_NOTIFICATION_MGR_MUTEX,
    /// Occurs during state synchronization for transactions in Query Notifications.
    QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX,
    /// Occurs during internal synchronization within the Query Notification Manager.
    QUERY_NOTIFICATION_TABLE_MGR_MUTEX,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    QUERY_NOTIFICATION_UNITTEST_MUTEX,
    /// Occurs during synchronization of query optimizer diagnostic output production. This wait type only occurs if diagnostic settings have been enabled under direction of Microsoft Product Support.
    QUERY_OPTIMIZER_PRINT_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    QUERY_TASK_ENQUEUE_MUTEX,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    QUERY_TRACEOUT,
    /// Identified for informational purposes only. Not supported.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    QUERY_WAIT_ERRHDL_SERVICE,
    /// Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the long term log storage.
    ///
    /// Applies to: Azure SQL Database Hyperscale.
    RBIO_RG_DESTAGE,
    /// Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the log service.
    ///
    /// Applies to: Azure SQL Database Hyperscale.
    RBIO_RG_LOCALDESTAGE,
    /// Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the readable secondary replica nodes.
    ///
    /// Applies to: Azure SQL Database Hyperscale.
    RBIO_RG_REPLICA,
    /// Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption at the page servers.
    ///
    /// Applies to: Azure SQL Database Hyperscale.
    RBIO_RG_STORAGE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    RBIO_WAIT_VLF,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    RECOVERY_MGR_LOCK,
    /// Occurs during synchronization of database status in warm standby database.
    RECOVER_CHANGEDB,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    REDO_THREAD_PENDING_WORK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    REDO_THREAD_SYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    REMOTE_BLOCK_IO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    REMOTE_DATA_ARCHIVE_MIGRATION_DMV,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    REMOTE_DATA_ARCHIVE_SCHEMA_DMV,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE,
    /// Occurs while a task waits for completion of page writes to database snapshots or DBCC replicas.
    REPLICA_WRITES,
    /// Occurs during synchronization on a replication article cache. During these waits, the replication log reader stalls, and data definition language (DDL) statements on a published table are blocked.
    REPL_CACHE_ACCESS,
    /// Internal use only.
    REPL_HISTORYCACHE_ACCESS,
    /// Occurs during synchronization of replication schema version information. This state exists when DDL statements are executed on the replicated object, and when the log reader builds or consumes versioned schema based on DDL occurrence. Contention can be seen on this wait type if you have many published databases on a single publisher with transactional replication and the published databases are very active.
    REPL_SCHEMA_ACCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    REPL_TRANFSINFO_ACCESS,
    /// Internal use only.
    REPL_TRANHASHTABLE_ACCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    REPL_TRANTEXTINFO_ACCESS,
    /// Occurs when a task is waiting for all outstanding I/O to complete, so that I/O to a file can be frozen for snapshot backup.
    REQUEST_DISPENSER_PAUSE,
    /// Occurs while the deadlock monitor waits to start the next deadlock search. This wait is expected between deadlock detections, and lengthy total waiting time on this resource doesn't indicate a problem.
    REQUEST_FOR_DEADLOCK_SEARCH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    RESERVED_MEMORY_ALLOCATION_EXT,
    /// Occurs when a new request comes in and is throttled based on the GROUP_MAX_REQUESTS setting.
    RESMGR_THROTTLED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    RESOURCE_GOVERNOR_IDLE,
    /// Occurs during synchronization of various internal resource queues.
    RESOURCE_QUEUE,
    /// Occurs when a query memory request during query execution can't be granted immediately due to other concurrent queries. High waits and wait times might indicate excessive number of concurrent queries, or excessive memory request amounts. Excessive waits of this type might raise SQL error 8645, "A time out occurred while waiting for memory resources to execute the query. Rerun the query."
    ///
    /// For detailed information and troubleshooting ideas on memory grant waits, see Troubleshoot slow performance or low memory issues caused by memory grants in SQL Server.
    RESOURCE_SEMAPHORE,
    /// Occurs while a query waits for its request for a thread reservation to be fulfilled. It also occurs when synchronizing query compile and memory grant requests.
    RESOURCE_SEMAPHORE_MUTEX,
    /// Occurs when the number of concurrent query compilations reaches a throttling limit. High waits and wait times might indicate excessive compilations, recompiles, or uncacheable plans.
    RESOURCE_SEMAPHORE_QUERY_COMPILE,
    /// Occurs when memory request by a small query can't be granted immediately due to other concurrent queries. Wait time shouldn't exceed more than a few seconds, because the server transfers the request to the main query memory pool if it fails to grant the requested memory within a few seconds. High waits might indicate an excessive number of concurrent small queries while the main memory pool is blocked by waiting queries.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    RESOURCE_SEMAPHORE_SMALL_QUERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    RESTORE_FILEHANDLECACHE_ENTRYLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    RESTORE_FILEHANDLECACHE_LOCK,
    /// Internal use only.
    RG_RECONFIG,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    ROWGROUP_OP_STATS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    ROWGROUP_VERSION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    RTDATA_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SATELLITE_CARGO,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SATELLITE_SERVICE_SETUP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SATELLITE_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    SBS_DISPATCH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    SBS_RECEIVE_TRANSPORT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    SBS_TRANSPORT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SCAN_CHAR_HASH_ARRAY_INITIALIZATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    SECURITY_CNG_PROVIDER_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SECURITY_CRYPTO_CONTEXT_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SECURITY_DBE_STATE_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SECURITY_KEYRING_RWLOCK,
    /// Occurs when there's a wait for mutexes that control access to the global list of Extensible Key Management (EKM) cryptographic providers and the session-scoped list of EKM sessions.
    SECURITY_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SECURITY_RULETABLE_MUTEX,
    /// Occurs after a failed attempt to drop a temporary security key before a retry attempt.
    SEC_DROP_TEMP_KEY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SEMPLAT_DSI_BUILD,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SEQUENCE_GENERATION,
    /// Occurs while a new sequential GUID is being obtained.
    SEQUENTIAL_GUID,
    /// Occurs during synchronization of SQL Server instance idle status when a resource monitor is attempting to declare a SQL Server instance as idle or trying to wake up.
    SERVER_IDLE_CHECK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SERVER_RECONFIGURE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SESSION_WAIT_STATS_CHILDREN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SHARED_DELTASTORE_CREATION,
    /// Occurs while a shutdown statement waits for active connections to exit.
    SHUTDOWN,
    /// Occurs when a checkpoint is throttling the issuance of new I/Os in order to avoid flooding the disk subsystem.
    SLEEP_BPOOL_FLUSH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SLEEP_BUFFERPOOL_HELPLW,
    /// Occurs during database startup while waiting for all databases to recover.
    SLEEP_DBSTARTUP,
    /// Occurs once at most during SQL Server instance startup while waiting for DCOM initialization to complete.
    SLEEP_DCOMSTARTUP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SLEEP_MASTERDBREADY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SLEEP_MASTERMDREADY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SLEEP_MASTERUPGRADED,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SLEEP_MEMORYPOOL_ALLOCATEPAGES,
    /// Occurs when SQL Trace waits for the msdb database to complete startup.
    SLEEP_MSDBSTARTUP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SLEEP_RETRY_VIRTUALALLOC,
    /// Occurs during the start of a background task while waiting for tempdb to complete startup.
    SLEEP_SYSTEMTASK,
    /// Occurs when a task sleeps while waiting for a generic event to occur.
    SLEEP_TASK,
    /// Occurs while a task waits for tempdb to complete startup.
    SLEEP_TEMPDBSTARTUP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SLEEP_WORKSPACE_ALLOCATEPAGE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    SLO_UPDATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SMSYNC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    SNI_CONN_DUP,
    /// Occurs during internal synchronization within SQL Server networking components.
    SNI_CRITICAL_SECTION,
    /// Occurs during SQL Server shutdown, while waiting for outstanding HTTP connections to exit.
    SNI_HTTP_WAITFOR_0_DISCON,
    /// Occurs while waiting for non-uniform memory access (NUMA) nodes to update state change. Access to state change is serialized.
    SNI_LISTENER_ACCESS,
    /// Occurs when there's a wait for all tasks to finish during a NUMA node state change.
    SNI_TASK_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    SNI_WRITE_ASYNC,
    /// Occurs while waiting for an HTTP network read to complete.
    SOAP_READ,
    /// Occurs while waiting for an HTTP network write to complete.
    SOAP_WRITE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    SOCKETDUPLICATEQUEUE_CLEANUP,
    /// Occurs when a hosted component, such as CLR, waits on a SQL Server event synchronization object.
    SOSHOST_EVENT,
    /// Occurs during synchronization of memory manager callbacks used by hosted components, such as CLR.
    SOSHOST_INTERNAL,
    /// Occurs when a hosted component, such as CLR, waits on a SQL Server mutex synchronization object.
    SOSHOST_MUTEX,
    /// Occurs when a hosted component, such as CLR, waits on a SQL Server reader-writer synchronization object.
    SOSHOST_RWLOCK,
    /// Occurs when a hosted component, such as CLR, waits on a SQL Server semaphore synchronization object.
    SOSHOST_SEMAPHORE,
    /// Occurs when a hosted task sleeps while waiting for a generic event to occur. Hosted tasks are used by hosted components such as CLR.
    SOSHOST_SLEEP,
    /// Occurs during synchronization of access to trace streams.
    SOSHOST_TRACELOCK,
    /// Occurs when a hosted component, such as CLR, waits for a task to complete.
    SOSHOST_WAITFORDONE,
    /// Occurs while performing synchronization on a callback list in order to remove a callback. It isn't expected for this counter to change after server initialization is completed.
    SOS_CALLBACK_REMOVAL,
    /// Occurs during internal synchronization of the dispatcher pool. This includes when the pool is being adjusted.
    SOS_DISPATCHER_MUTEX,
    /// Occurs during internal synchronization in the SQL Server memory manager.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    SOS_LOCALALLOCATORLIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SOS_MEMORY_TOPLEVELBLOCKALLOCATOR,
    /// Occurs when memory usage is being adjusted among pools.
    SOS_MEMORY_USAGE_ADJUSTMENT,
    /// Occurs during internal synchronization in memory pools when destroying objects from the pool.
    SOS_OBJECT_STORE_DESTROY_MUTEX,
    /// Accounts for the time a thread waits to acquire the mutex it must acquire before it allocates physical pages or before it returns those pages to the operating system. Waits on this type only appear if the instance of SQL Server uses AWE memory.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SOS_PHYS_PAGE_CACHE,
    /// Occurs during synchronizing of access to process affinity settings.
    SOS_PROCESS_AFFINITY_MUTEX,
    /// Occurs during internal synchronization in the SQL Server Memory Manager.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    SOS_RESERVEDMEMBLOCKLIST,
    /// Occurs when a task voluntarily yields the scheduler for other tasks to execute. During this wait, the task is waiting in a runnable queue for its quantum to be renewed, that is, waiting to be scheduled to run on the CPU again. Prolonged waits on this wait type most frequently indicate opportunities to optimize queries that perform index or table scans. Focus on plan regression, missing index, stats updates, and query rewrites. Optimizing runtimes reduces the need for tasks to be yielding multiple times. If query times for such CPU-consuming tasks are acceptable, then this wait type is expected and can be ignored.
    SOS_SCHEDULER_YIELD,
    /// Occurs during the allocation and freeing of memory that is managed by some memory objects.
    SOS_SMALL_PAGE_ALLOC,
    /// Occurs during synchronization of internal store initialization.
    SOS_STACKSTORE_INIT_MUTEX,
    /// Occurs when a task is started in a synchronous manner. Most tasks in SQL Server are started in an asynchronous manner, in which control returns to the starter immediately after the task request has been placed on the work queue.
    SOS_SYNC_TASK_ENQUEUE_EVENT,
    /// Occurs when a memory allocation waits for a Resource Manager to free up virtual memory.
    SOS_VIRTUALMEMORY_LOW,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2019 (15.x) and later versions.
    SOS_WORK_DISPATCHER,
    /// Occurs when a thread is waiting to acquire a spinlock. Includes both the spinning and the sleeping time. High values might indicate spinlock contention.
    ///
    /// Because of a possibility of a minor performance impact with high throughput and high concurrency workloads, the SPINLOCK_EXT waits are tracked only if trace flag 8134 is enabled.
    ///
    /// Applies to: SQL Server 2025 (17.x) and later versions.
    SPINLOCK_EXT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SP_SERVER_DIAGNOSTICS_INIT_MUTEX,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SP_SERVER_DIAGNOSTICS_SLEEP,
    /// Occurs while CLR waits for an application domain to complete startup.
    SQLCLR_APPDOMAIN,
    /// Occurs while waiting for access to the loaded assembly list in the appdomain.
    SQLCLR_ASSEMBLY,
    /// Occurs while CLR waits for deadlock detection to complete.
    SQLCLR_DEADLOCK_DETECTION,
    /// Occurs when a CLR task is throttled because it has exceeded its execution quantum. This throttling is done in order to reduce the effect of this resource-intensive task on other tasks.
    SQLCLR_QUANTUM_PUNISHMENT,
    /// Occurs during internal synchronization, while initializing internal sorting structures.
    SQLSORT_NORMMUTEX,
    /// Occurs during internal synchronization, while initializing internal sorting structures.
    SQLSORT_SORTMUTEX,
    /// Occurs when a task is waiting for a background task to flush trace buffers to disk every four seconds.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    SQLTRACE_BUFFER_FLUSH,
    /// Occurs during synchronization on trace buffers during a file trace.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SQLTRACE_FILE_BUFFER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SQLTRACE_FILE_READ_IO_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SQLTRACE_FILE_WRITE_IO_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SQLTRACE_INCREMENTAL_FLUSH_SLEEP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    SQLTRACE_LOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    SQLTRACE_PENDING_BUFFER_WRITERS,
    /// Occurs while trace shutdown waits for outstanding trace events to complete.
    SQLTRACE_SHUTDOWN,
    /// Occurs while a SQL Trace event queue waits for packets to arrive on the queue.
    SQLTRACE_WAIT_ENTRIES,
    /// Occurs while the shutdown process waits for internal resources to be released to shut down cleanly.
    SRVPROC_SHUTDOWN,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    STARTUP_DEPENDENCY_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    TDS_BANDWIDTH_STATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    TDS_INIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    TDS_PROXY_CONTAINER,
    /// Occurs when temporary object drops are synchronized. This wait is rare, and only occurs if a task has requested exclusive access for temp table drops.
    TEMPOBJ,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    TEMPORAL_BACKGROUND_PROCEED_CLEANUP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    TERMINATE_LISTENER,
    /// Occurs when a task (query or login/logout) is waiting for a worker thread to execute it. This can indicate that the maximum worker thread setting is misconfigured, or, most commonly, that batch executions are taking unusually long, thus reducing the number of worker threads available to satisfy other batches. Examine the performance of batches (queries) and reduce query duration by either reducing bottlenecks (blocking, parallelism, I/O, latch waits), or providing proper indexing or query design.
    THREADPOOL,
    /// Occurs during internal synchronization of the Extended Events timer.
    TIMEPRIV_TIMEPERIOD,
    /// Occurs when the SQL Trace rowset trace provider waits for either a free buffer or a buffer with events to process.
    TRACEWRITE,
    /// Internal use only.
    TRACE_EVTNOTIF,
    /// Occurs during synchronization of access to a transaction by multiple batches.
    TRANSACTION_MUTEX,
    /// Occurs when waiting for a destroy mode latch on a transaction mark latch. Transaction mark latches are used for synchronization of commits with marked transactions.
    TRAN_MARKLATCH_DT,
    /// Occurs when waiting for an exclusive mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.
    TRAN_MARKLATCH_EX,
    /// Occurs when waiting for a keep mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.
    TRAN_MARKLATCH_KP,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    TRAN_MARKLATCH_NL,
    /// Occurs when waiting for a shared mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.
    TRAN_MARKLATCH_SH,
    /// Occurs when waiting for an update mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.
    TRAN_MARKLATCH_UP,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_ENDPOINT_CHANGE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_MANAGER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_MEMORY_NOTIFICATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_SESSION_REGISTRATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_TRANSPORT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    UCS_TRANSPORT_STREAM_CHANGE,
    /// Occurs when transaction log scans wait for memory to be available during memory pressure.
    UTIL_PAGE_ALLOC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    VDI_CLIENT_COMPLETECOMMAND,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    VDI_CLIENT_GETCOMMAND,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    VDI_CLIENT_OPERATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    VDI_CLIENT_OTHER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    VERSIONING_COMMITTING,
    /// Occurs when a Virtual Interface Adapter (VIA) provider connection is completed during startup.
    VIA_ACCEPT,
    /// Occurs during synchronization on access to cached view definitions.
    VIEW_DEFINITION_MUTEX,
    /// Occurs as a result of a WAITFOR Transact-SQL statement. The duration of the wait is determined by the parameters to the statement. This is a user-initiated wait.
    WAITFOR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    WAITFOR_PER_QUEUE,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    WAITFOR_TASKSHUTDOWN,
    /// Occurs during synchronization of access to the collection of statistics used to populate sys.dm_os_wait_stats.
    WAITSTAT_MUTEX,
    /// Occurs when waiting for a query notification to be triggered.
    WAIT_FOR_RESULTS,
    /// Occurs when waiting for synchronous statistics update to complete before query compilation and execution can resume.
    ///
    /// Applies to: Starting with SQL Server 2019 (15.x)
    WAIT_ON_SYNC_STATISTICS_REFRESH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_SCRIPTDEPLOYMENT_REQUEST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_SCRIPTDEPLOYMENT_WORKER,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    WAIT_XLOGREAD_SIGNAL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_ASYNC_TX_COMPLETION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_CKPT_AGENT_WAKEUP,
    /// Occurs when waiting for a checkpoint to complete.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_CKPT_CLOSE,
    /// Occurs when checkpointing is disabled, and waiting for checkpointing to be enabled.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_CKPT_ENABLED,
    /// Occurs when synchronizing checking of checkpoint state.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_CKPT_STATE_LOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    WAIT_XTP_COMPILE_WAIT,
    /// Occurs when the database memory allocator needs to stop receiving low-memory notifications.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    WAIT_XTP_GUEST,
    /// Occurs when waits are triggered by the database engine and implemented by the host.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_HOST_WAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO,
    /// Occurs when offline checkpoint is waiting for a log read IO to complete.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_OFFLINE_CKPT_LOG_IO,
    /// Occurs when offline checkpoint is waiting for new log records to scan.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_OFFLINE_CKPT_NEW_LOG,
    /// Occurs when a drop procedure is waiting for all current executions of that procedure to complete.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_PROCEDURE_ENTRY,
    /// Occurs when database recovery is waiting for recovery of memory-optimized objects to finish.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    WAIT_XTP_SERIAL_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    WAIT_XTP_SWITCH_TO_INACTIVE,
    /// Occurs when waiting for an In-Memory OLTP thread to complete.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    WAIT_XTP_TASK_SHUTDOWN,
    /// Occurs when waiting for transaction dependencies.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WAIT_XTP_TRAN_DEPENDENCY,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    WCC,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    WINDOW_AGGREGATES_MULTIPASS,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WINFAB_API_CALL,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    WINFAB_REPLICA_BUILD_OPERATION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    WINFAB_REPORT_FAULT,
    /// Occurs while pausing before retrying, after a failed worktable drop.
    WORKTBL_DROP,
    /// Occurs while waiting for a log flush to complete. Common operations that cause log flushes are transaction commits and checkpoints. Common reasons for long waits on WRITELOG are: disk latency (where transaction log files reside), the inability for I/O to keep up with transactions, or, a large number of transaction log operations and flushes (commits, rollback)
    WRITELOG,
    /// Occurs when a write operation is in progress.
    WRITE_COMPLETION,
    /// Occurs during synchronization of access to the list of locks for a transaction. In addition to the transaction itself, the list of locks is accessed by operations such as deadlock detection and lock migration during page splits.
    XACTLOCKINFO,
    /// Occurs during synchronization of defections from a transaction, as well as the number of database locks between enlist members of a transaction.
    XACTWORKSPACE_MUTEX,
    /// Occurs while waiting to acquire ownership of a transaction.
    XACT_OWN_TRANSACTION,
    /// Occurs while waiting for the current owner of a session to release ownership of the session.
    XACT_RECLAIM_SESSION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XDB_CONN_DUP_HASH,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XDESTSVERMGR,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XDES_HISTORY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XDES_OUT_OF_ORDER_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XDES_SNAPSHOT,
    /// Occurs when Extended Events session buffers are flushed to targets. This wait occurs on a background thread.
    XE_BUFFERMGR_ALLPROCESSED_EVENT,
    /// Occurs when either of the following conditions is true:
    ///
    /// - An Extended Events session is configured for no event loss, and all buffers in the session are currently full. This can indicate that the buffers for an Extended Events session are too small or should be partitioned.
    /// - Audits experience a delay. This can indicate a disk bottleneck on the drive where the audits are written.
    XE_BUFFERMGR_FREEBUF_EVENT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XE_CALLBACK_LIST,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    XE_CX_FILE_READ,
    /// Occurs when an Extended Events session that is using asynchronous targets is started or stopped. This wait indicates either of the following conditions:
    ///
    /// - An Extended Events session is registering with a background thread pool.
    /// - The background thread pool is calculating the required number of threads based on current load.
    XE_DISPATCHER_CONFIG_SESSION_LIST,
    /// Occurs when a background thread that is used for Extended Events sessions is terminating.
    XE_DISPATCHER_JOIN,
    /// Occurs when a background thread that is used for Extended Events sessions is waiting for event buffers to process.
    XE_DISPATCHER_WAIT,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XE_FILE_TARGET_TVF,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XE_LIVE_TARGET_TVF,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    XE_MODULEMGR_SYNC,
    /// Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.
    XE_OLS_LOCK,
    /// Identified for informational purposes only. Not supported.
    ///
    /// Applies to: SQL Server 2008 R2 (10.50.x) only.
    XE_PACKAGE_LOCK_BACKOFF,
    /// Internal use only.
    XE_SERVICES_EVENTMANUAL,
    /// Internal use only.
    XE_SERVICES_MUTEX,
    /// Internal use only.
    XE_SERVICES_RWLOCK,
    /// Internal use only.
    XE_SESSION_CREATE_SYNC,
    /// Internal use only.
    XE_SESSION_FLUSH,
    /// Internal use only.
    XE_SESSION_SYNC,
    /// Internal use only.
    XE_STM_CREATE,
    /// Internal use only.
    XE_TIMER_EVENT,
    /// Internal use only.
    XE_TIMER_MUTEX,
    /// Internal use only.
    XE_TIMER_TASK_DONE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XIO_CREDENTIAL_MGR_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XIO_CREDENTIAL_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    XIO_EDS_MGR_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    XIO_EDS_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    XIO_IOSTATS_BLOBLIST_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2017 (14.x) and later versions.
    XIO_IOSTATS_FCBLIST_RWLOCK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XIO_LEASE_RENEW_MGR_RWLOCK,
    /// Occurs when for accessing all natively compiled stored procedure cache objects.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    XTPPROC_CACHE_ACCESS,
    /// Occurs when allocating per-NUMA node natively compiled stored procedure cache structures (must be done single threaded) for a given procedure.
    ///
    /// Applies to: SQL Server 2012 (11.x) and later versions.
    XTPPROC_PARTITIONED_STACK_CREATE,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    XTP_HOST_DB_COLLECTION,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2014 (12.x) and later versions.
    XTP_HOST_LOG_ACTIVITY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XTP_HOST_PARALLEL_RECOVERY,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XTP_PREEMPTIVE_TASK,
    /// Internal use only.
    ///
    /// Applies to: SQL Server 2016 (13.x) and later versions.
    XTP_TRUNCATION_LSN,
    /// A wait type not present in the generated table.
    Unknown(String),
}

impl WaitType {
    pub fn parse(s: &str) -> WaitType {
        match s {
            "ABR" => WaitType::ABR,
            "AM_INDBUILD_ALLOCATION" => WaitType::AM_INDBUILD_ALLOCATION,
            "AM_SCHEMAMGR_UNSHARED_CACHE" => WaitType::AM_SCHEMAMGR_UNSHARED_CACHE,
            "ASSEMBLY_FILTER_HASHTABLE" => WaitType::ASSEMBLY_FILTER_HASHTABLE,
            "ASSEMBLY_LOAD" => WaitType::ASSEMBLY_LOAD,
            "ASYNC_DISKPOOL_LOCK" => WaitType::ASYNC_DISKPOOL_LOCK,
            "ASYNC_IO_COMPLETION" => WaitType::ASYNC_IO_COMPLETION,
            "ASYNC_NETWORK_IO" => WaitType::ASYNC_NETWORK_IO,
            "ASYNC_OP_COMPLETION" => WaitType::ASYNC_OP_COMPLETION,
            "ASYNC_OP_CONTEXT_READ" => WaitType::ASYNC_OP_CONTEXT_READ,
            "ASYNC_OP_CONTEXT_WRITE" => WaitType::ASYNC_OP_CONTEXT_WRITE,
            "ASYNC_SOCKETDUP_IO" => WaitType::ASYNC_SOCKETDUP_IO,
            "AUDIT_GROUPCACHE_LOCK" => WaitType::AUDIT_GROUPCACHE_LOCK,
            "AUDIT_LOGINCACHE_LOCK" => WaitType::AUDIT_LOGINCACHE_LOCK,
            "AUDIT_ON_DEMAND_TARGET_LOCK" => WaitType::AUDIT_ON_DEMAND_TARGET_LOCK,
            "AUDIT_XE_SESSION_MGR" => WaitType::AUDIT_XE_SESSION_MGR,
            "BACKUP" => WaitType::BACKUP,
            "BACKUPBUFFER" => WaitType::BACKUPBUFFER,
            "BACKUPIO" => WaitType::BACKUPIO,
            "BACKUPTHREAD" => WaitType::BACKUPTHREAD,
            "BACKUP_OPERATOR" => WaitType::BACKUP_OPERATOR,
            "BAD_PAGE_PROCESS" => WaitType::BAD_PAGE_PROCESS,
            "BLOB_METADATA" => WaitType::BLOB_METADATA,
            "BMPALLOCATION" => WaitType::BMPALLOCATION,
            "BMPBUILD" => WaitType::BMPBUILD,
            "BMPREPARTITION" => WaitType::BMPREPARTITION,
            "BMPREPLICATION" => WaitType::BMPREPLICATION,
            "BPSORT" => WaitType::BPSORT,
            "BROKER_CONNECTION_RECEIVE_TASK" => WaitType::BROKER_CONNECTION_RECEIVE_TASK,
            "BROKER_DISPATCHER" => WaitType::BROKER_DISPATCHER,
            "BROKER_ENDPOINT_STATE_MUTEX" => WaitType::BROKER_ENDPOINT_STATE_MUTEX,
            "BROKER_EVENTHANDLER" => WaitType::BROKER_EVENTHANDLER,
            "BROKER_FORWARDER" => WaitType::BROKER_FORWARDER,
            "BROKER_INIT" => WaitType::BROKER_INIT,
            "BROKER_MASTERSTART" => WaitType::BROKER_MASTERSTART,
            "BROKER_RECEIVE_WAITFOR" => WaitType::BROKER_RECEIVE_WAITFOR,
            "BROKER_REGISTERALLENDPOINTS" => WaitType::BROKER_REGISTERALLENDPOINTS,
            "BROKER_SERVICE" => WaitType::BROKER_SERVICE,
            "BROKER_SHUTDOWN" => WaitType::BROKER_SHUTDOWN,
            "BROKER_START" => WaitType::BROKER_START,
            "BROKER_TASK_SHUTDOWN" => WaitType::BROKER_TASK_SHUTDOWN,
            "BROKER_TASK_STOP" => WaitType::BROKER_TASK_STOP,
            "BROKER_TASK_SUBMIT" => WaitType::BROKER_TASK_SUBMIT,
            "BROKER_TO_FLUSH" => WaitType::BROKER_TO_FLUSH,
            "BROKER_TRANSMISSION_OBJECT" => WaitType::BROKER_TRANSMISSION_OBJECT,
            "BROKER_TRANSMISSION_TABLE" => WaitType::BROKER_TRANSMISSION_TABLE,
            "BROKER_TRANSMISSION_WORK" => WaitType::BROKER_TRANSMISSION_WORK,
            "BROKER_TRANSMITTER" => WaitType::BROKER_TRANSMITTER,
            "BUFFERPOOL_SCAN" => WaitType::BUFFERPOOL_SCAN,
            "BUILTIN_HASHKEY_MUTEX" => WaitType::BUILTIN_HASHKEY_MUTEX,
            "CHANGE_TRACKING_WAITFORCHANGES" => WaitType::CHANGE_TRACKING_WAITFORCHANGES,
            "CHECKPOINT_QUEUE" => WaitType::CHECKPOINT_QUEUE,
            "CHECK_PRINT_RECORD" => WaitType::CHECK_PRINT_RECORD,
            "CHECK_SCANNER_MUTEX" => WaitType::CHECK_SCANNER_MUTEX,
            "CHECK_TABLES_INITIALIZATION" => WaitType::CHECK_TABLES_INITIALIZATION,
            "CHECK_TABLES_SINGLE_SCAN" => WaitType::CHECK_TABLES_SINGLE_SCAN,
            "CHECK_TABLES_THREAD_BARRIER" => WaitType::CHECK_TABLES_THREAD_BARRIER,
            "CHKPT" => WaitType::CHKPT,
            "CLEAR_DB" => WaitType::CLEAR_DB,
            "CLRHOST_STATE_ACCESS" => WaitType::CLRHOST_STATE_ACCESS,
            "CLR_AUTO_EVENT" => WaitType::CLR_AUTO_EVENT,
            "CLR_CRST" => WaitType::CLR_CRST,
            "CLR_JOIN" => WaitType::CLR_JOIN,
            "CLR_MANUAL_EVENT" => WaitType::CLR_MANUAL_EVENT,
            "CLR_MEMORY_SPY" => WaitType::CLR_MEMORY_SPY,
            "CLR_MONITOR" => WaitType::CLR_MONITOR,
            "CLR_RWLOCK_READER" => WaitType::CLR_RWLOCK_READER,
            "CLR_RWLOCK_WRITER" => WaitType::CLR_RWLOCK_WRITER,
            "CLR_SEMAPHORE" => WaitType::CLR_SEMAPHORE,
            "CLR_TASK_START" => WaitType::CLR_TASK_START,
            "CMEMPARTITIONED" => WaitType::CMEMPARTITIONED,
            "CMEMTHREAD" => WaitType::CMEMTHREAD,
            "COLUMNSTORE_BUILD_THROTTLE" => WaitType::COLUMNSTORE_BUILD_THROTTLE,
            "COLUMNSTORE_COLUMNDATASET_SESSION_LIST" => WaitType::COLUMNSTORE_COLUMNDATASET_SESSION_LIST,
            "COMMIT_TABLE" => WaitType::COMMIT_TABLE,
            "CONNECTION_ENDPOINT_LOCK" => WaitType::CONNECTION_ENDPOINT_LOCK,
            "COUNTRECOVERYMGR" => WaitType::COUNTRECOVERYMGR,
            "CREATE_DATINISERVICE" => WaitType::CREATE_DATINISERVICE,
            "CXCONSUMER" => WaitType::CXCONSUMER,
            "CXPACKET" => WaitType::CXPACKET,
            "CXROWSET_SYNC" => WaitType::CXROWSET_SYNC,
            "CXSYNC_CONSUMER" => WaitType::CXSYNC_CONSUMER,
            "CXSYNC_PORT" => WaitType::CXSYNC_PORT,
            "DAC_INIT" => WaitType::DAC_INIT,
            "DBCC_SCALE_OUT_EXPR_CACHE" => WaitType::DBCC_SCALE_OUT_EXPR_CACHE,
            "DBMIRRORING_CMD" => WaitType::DBMIRRORING_CMD,
            "DBMIRROR_DBM_EVENT" => WaitType::DBMIRROR_DBM_EVENT,
            "DBMIRROR_DBM_MUTEX" => WaitType::DBMIRROR_DBM_MUTEX,
            "DBMIRROR_EVENTS_QUEUE" => WaitType::DBMIRROR_EVENTS_QUEUE,
            "DBMIRROR_SEND" => WaitType::DBMIRROR_SEND,
            "DBMIRROR_WORKER_QUEUE" => WaitType::DBMIRROR_WORKER_QUEUE,
            "DBSEEDING_FLOWCONTROL" => WaitType::DBSEEDING_FLOWCONTROL,
            "DBSEEDING_OPERATION" => WaitType::DBSEEDING_OPERATION,
            "DEADLOCK_ENUM_MUTEX" => WaitType::DEADLOCK_ENUM_MUTEX,
            "DEADLOCK_TASK_SEARCH" => WaitType::DEADLOCK_TASK_SEARCH,
            "DEBUG" => WaitType::DEBUG,
            "DIRECTLOGCONSUMER_LIST" => WaitType::DIRECTLOGCONSUMER_LIST,
            "DIRTY_PAGE_POLL" => WaitType::DIRTY_PAGE_POLL,
            "DIRTY_PAGE_SYNC" => WaitType::DIRTY_PAGE_SYNC,
            "DIRTY_PAGE_TABLE_LOCK" => WaitType::DIRTY_PAGE_TABLE_LOCK,
            "DISABLE_VERSIONING" => WaitType::DISABLE_VERSIONING,
            "DISKIO_SUSPEND" => WaitType::DISKIO_SUSPEND,
            "DISPATCHER_PRIORITY_QUEUE_SEMAPHORE" => WaitType::DISPATCHER_PRIORITY_QUEUE_SEMAPHORE,
            "DISPATCHER_QUEUE_SEMAPHORE" => WaitType::DISPATCHER_QUEUE_SEMAPHORE,
            "DLL_LOADING_MUTEX" => WaitType::DLL_LOADING_MUTEX,
            "DPT_ENTRY_LOCK" => WaitType::DPT_ENTRY_LOCK,
            "DROPTEMP" => WaitType::DROPTEMP,
            "DROP_DATABASE_TIMER_TASK" => WaitType::DROP_DATABASE_TIMER_TASK,
            "DTC" => WaitType::DTC,
            "DTCNEW_ENLIST" => WaitType::DTCNEW_ENLIST,
            "DTCNEW_PREPARE" => WaitType::DTCNEW_PREPARE,
            "DTCNEW_RECOVERY" => WaitType::DTCNEW_RECOVERY,
            "DTCNEW_TM" => WaitType::DTCNEW_TM,
            "DTCNEW_TRANSACTION_ENLISTMENT" => WaitType::DTCNEW_TRANSACTION_ENLISTMENT,
            "DTCPNTSYNC" => WaitType::DTCPNTSYNC,
            "DTC_ABORT_REQUEST" => WaitType::DTC_ABORT_REQUEST,
            "DTC_RESOLVE" => WaitType::DTC_RESOLVE,
            "DTC_STATE" => WaitType::DTC_STATE,
            "DTC_TMDOWN_REQUEST" => WaitType::DTC_TMDOWN_REQUEST,
            "DTC_WAITFOR_OUTCOME" => WaitType::DTC_WAITFOR_OUTCOME,
            "DUMPTRIGGER" => WaitType::DUMPTRIGGER,
            "DUMP_LOG_COORDINATOR" => WaitType::DUMP_LOG_COORDINATOR,
            "DUMP_LOG_COORDINATOR_QUEUE" => WaitType::DUMP_LOG_COORDINATOR_QUEUE,
            "EC" => WaitType::EC,
            "EE_PMOLOCK" => WaitType::EE_PMOLOCK,
            "EE_SPECPROC_MAP_INIT" => WaitType::EE_SPECPROC_MAP_INIT,
            "ENABLE_EMPTY_VERSIONING" => WaitType::ENABLE_EMPTY_VERSIONING,
            "ENABLE_VERSIONING" => WaitType::ENABLE_VERSIONING,
            "ERROR_REPORTING_MANAGER" => WaitType::ERROR_REPORTING_MANAGER,
            "EXCHANGE" => WaitType::EXCHANGE,
            "EXECSYNC" => WaitType::EXECSYNC,
            "EXECUTION_PIPE_EVENT_INTERNAL" => WaitType::EXECUTION_PIPE_EVENT_INTERNAL,
            "EXTERNAL_RG_UPDATE" => WaitType::EXTERNAL_RG_UPDATE,
            "EXTERNAL_SCRIPT_NETWORK_IO" => WaitType::EXTERNAL_SCRIPT_NETWORK_IO,
            "EXTERNAL_SCRIPT_PREPARE_SERVICE" => WaitType::EXTERNAL_SCRIPT_PREPARE_SERVICE,
            "EXTERNAL_SCRIPT_SHUTDOWN" => WaitType::EXTERNAL_SCRIPT_SHUTDOWN,
            "EXTERNAL_WAIT_ON_LAUNCHER" => WaitType::EXTERNAL_WAIT_ON_LAUNCHER,
            "FABRIC_HADR_TRANSPORT_CONNECTION" => WaitType::FABRIC_HADR_TRANSPORT_CONNECTION,
            "FABRIC_REPLICA_CONTROLLER_LIST" => WaitType::FABRIC_REPLICA_CONTROLLER_LIST,
            "FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG" => WaitType::FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG,
            "FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH" => WaitType::FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH,
            "FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST" => WaitType::FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST,
            "FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING" => WaitType::FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING,
            "FAILPOINT" => WaitType::FAILPOINT,
            "FCB_REPLICA_READ" => WaitType::FCB_REPLICA_READ,
            "FCB_REPLICA_WRITE" => WaitType::FCB_REPLICA_WRITE,
            "FEATURE_SWITCHES_UPDATE" => WaitType::FEATURE_SWITCHES_UPDATE,
            "FFT_NSO_DB_KILL_FLAG" => WaitType::FFT_NSO_DB_KILL_FLAG,
            "FFT_NSO_DB_LIST" => WaitType::FFT_NSO_DB_LIST,
            "FFT_NSO_FCB" => WaitType::FFT_NSO_FCB,
            "FFT_NSO_FCB_FIND" => WaitType::FFT_NSO_FCB_FIND,
            "FFT_NSO_FCB_PARENT" => WaitType::FFT_NSO_FCB_PARENT,
            "FFT_NSO_FCB_RELEASE_CACHED_ENTRIES" => WaitType::FFT_NSO_FCB_RELEASE_CACHED_ENTRIES,
            "FFT_NSO_FCB_STATE" => WaitType::FFT_NSO_FCB_STATE,
            "FFT_NSO_FILEOBJECT" => WaitType::FFT_NSO_FILEOBJECT,
            "FFT_NSO_TABLE_LIST" => WaitType::FFT_NSO_TABLE_LIST,
            "FFT_NTFS_STORE" => WaitType::FFT_NTFS_STORE,
            "FFT_RECOVERY" => WaitType::FFT_RECOVERY,
            "FFT_RSFX_COMM" => WaitType::FFT_RSFX_COMM,
            "FFT_RSFX_WAIT_FOR_MEMORY" => WaitType::FFT_RSFX_WAIT_FOR_MEMORY,
            "FFT_STARTUP_SHUTDOWN" => WaitType::FFT_STARTUP_SHUTDOWN,
            "FFT_STORE_DB" => WaitType::FFT_STORE_DB,
            "FFT_STORE_ROWSET_LIST" => WaitType::FFT_STORE_ROWSET_LIST,
            "FFT_STORE_TABLE" => WaitType::FFT_STORE_TABLE,
            "FILESTREAM_CACHE" => WaitType::FILESTREAM_CACHE,
            "FILESTREAM_CHUNKER" => WaitType::FILESTREAM_CHUNKER,
            "FILESTREAM_CHUNKER_INIT" => WaitType::FILESTREAM_CHUNKER_INIT,
            "FILESTREAM_FCB" => WaitType::FILESTREAM_FCB,
            "FILESTREAM_FILE_OBJECT" => WaitType::FILESTREAM_FILE_OBJECT,
            "FILESTREAM_WORKITEM_QUEUE" => WaitType::FILESTREAM_WORKITEM_QUEUE,
            "FILETABLE_SHUTDOWN" => WaitType::FILETABLE_SHUTDOWN,
            "FILE_VALIDATION_THREADS" => WaitType::FILE_VALIDATION_THREADS,
            "FOREIGN_REDO" => WaitType::FOREIGN_REDO,
            "FORWARDER_TRANSITION" => WaitType::FORWARDER_TRANSITION,
            "FSAGENT" => WaitType::FSAGENT,
            "FSA_FORCE_OWN_XACT" => WaitType::FSA_FORCE_OWN_XACT,
            "FSTR_CONFIG_MUTEX" => WaitType::FSTR_CONFIG_MUTEX,
            "FSTR_CONFIG_RWLOCK" => WaitType::FSTR_CONFIG_RWLOCK,
            "FS_FC_RWLOCK" => WaitType::FS_FC_RWLOCK,
            "FS_GARBAGE_COLLECTOR_SHUTDOWN" => WaitType::FS_GARBAGE_COLLECTOR_SHUTDOWN,
            "FS_HEADER_RWLOCK" => WaitType::FS_HEADER_RWLOCK,
            "FS_LOGTRUNC_RWLOCK" => WaitType::FS_LOGTRUNC_RWLOCK,
            "FT_COMPROWSET_RWLOCK" => WaitType::FT_COMPROWSET_RWLOCK,
            "FT_IFTSHC_MUTEX" => WaitType::FT_IFTSHC_MUTEX,
            "FT_IFTSISM_MUTEX" => WaitType::FT_IFTSISM_MUTEX,
            "FT_IFTS_ASYNC_WRITE_PIPE" => WaitType::FT_IFTS_ASYNC_WRITE_PIPE,
            "FT_IFTS_BLOB_HASH" => WaitType::FT_IFTS_BLOB_HASH,
            "FT_IFTS_CATEALOG_SOURCE" => WaitType::FT_IFTS_CATEALOG_SOURCE,
            "FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER" => WaitType::FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER,
            "FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST" => WaitType::FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST,
            "FT_IFTS_COMP_DESC_MANAGER" => WaitType::FT_IFTS_COMP_DESC_MANAGER,
            "FT_IFTS_CONSUMER_PLUGIN" => WaitType::FT_IFTS_CONSUMER_PLUGIN,
            "FT_IFTS_CRAWL_BATCH_LIST" => WaitType::FT_IFTS_CRAWL_BATCH_LIST,
            "FT_IFTS_CRAWL_CHILDREN" => WaitType::FT_IFTS_CRAWL_CHILDREN,
            "FT_IFTS_DOCID_INTERFACE_LIST" => WaitType::FT_IFTS_DOCID_INTERFACE_LIST,
            "FT_IFTS_DOCID_LIST" => WaitType::FT_IFTS_DOCID_LIST,
            "FT_IFTS_FP_INFO_LIST" => WaitType::FT_IFTS_FP_INFO_LIST,
            "FT_IFTS_HOST_CONTROLLER" => WaitType::FT_IFTS_HOST_CONTROLLER,
            "FT_IFTS_MASTER_MERGE_TASK_LIST" => WaitType::FT_IFTS_MASTER_MERGE_TASK_LIST,
            "FT_IFTS_MEMREGPOOL" => WaitType::FT_IFTS_MEMREGPOOL,
            "FT_IFTS_MERGE_FRAGMENT_SYNC" => WaitType::FT_IFTS_MERGE_FRAGMENT_SYNC,
            "FT_IFTS_NOISE_WORDS_COLLECTION_CACHE" => WaitType::FT_IFTS_NOISE_WORDS_COLLECTION_CACHE,
            "FT_IFTS_NOISE_WORDS_RESOURCE" => WaitType::FT_IFTS_NOISE_WORDS_RESOURCE,
            "FT_IFTS_OCCURRENCE_BUFFER_POOL" => WaitType::FT_IFTS_OCCURRENCE_BUFFER_POOL,
            "FT_IFTS_PIPELINE" => WaitType::FT_IFTS_PIPELINE,
            "FT_IFTS_PIPELINE_LIST" => WaitType::FT_IFTS_PIPELINE_LIST,
            "FT_IFTS_PIPELINE_MANAGER" => WaitType::FT_IFTS_PIPELINE_MANAGER,
            "FT_IFTS_PROJECT_FD_INFO_MAP" => WaitType::FT_IFTS_PROJECT_FD_INFO_MAP,
            "FT_IFTS_RWLOCK" => WaitType::FT_IFTS_RWLOCK,
            "FT_IFTS_SCHEDULER" => WaitType::FT_IFTS_SCHEDULER,
            "FT_IFTS_SCHEDULER_IDLE_WAIT" => WaitType::FT_IFTS_SCHEDULER_IDLE_WAIT,
            "FT_IFTS_SHARED_MEMORY" => WaitType::FT_IFTS_SHARED_MEMORY,
            "FT_IFTS_SHUTDOWN_PIPE" => WaitType::FT_IFTS_SHUTDOWN_PIPE,
            "FT_IFTS_SRCH_FD_MANAGER" => WaitType::FT_IFTS_SRCH_FD_MANAGER,
            "FT_IFTS_SRCH_FD_SERVICE" => WaitType::FT_IFTS_SRCH_FD_SERVICE,
            "FT_IFTS_STOPLIST_CACHE_MANAGER" => WaitType::FT_IFTS_STOPLIST_CACHE_MANAGER,
            "FT_IFTS_THESAURUS" => WaitType::FT_IFTS_THESAURUS,
            "FT_IFTS_VERSION_MANAGER" => WaitType::FT_IFTS_VERSION_MANAGER,
            "FT_IFTS_WORK_QUEUE" => WaitType::FT_IFTS_WORK_QUEUE,
            "FT_MASTER_MERGE" => WaitType::FT_MASTER_MERGE,
            "FT_MASTER_MERGE_COORDINATOR" => WaitType::FT_MASTER_MERGE_COORDINATOR,
            "FT_METADATA_MUTEX" => WaitType::FT_METADATA_MUTEX,
            "FT_PROPERTYLIST_CACHE" => WaitType::FT_PROPERTYLIST_CACHE,
            "FT_RESTART_CRAWL" => WaitType::FT_RESTART_CRAWL,
            "FULLTEXT GATHERER" => WaitType::FULLTEXT_GATHERER,
            "GDMA_GET_RESOURCE_OWNER" => WaitType::GDMA_GET_RESOURCE_OWNER,
            "GHOSTCLEANUPSYNCMGR" => WaitType::GHOSTCLEANUPSYNCMGR,
            "GHOSTCLEANUP_UPDATE_STATS" => WaitType::GHOSTCLEANUP_UPDATE_STATS,
            "GLOBAL_QUERY_CANCEL" => WaitType::GLOBAL_QUERY_CANCEL,
            "GLOBAL_QUERY_CLOSE" => WaitType::GLOBAL_QUERY_CLOSE,
            "GLOBAL_QUERY_CONSUMER" => WaitType::GLOBAL_QUERY_CONSUMER,
            "GLOBAL_QUERY_PRODUCER" => WaitType::GLOBAL_QUERY_PRODUCER,
            "GLOBAL_TRAN_CREATE" => WaitType::GLOBAL_TRAN_CREATE,
            "GLOBAL_TRAN_UCS_SESSION" => WaitType::GLOBAL_TRAN_UCS_SESSION,
            "GUARDIAN" => WaitType::GUARDIAN,
            "HADR_AG_MUTEX" => WaitType::HADR_AG_MUTEX,
            "HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST" => WaitType::HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST,
            "HADR_AR_CRITICAL_SECTION_ENTRY" => WaitType::HADR_AR_CRITICAL_SECTION_ENTRY,
            "HADR_AR_MANAGER_MUTEX" => WaitType::HADR_AR_MANAGER_MUTEX,
            "HADR_AR_UNLOAD_COMPLETED" => WaitType::HADR_AR_UNLOAD_COMPLETED,
            "HADR_BACKUP_BULK_LOCK" => WaitType::HADR_BACKUP_BULK_LOCK,
            "HADR_BACKUP_QUEUE" => WaitType::HADR_BACKUP_QUEUE,
            "HADR_CLUSAPI_CALL" => WaitType::HADR_CLUSAPI_CALL,
            "HADR_COMPRESSED_CACHE_SYNC" => WaitType::HADR_COMPRESSED_CACHE_SYNC,
            "HADR_CONNECTIVITY_INFO" => WaitType::HADR_CONNECTIVITY_INFO,
            "HADR_DATABASE_FLOW_CONTROL" => WaitType::HADR_DATABASE_FLOW_CONTROL,
            "HADR_DATABASE_VERSIONING_STATE" => WaitType::HADR_DATABASE_VERSIONING_STATE,
            "HADR_DATABASE_WAIT_FOR_RECOVERY" => WaitType::HADR_DATABASE_WAIT_FOR_RECOVERY,
            "HADR_DATABASE_WAIT_FOR_RESTART" => WaitType::HADR_DATABASE_WAIT_FOR_RESTART,
            "HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING" => WaitType::HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING,
            "HADR_DBR_SUBSCRIBER" => WaitType::HADR_DBR_SUBSCRIBER,
            "HADR_DBR_SUBSCRIBER_FILTER_LIST" => WaitType::HADR_DBR_SUBSCRIBER_FILTER_LIST,
            "HADR_DBSEEDING" => WaitType::HADR_DBSEEDING,
            "HADR_DBSEEDING_LIST" => WaitType::HADR_DBSEEDING_LIST,
            "HADR_DBSTATECHANGE_SYNC" => WaitType::HADR_DBSTATECHANGE_SYNC,
            "HADR_DB_COMMAND" => WaitType::HADR_DB_COMMAND,
            "HADR_DB_OP_COMPLETION_SYNC" => WaitType::HADR_DB_OP_COMPLETION_SYNC,
            "HADR_DB_OP_START_SYNC" => WaitType::HADR_DB_OP_START_SYNC,
            "HADR_FABRIC_CALLBACK" => WaitType::HADR_FABRIC_CALLBACK,
            "HADR_FILESTREAM_BLOCK_FLUSH" => WaitType::HADR_FILESTREAM_BLOCK_FLUSH,
            "HADR_FILESTREAM_FILE_CLOSE" => WaitType::HADR_FILESTREAM_FILE_CLOSE,
            "HADR_FILESTREAM_FILE_REQUEST" => WaitType::HADR_FILESTREAM_FILE_REQUEST,
            "HADR_FILESTREAM_IOMGR" => WaitType::HADR_FILESTREAM_IOMGR,
            "HADR_FILESTREAM_IOMGR_IOCOMPLETION" => WaitType::HADR_FILESTREAM_IOMGR_IOCOMPLETION,
            "HADR_FILESTREAM_MANAGER" => WaitType::HADR_FILESTREAM_MANAGER,
            "HADR_FILESTREAM_PREPROC" => WaitType::HADR_FILESTREAM_PREPROC,
            "HADR_GROUP_COMMIT" => WaitType::HADR_GROUP_COMMIT,
            "HADR_LOGCAPTURE_SYNC" => WaitType::HADR_LOGCAPTURE_SYNC,
            "HADR_LOGCAPTURE_WAIT" => WaitType::HADR_LOGCAPTURE_WAIT,
            "HADR_LOGPROGRESS_SYNC" => WaitType::HADR_LOGPROGRESS_SYNC,
            "HADR_NOTIFICATION_DEQUEUE" => WaitType::HADR_NOTIFICATION_DEQUEUE,
            "HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS" => WaitType::HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS,
            "HADR_NOTIFICATION_WORKER_STARTUP_SYNC" => WaitType::HADR_NOTIFICATION_WORKER_STARTUP_SYNC,
            "HADR_NOTIFICATION_WORKER_TERMINATION_SYNC" => WaitType::HADR_NOTIFICATION_WORKER_TERMINATION_SYNC,
            "HADR_PARTNER_SYNC" => WaitType::HADR_PARTNER_SYNC,
            "HADR_READ_ALL_NETWORKS" => WaitType::HADR_READ_ALL_NETWORKS,
            "HADR_RECOVERY_WAIT_FOR_CONNECTION" => WaitType::HADR_RECOVERY_WAIT_FOR_CONNECTION,
            "HADR_RECOVERY_WAIT_FOR_UNDO" => WaitType::HADR_RECOVERY_WAIT_FOR_UNDO,
            "HADR_REPLICAINFO_SYNC" => WaitType::HADR_REPLICAINFO_SYNC,
            "HADR_SEEDING_CANCELLATION" => WaitType::HADR_SEEDING_CANCELLATION,
            "HADR_SEEDING_FILE_LIST" => WaitType::HADR_SEEDING_FILE_LIST,
            "HADR_SEEDING_LIMIT_BACKUPS" => WaitType::HADR_SEEDING_LIMIT_BACKUPS,
            "HADR_SEEDING_SYNC_COMPLETION" => WaitType::HADR_SEEDING_SYNC_COMPLETION,
            "HADR_SEEDING_TIMEOUT_TASK" => WaitType::HADR_SEEDING_TIMEOUT_TASK,
            "HADR_SEEDING_WAIT_FOR_COMPLETION" => WaitType::HADR_SEEDING_WAIT_FOR_COMPLETION,
            "HADR_SYNCHRONIZING_THROTTLE" => WaitType::HADR_SYNCHRONIZING_THROTTLE,
            "HADR_SYNC_COMMIT" => WaitType::HADR_SYNC_COMMIT,
            "HADR_TDS_LISTENER_SYNC" => WaitType::HADR_TDS_LISTENER_SYNC,
            "HADR_TDS_LISTENER_SYNC_PROCESSING" => WaitType::HADR_TDS_LISTENER_SYNC_PROCESSING,
            "HADR_THROTTLE_LOG_RATE_GOVERNOR" => WaitType::HADR_THROTTLE_LOG_RATE_GOVERNOR,
            "HADR_THROTTLE_LOG_RATE_LOG_SIZE" => WaitType::HADR_THROTTLE_LOG_RATE_LOG_SIZE,
            "HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO" => WaitType::HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO,
            "HADR_THROTTLE_LOG_RATE_SEEDING" => WaitType::HADR_THROTTLE_LOG_RATE_SEEDING,
            "HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE" => WaitType::HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE,
            "HADR_TIMER_TASK" => WaitType::HADR_TIMER_TASK,
            "HADR_TRANSPORT_DBRLIST" => WaitType::HADR_TRANSPORT_DBRLIST,
            "HADR_TRANSPORT_FLOW_CONTROL" => WaitType::HADR_TRANSPORT_FLOW_CONTROL,
            "HADR_TRANSPORT_SESSION" => WaitType::HADR_TRANSPORT_SESSION,
            "HADR_WORK_POOL" => WaitType::HADR_WORK_POOL,
            "HADR_WORK_QUEUE" => WaitType::HADR_WORK_QUEUE,
            "HADR_XRF_STACK_ACCESS" => WaitType::HADR_XRF_STACK_ACCESS,
            "HCCO_CACHE" => WaitType::HCCO_CACHE,
            "HKCS_PARALLEL_MIGRATION" => WaitType::HKCS_PARALLEL_MIGRATION,
            "HKCS_PARALLEL_RECOVERY" => WaitType::HKCS_PARALLEL_RECOVERY,
            "HK_RESTORE_FILEMAP" => WaitType::HK_RESTORE_FILEMAP,
            "HTBUILD" => WaitType::HTBUILD,
            "HTBUILD_AGG" => WaitType::HTBUILD_AGG,
            "HTBUILD_JOIN" => WaitType::HTBUILD_JOIN,
            "HTDELETE" => WaitType::HTDELETE,
            "HTDELETE_AGG" => WaitType::HTDELETE_AGG,
            "HTDELETE_JOIN" => WaitType::HTDELETE_JOIN,
            "HTMEMO" => WaitType::HTMEMO,
            "HTREINIT" => WaitType::HTREINIT,
            "HTREPARTITION" => WaitType::HTREPARTITION,
            "HTTP_ENUMERATION" => WaitType::HTTP_ENUMERATION,
            "HTTP_START" => WaitType::HTTP_START,
            "HTTP_STORAGE_CONNECTION" => WaitType::HTTP_STORAGE_CONNECTION,
            "IMPPROV_IOWAIT" => WaitType::IMPPROV_IOWAIT,
            "INSTANCE_LOG_RATE_GOVERNOR" => WaitType::INSTANCE_LOG_RATE_GOVERNOR,
            "INTERNAL_TESTING" => WaitType::INTERNAL_TESTING,
            "IOAFF_RANGE_QUEUE" => WaitType::IOAFF_RANGE_QUEUE,
            "IO_AUDIT_MUTEX" => WaitType::IO_AUDIT_MUTEX,
            "IO_COMPLETION" => WaitType::IO_COMPLETION,
            "IO_QUEUE_LIMIT" => WaitType::IO_QUEUE_LIMIT,
            "IO_RETRY" => WaitType::IO_RETRY,
            "KSOURCE_WAKEUP" => WaitType::KSOURCE_WAKEUP,
            "KTM_ENLISTMENT" => WaitType::KTM_ENLISTMENT,
            "KTM_RECOVERY_MANAGER" => WaitType::KTM_RECOVERY_MANAGER,
            "KTM_RECOVERY_RESOLUTION" => WaitType::KTM_RECOVERY_RESOLUTION,
            "LATCH_DT" => WaitType::LATCH_DT,
            "LATCH_EX" => WaitType::LATCH_EX,
            "LATCH_KP" => WaitType::LATCH_KP,
            "LATCH_NL" => WaitType::LATCH_NL,
            "LATCH_SH" => WaitType::LATCH_SH,
            "LATCH_UP" => WaitType::LATCH_UP,
            "LAZYWRITER_SLEEP" => WaitType::LAZYWRITER_SLEEP,
            "LCK_M_BU" => WaitType::LCK_M_BU,
            "LCK_M_BU_ABORT_BLOCKERS" => WaitType::LCK_M_BU_ABORT_BLOCKERS,
            "LCK_M_BU_LOW_PRIORITY" => WaitType::LCK_M_BU_LOW_PRIORITY,
            "LCK_M_IS" => WaitType::LCK_M_IS,
            "LCK_M_IS_ABORT_BLOCKERS" => WaitType::LCK_M_IS_ABORT_BLOCKERS,
            "LCK_M_IS_LOW_PRIORITY" => WaitType::LCK_M_IS_LOW_PRIORITY,
            "LCK_M_IU" => WaitType::LCK_M_IU,
            "LCK_M_IU_ABORT_BLOCKERS" => WaitType::LCK_M_IU_ABORT_BLOCKERS,
            "LCK_M_IU_LOW_PRIORITY" => WaitType::LCK_M_IU_LOW_PRIORITY,
            "LCK_M_IX" => WaitType::LCK_M_IX,
            "LCK_M_IX_ABORT_BLOCKERS" => WaitType::LCK_M_IX_ABORT_BLOCKERS,
            "LCK_M_IX_LOW_PRIORITY" => WaitType::LCK_M_IX_LOW_PRIORITY,
            "LCK_M_RIn_NL" => WaitType::LCK_M_RIn_NL,
            "LCK_M_RIn_NL_ABORT_BLOCKERS" => WaitType::LCK_M_RIn_NL_ABORT_BLOCKERS,
            "LCK_M_RIn_NL_LOW_PRIORITY" => WaitType::LCK_M_RIn_NL_LOW_PRIORITY,
            "LCK_M_RIn_S" => WaitType::LCK_M_RIn_S,
            "LCK_M_RIn_S_ABORT_BLOCKERS" => WaitType::LCK_M_RIn_S_ABORT_BLOCKERS,
            "LCK_M_RIn_S_LOW_PRIORITY" => WaitType::LCK_M_RIn_S_LOW_PRIORITY,
            "LCK_M_RIn_U" => WaitType::LCK_M_RIn_U,
            "LCK_M_RIn_U_ABORT_BLOCKERS" => WaitType::LCK_M_RIn_U_ABORT_BLOCKERS,
            "LCK_M_RIn_U_LOW_PRIORITY" => WaitType::LCK_M_RIn_U_LOW_PRIORITY,
            "LCK_M_RIn_X" => WaitType::LCK_M_RIn_X,
            "LCK_M_RIn_X_ABORT_BLOCKERS" => WaitType::LCK_M_RIn_X_ABORT_BLOCKERS,
            "LCK_M_RIn_X_LOW_PRIORITY" => WaitType::LCK_M_RIn_X_LOW_PRIORITY,
            "LCK_M_RS_S" => WaitType::LCK_M_RS_S,
            "LCK_M_RS_S_ABORT_BLOCKERS" => WaitType::LCK_M_RS_S_ABORT_BLOCKERS,
            "LCK_M_RS_S_LOW_PRIORITY" => WaitType::LCK_M_RS_S_LOW_PRIORITY,
            "LCK_M_RS_U" => WaitType::LCK_M_RS_U,
            "LCK_M_RS_U_ABORT_BLOCKERS" => WaitType::LCK_M_RS_U_ABORT_BLOCKERS,
            "LCK_M_RS_U_LOW_PRIORITY" => WaitType::LCK_M_RS_U_LOW_PRIORITY,
            "LCK_M_RX_S" => WaitType::LCK_M_RX_S,
            "LCK_M_RX_S_ABORT_BLOCKERS" => WaitType::LCK_M_RX_S_ABORT_BLOCKERS,
            "LCK_M_RX_S_LOW_PRIORITY" => WaitType::LCK_M_RX_S_LOW_PRIORITY,
            "LCK_M_RX_U" => WaitType::LCK_M_RX_U,
            "LCK_M_RX_U_ABORT_BLOCKERS" => WaitType::LCK_M_RX_U_ABORT_BLOCKERS,
            "LCK_M_RX_U_LOW_PRIORITY" => WaitType::LCK_M_RX_U_LOW_PRIORITY,
            "LCK_M_RX_X" => WaitType::LCK_M_RX_X,
            "LCK_M_RX_X_ABORT_BLOCKERS" => WaitType::LCK_M_RX_X_ABORT_BLOCKERS,
            "LCK_M_RX_X_LOW_PRIORITY" => WaitType::LCK_M_RX_X_LOW_PRIORITY,
            "LCK_M_S" => WaitType::LCK_M_S,
            "LCK_M_SCH_M" => WaitType::LCK_M_SCH_M,
            "LCK_M_SCH_M_ABORT_BLOCKERS" => WaitType::LCK_M_SCH_M_ABORT_BLOCKERS,
            "LCK_M_SCH_M_LOW_PRIORITY" => WaitType::LCK_M_SCH_M_LOW_PRIORITY,
            "LCK_M_SCH_S" => WaitType::LCK_M_SCH_S,
            "LCK_M_SCH_S_ABORT_BLOCKERS" => WaitType::LCK_M_SCH_S_ABORT_BLOCKERS,
            "LCK_M_SCH_S_LOW_PRIORITY" => WaitType::LCK_M_SCH_S_LOW_PRIORITY,
            "LCK_M_SIU" => WaitType::LCK_M_SIU,
            "LCK_M_SIU_ABORT_BLOCKERS" => WaitType::LCK_M_SIU_ABORT_BLOCKERS,
            "LCK_M_SIU_LOW_PRIORITY" => WaitType::LCK_M_SIU_LOW_PRIORITY,
            "LCK_M_SIX" => WaitType::LCK_M_SIX,
            "LCK_M_SIX_ABORT_BLOCKERS" => WaitType::LCK_M_SIX_ABORT_BLOCKERS,
            "LCK_M_SIX_LOW_PRIORITY" => WaitType::LCK_M_SIX_LOW_PRIORITY,
            "LCK_M_S_ABORT_BLOCKERS" => WaitType::LCK_M_S_ABORT_BLOCKERS,
            "LCK_M_S_LOW_PRIORITY" => WaitType::LCK_M_S_LOW_PRIORITY,
            "LCK_M_S_XACT" => WaitType::LCK_M_S_XACT,
            "LCK_M_S_XACT_MODIFY" => WaitType::LCK_M_S_XACT_MODIFY,
            "LCK_M_S_XACT_READ" => WaitType::LCK_M_S_XACT_READ,
            "LCK_M_U" => WaitType::LCK_M_U,
            "LCK_M_UIX" => WaitType::LCK_M_UIX,
            "LCK_M_UIX_ABORT_BLOCKERS" => WaitType::LCK_M_UIX_ABORT_BLOCKERS,
            "LCK_M_UIX_LOW_PRIORITY" => WaitType::LCK_M_UIX_LOW_PRIORITY,
            "LCK_M_U_ABORT_BLOCKERS" => WaitType::LCK_M_U_ABORT_BLOCKERS,
            "LCK_M_U_LOW_PRIORITY" => WaitType::LCK_M_U_LOW_PRIORITY,
            "LCK_M_X" => WaitType::LCK_M_X,
            "LCK_M_X_ABORT_BLOCKERS" => WaitType::LCK_M_X_ABORT_BLOCKERS,
            "LCK_M_X_LOW_PRIORITY" => WaitType::LCK_M_X_LOW_PRIORITY,
            "LOGBUFFER" => WaitType::LOGBUFFER,
            "LOGCAPTURE_LOGPOOLTRUNCPOINT" => WaitType::LOGCAPTURE_LOGPOOLTRUNCPOINT,
            "LOGGENERATION" => WaitType::LOGGENERATION,
            "LOGMGR" => WaitType::LOGMGR,
            "LOGMGR_FLUSH" => WaitType::LOGMGR_FLUSH,
            "LOGMGR_PMM_LOG" => WaitType::LOGMGR_PMM_LOG,
            "LOGMGR_QUEUE" => WaitType::LOGMGR_QUEUE,
            "LOGMGR_RESERVE_APPEND" => WaitType::LOGMGR_RESERVE_APPEND,
            "LOGPOOLREFCOUNTEDOBJECT_REFDONE" => WaitType::LOGPOOLREFCOUNTEDOBJECT_REFDONE,
            "LOGPOOL_CACHESIZE" => WaitType::LOGPOOL_CACHESIZE,
            "LOGPOOL_CONSUMER" => WaitType::LOGPOOL_CONSUMER,
            "LOGPOOL_CONSUMERSET" => WaitType::LOGPOOL_CONSUMERSET,
            "LOGPOOL_FREEPOOLS" => WaitType::LOGPOOL_FREEPOOLS,
            "LOGPOOL_MGRSET" => WaitType::LOGPOOL_MGRSET,
            "LOGPOOL_REPLACEMENTSET" => WaitType::LOGPOOL_REPLACEMENTSET,
            "LOG_POOL_SCAN" => WaitType::LOG_POOL_SCAN,
            "LOG_RATE_GOVERNOR" => WaitType::LOG_RATE_GOVERNOR,
            "LOWFAIL_MEMMGR_QUEUE" => WaitType::LOWFAIL_MEMMGR_QUEUE,
            "MD_AGENT_YIELD" => WaitType::MD_AGENT_YIELD,
            "MD_LAZYCACHE_RWLOCK" => WaitType::MD_LAZYCACHE_RWLOCK,
            "MEMORY_ALLOCATION_EXT" => WaitType::MEMORY_ALLOCATION_EXT,
            "MEMORY_GRANT_UPDATE" => WaitType::MEMORY_GRANT_UPDATE,
            "METADATA_LAZYCACHE_RWLOCK" => WaitType::METADATA_LAZYCACHE_RWLOCK,
            "MIGRATIONBUFFER" => WaitType::MIGRATIONBUFFER,
            "MISCELLANEOUS" => WaitType::MISCELLANEOUS,
            "MSQL_DQ" => WaitType::MSQL_DQ,
            "MSQL_XACT_MGR_MUTEX" => WaitType::MSQL_XACT_MGR_MUTEX,
            "MSQL_XACT_MUTEX" => WaitType::MSQL_XACT_MUTEX,
            "MSQL_XP" => WaitType::MSQL_XP,
            "MSSEARCH" => WaitType::MSSEARCH,
            "NETWORKSXMLMGRLOAD" => WaitType::NETWORKSXMLMGRLOAD,
            "NET_WAITFOR_PACKET" => WaitType::NET_WAITFOR_PACKET,
            "NODE_CACHE_MUTEX" => WaitType::NODE_CACHE_MUTEX,
            "OLEDB" => WaitType::OLEDB,
            "ONDEMAND_TASK_QUEUE" => WaitType::ONDEMAND_TASK_QUEUE,
            "PAGEIOLATCH_DT" => WaitType::PAGEIOLATCH_DT,
            "PAGEIOLATCH_EX" => WaitType::PAGEIOLATCH_EX,
            "PAGEIOLATCH_KP" => WaitType::PAGEIOLATCH_KP,
            "PAGEIOLATCH_NL" => WaitType::PAGEIOLATCH_NL,
            "PAGEIOLATCH_SH" => WaitType::PAGEIOLATCH_SH,
            "PAGEIOLATCH_UP" => WaitType::PAGEIOLATCH_UP,
            "PAGELATCH_DT" => WaitType::PAGELATCH_DT,
            "PAGELATCH_EX" => WaitType::PAGELATCH_EX,
            "PAGELATCH_KP" => WaitType::PAGELATCH_KP,
            "PAGELATCH_NL" => WaitType::PAGELATCH_NL,
            "PAGELATCH_SH" => WaitType::PAGELATCH_SH,
            "PAGELATCH_UP" => WaitType::PAGELATCH_UP,
            "PARALLEL_BACKUP_QUEUE" => WaitType::PARALLEL_BACKUP_QUEUE,
            "PARALLEL_REDO_DRAIN_WORKER" => WaitType::PARALLEL_REDO_DRAIN_WORKER,
            "PARALLEL_REDO_FLOW_CONTROL" => WaitType::PARALLEL_REDO_FLOW_CONTROL,
            "PARALLEL_REDO_LOG_CACHE" => WaitType::PARALLEL_REDO_LOG_CACHE,
            "PARALLEL_REDO_TRAN_LIST" => WaitType::PARALLEL_REDO_TRAN_LIST,
            "PARALLEL_REDO_TRAN_TURN" => WaitType::PARALLEL_REDO_TRAN_TURN,
            "PARALLEL_REDO_WORKER_SYNC" => WaitType::PARALLEL_REDO_WORKER_SYNC,
            "PARALLEL_REDO_WORKER_WAIT_WORK" => WaitType::PARALLEL_REDO_WORKER_WAIT_WORK,
            "PERFORMANCE_COUNTERS_RWLOCK" => WaitType::PERFORMANCE_COUNTERS_RWLOCK,
            "PHYSICAL_SEEDING_DMV" => WaitType::PHYSICAL_SEEDING_DMV,
            "POOL_LOG_RATE_GOVERNOR" => WaitType::POOL_LOG_RATE_GOVERNOR,
            "PREEMPTIVE_ABR" => WaitType::PREEMPTIVE_ABR,
            "PREEMPTIVE_AUDIT_ACCESS_EVENTLOG" => WaitType::PREEMPTIVE_AUDIT_ACCESS_EVENTLOG,
            "PREEMPTIVE_AUDIT_ACCESS_SECLOG" => WaitType::PREEMPTIVE_AUDIT_ACCESS_SECLOG,
            "PREEMPTIVE_CLOSEBACKUPMEDIA" => WaitType::PREEMPTIVE_CLOSEBACKUPMEDIA,
            "PREEMPTIVE_CLOSEBACKUPTAPE" => WaitType::PREEMPTIVE_CLOSEBACKUPTAPE,
            "PREEMPTIVE_CLOSEBACKUPVDIDEVICE" => WaitType::PREEMPTIVE_CLOSEBACKUPVDIDEVICE,
            "PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL" => WaitType::PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL,
            "PREEMPTIVE_COM_COCREATEINSTANCE" => WaitType::PREEMPTIVE_COM_COCREATEINSTANCE,
            "PREEMPTIVE_COM_COGETCLASSOBJECT" => WaitType::PREEMPTIVE_COM_COGETCLASSOBJECT,
            "PREEMPTIVE_COM_CREATEACCESSOR" => WaitType::PREEMPTIVE_COM_CREATEACCESSOR,
            "PREEMPTIVE_COM_DELETEROWS" => WaitType::PREEMPTIVE_COM_DELETEROWS,
            "PREEMPTIVE_COM_GETCOMMANDTEXT" => WaitType::PREEMPTIVE_COM_GETCOMMANDTEXT,
            "PREEMPTIVE_COM_GETDATA" => WaitType::PREEMPTIVE_COM_GETDATA,
            "PREEMPTIVE_COM_GETNEXTROWS" => WaitType::PREEMPTIVE_COM_GETNEXTROWS,
            "PREEMPTIVE_COM_GETRESULT" => WaitType::PREEMPTIVE_COM_GETRESULT,
            "PREEMPTIVE_COM_GETROWSBYBOOKMARK" => WaitType::PREEMPTIVE_COM_GETROWSBYBOOKMARK,
            "PREEMPTIVE_COM_LBFLUSH" => WaitType::PREEMPTIVE_COM_LBFLUSH,
            "PREEMPTIVE_COM_LBLOCKREGION" => WaitType::PREEMPTIVE_COM_LBLOCKREGION,
            "PREEMPTIVE_COM_LBREADAT" => WaitType::PREEMPTIVE_COM_LBREADAT,
            "PREEMPTIVE_COM_LBSETSIZE" => WaitType::PREEMPTIVE_COM_LBSETSIZE,
            "PREEMPTIVE_COM_LBSTAT" => WaitType::PREEMPTIVE_COM_LBSTAT,
            "PREEMPTIVE_COM_LBUNLOCKREGION" => WaitType::PREEMPTIVE_COM_LBUNLOCKREGION,
            "PREEMPTIVE_COM_LBWRITEAT" => WaitType::PREEMPTIVE_COM_LBWRITEAT,
            "PREEMPTIVE_COM_QUERYINTERFACE" => WaitType::PREEMPTIVE_COM_QUERYINTERFACE,
            "PREEMPTIVE_COM_RELEASE" => WaitType::PREEMPTIVE_COM_RELEASE,
            "PREEMPTIVE_COM_RELEASEACCESSOR" => WaitType::PREEMPTIVE_COM_RELEASEACCESSOR,
            "PREEMPTIVE_COM_RELEASEROWS" => WaitType::PREEMPTIVE_COM_RELEASEROWS,
            "PREEMPTIVE_COM_RELEASESESSION" => WaitType::PREEMPTIVE_COM_RELEASESESSION,
            "PREEMPTIVE_COM_RESTARTPOSITION" => WaitType::PREEMPTIVE_COM_RESTARTPOSITION,
            "PREEMPTIVE_COM_SEQSTRMREAD" => WaitType::PREEMPTIVE_COM_SEQSTRMREAD,
            "PREEMPTIVE_COM_SEQSTRMREADANDWRITE" => WaitType::PREEMPTIVE_COM_SEQSTRMREADANDWRITE,
            "PREEMPTIVE_COM_SETDATAFAILURE" => WaitType::PREEMPTIVE_COM_SETDATAFAILURE,
            "PREEMPTIVE_COM_SETPARAMETERINFO" => WaitType::PREEMPTIVE_COM_SETPARAMETERINFO,
            "PREEMPTIVE_COM_SETPARAMETERPROPERTIES" => WaitType::PREEMPTIVE_COM_SETPARAMETERPROPERTIES,
            "PREEMPTIVE_COM_STRMLOCKREGION" => WaitType::PREEMPTIVE_COM_STRMLOCKREGION,
            "PREEMPTIVE_COM_STRMSEEKANDREAD" => WaitType::PREEMPTIVE_COM_STRMSEEKANDREAD,
            "PREEMPTIVE_COM_STRMSEEKANDWRITE" => WaitType::PREEMPTIVE_COM_STRMSEEKANDWRITE,
            "PREEMPTIVE_COM_STRMSETSIZE" => WaitType::PREEMPTIVE_COM_STRMSETSIZE,
            "PREEMPTIVE_COM_STRMSTAT" => WaitType::PREEMPTIVE_COM_STRMSTAT,
            "PREEMPTIVE_COM_STRMUNLOCKREGION" => WaitType::PREEMPTIVE_COM_STRMUNLOCKREGION,
            "PREEMPTIVE_CONSOLEWRITE" => WaitType::PREEMPTIVE_CONSOLEWRITE,
            "PREEMPTIVE_CREATEPARAM" => WaitType::PREEMPTIVE_CREATEPARAM,
            "PREEMPTIVE_DEBUG" => WaitType::PREEMPTIVE_DEBUG,
            "PREEMPTIVE_DFSADDLINK" => WaitType::PREEMPTIVE_DFSADDLINK,
            "PREEMPTIVE_DFSLINKEXISTCHECK" => WaitType::PREEMPTIVE_DFSLINKEXISTCHECK,
            "PREEMPTIVE_DFSLINKHEALTHCHECK" => WaitType::PREEMPTIVE_DFSLINKHEALTHCHECK,
            "PREEMPTIVE_DFSREMOVELINK" => WaitType::PREEMPTIVE_DFSREMOVELINK,
            "PREEMPTIVE_DFSREMOVEROOT" => WaitType::PREEMPTIVE_DFSREMOVEROOT,
            "PREEMPTIVE_DFSROOTFOLDERCHECK" => WaitType::PREEMPTIVE_DFSROOTFOLDERCHECK,
            "PREEMPTIVE_DFSROOTINIT" => WaitType::PREEMPTIVE_DFSROOTINIT,
            "PREEMPTIVE_DFSROOTSHARECHECK" => WaitType::PREEMPTIVE_DFSROOTSHARECHECK,
            "PREEMPTIVE_DTC_ABORT" => WaitType::PREEMPTIVE_DTC_ABORT,
            "PREEMPTIVE_DTC_ABORTREQUESTDONE" => WaitType::PREEMPTIVE_DTC_ABORTREQUESTDONE,
            "PREEMPTIVE_DTC_BEGINTRANSACTION" => WaitType::PREEMPTIVE_DTC_BEGINTRANSACTION,
            "PREEMPTIVE_DTC_COMMITREQUESTDONE" => WaitType::PREEMPTIVE_DTC_COMMITREQUESTDONE,
            "PREEMPTIVE_DTC_ENLIST" => WaitType::PREEMPTIVE_DTC_ENLIST,
            "PREEMPTIVE_DTC_PREPAREREQUESTDONE" => WaitType::PREEMPTIVE_DTC_PREPAREREQUESTDONE,
            "PREEMPTIVE_FILESIZEGET" => WaitType::PREEMPTIVE_FILESIZEGET,
            "PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION" => WaitType::PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION,
            "PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION" => WaitType::PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION,
            "PREEMPTIVE_FSAOLEDB_STARTTRANSACTION" => WaitType::PREEMPTIVE_FSAOLEDB_STARTTRANSACTION,
            "PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO" => WaitType::PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO,
            "PREEMPTIVE_GETRMINFO" => WaitType::PREEMPTIVE_GETRMINFO,
            "PREEMPTIVE_HADR_LEASE_MECHANISM" => WaitType::PREEMPTIVE_HADR_LEASE_MECHANISM,
            "PREEMPTIVE_HTTP_EVENT_WAIT" => WaitType::PREEMPTIVE_HTTP_EVENT_WAIT,
            "PREEMPTIVE_HTTP_REQUEST" => WaitType::PREEMPTIVE_HTTP_REQUEST,
            "PREEMPTIVE_LOCKMONITOR" => WaitType::PREEMPTIVE_LOCKMONITOR,
            "PREEMPTIVE_MSS_RELEASE" => WaitType::PREEMPTIVE_MSS_RELEASE,
            "PREEMPTIVE_ODBCOPS" => WaitType::PREEMPTIVE_ODBCOPS,
            "PREEMPTIVE_OLEDBOPS" => WaitType::PREEMPTIVE_OLEDBOPS,
            "PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN" => WaitType::PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN,
            "PREEMPTIVE_OLEDB_ABORTTRAN" => WaitType::PREEMPTIVE_OLEDB_ABORTTRAN,
            "PREEMPTIVE_OLEDB_GETDATASOURCE" => WaitType::PREEMPTIVE_OLEDB_GETDATASOURCE,
            "PREEMPTIVE_OLEDB_GETLITERALINFO" => WaitType::PREEMPTIVE_OLEDB_GETLITERALINFO,
            "PREEMPTIVE_OLEDB_GETPROPERTIES" => WaitType::PREEMPTIVE_OLEDB_GETPROPERTIES,
            "PREEMPTIVE_OLEDB_GETPROPERTYINFO" => WaitType::PREEMPTIVE_OLEDB_GETPROPERTYINFO,
            "PREEMPTIVE_OLEDB_GETSCHEMALOCK" => WaitType::PREEMPTIVE_OLEDB_GETSCHEMALOCK,
            "PREEMPTIVE_OLEDB_JOINTRANSACTION" => WaitType::PREEMPTIVE_OLEDB_JOINTRANSACTION,
            "PREEMPTIVE_OLEDB_RELEASE" => WaitType::PREEMPTIVE_OLEDB_RELEASE,
            "PREEMPTIVE_OLEDB_SETPROPERTIES" => WaitType::PREEMPTIVE_OLEDB_SETPROPERTIES,
            "PREEMPTIVE_OLE_UNINIT" => WaitType::PREEMPTIVE_OLE_UNINIT,
            "PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT" => WaitType::PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT,
            "PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE" => WaitType::PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE,
            "PREEMPTIVE_OS_AUTHENTICATIONOPS" => WaitType::PREEMPTIVE_OS_AUTHENTICATIONOPS,
            "PREEMPTIVE_OS_AUTHORIZATIONOPS" => WaitType::PREEMPTIVE_OS_AUTHORIZATIONOPS,
            "PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT" => WaitType::PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT,
            "PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID" => WaitType::PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID,
            "PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER" => WaitType::PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER,
            "PREEMPTIVE_OS_BACKUPREAD" => WaitType::PREEMPTIVE_OS_BACKUPREAD,
            "PREEMPTIVE_OS_CLOSEHANDLE" => WaitType::PREEMPTIVE_OS_CLOSEHANDLE,
            "PREEMPTIVE_OS_CLUSTEROPS" => WaitType::PREEMPTIVE_OS_CLUSTEROPS,
            "PREEMPTIVE_OS_COMOPS" => WaitType::PREEMPTIVE_OS_COMOPS,
            "PREEMPTIVE_OS_COMPLETEAUTHTOKEN" => WaitType::PREEMPTIVE_OS_COMPLETEAUTHTOKEN,
            "PREEMPTIVE_OS_COPYFILE" => WaitType::PREEMPTIVE_OS_COPYFILE,
            "PREEMPTIVE_OS_CREATEDIRECTORY" => WaitType::PREEMPTIVE_OS_CREATEDIRECTORY,
            "PREEMPTIVE_OS_CREATEFILE" => WaitType::PREEMPTIVE_OS_CREATEFILE,
            "PREEMPTIVE_OS_CRYPTACQUIRECONTEXT" => WaitType::PREEMPTIVE_OS_CRYPTACQUIRECONTEXT,
            "PREEMPTIVE_OS_CRYPTIMPORTKEY" => WaitType::PREEMPTIVE_OS_CRYPTIMPORTKEY,
            "PREEMPTIVE_OS_CRYPTOPS" => WaitType::PREEMPTIVE_OS_CRYPTOPS,
            "PREEMPTIVE_OS_DECRYPTMESSAGE" => WaitType::PREEMPTIVE_OS_DECRYPTMESSAGE,
            "PREEMPTIVE_OS_DELETEFILE" => WaitType::PREEMPTIVE_OS_DELETEFILE,
            "PREEMPTIVE_OS_DELETESECURITYCONTEXT" => WaitType::PREEMPTIVE_OS_DELETESECURITYCONTEXT,
            "PREEMPTIVE_OS_DEVICEIOCONTROL" => WaitType::PREEMPTIVE_OS_DEVICEIOCONTROL,
            "PREEMPTIVE_OS_DEVICEOPS" => WaitType::PREEMPTIVE_OS_DEVICEOPS,
            "PREEMPTIVE_OS_DIRSVC_NETWORKOPS" => WaitType::PREEMPTIVE_OS_DIRSVC_NETWORKOPS,
            "PREEMPTIVE_OS_DISCONNECTNAMEDPIPE" => WaitType::PREEMPTIVE_OS_DISCONNECTNAMEDPIPE,
            "PREEMPTIVE_OS_DOMAINSERVICESOPS" => WaitType::PREEMPTIVE_OS_DOMAINSERVICESOPS,
            "PREEMPTIVE_OS_DSGETDCNAME" => WaitType::PREEMPTIVE_OS_DSGETDCNAME,
            "PREEMPTIVE_OS_DTCOPS" => WaitType::PREEMPTIVE_OS_DTCOPS,
            "PREEMPTIVE_OS_ENCRYPTMESSAGE" => WaitType::PREEMPTIVE_OS_ENCRYPTMESSAGE,
            "PREEMPTIVE_OS_FILEOPS" => WaitType::PREEMPTIVE_OS_FILEOPS,
            "PREEMPTIVE_OS_FINDFILE" => WaitType::PREEMPTIVE_OS_FINDFILE,
            "PREEMPTIVE_OS_FLUSHFILEBUFFERS" => WaitType::PREEMPTIVE_OS_FLUSHFILEBUFFERS,
            "PREEMPTIVE_OS_FORMATMESSAGE" => WaitType::PREEMPTIVE_OS_FORMATMESSAGE,
            "PREEMPTIVE_OS_FREECREDENTIALSHANDLE" => WaitType::PREEMPTIVE_OS_FREECREDENTIALSHANDLE,
            "PREEMPTIVE_OS_FREELIBRARY" => WaitType::PREEMPTIVE_OS_FREELIBRARY,
            "PREEMPTIVE_OS_GENERICOPS" => WaitType::PREEMPTIVE_OS_GENERICOPS,
            "PREEMPTIVE_OS_GETADDRINFO" => WaitType::PREEMPTIVE_OS_GETADDRINFO,
            "PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE" => WaitType::PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE,
            "PREEMPTIVE_OS_GETDISKFREESPACE" => WaitType::PREEMPTIVE_OS_GETDISKFREESPACE,
            "PREEMPTIVE_OS_GETFILEATTRIBUTES" => WaitType::PREEMPTIVE_OS_GETFILEATTRIBUTES,
            "PREEMPTIVE_OS_GETFILESIZE" => WaitType::PREEMPTIVE_OS_GETFILESIZE,
            "PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE" => WaitType::PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE,
            "PREEMPTIVE_OS_GETLONGPATHNAME" => WaitType::PREEMPTIVE_OS_GETLONGPATHNAME,
            "PREEMPTIVE_OS_GETPROCADDRESS" => WaitType::PREEMPTIVE_OS_GETPROCADDRESS,
            "PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT" => WaitType::PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT,
            "PREEMPTIVE_OS_GETVOLUMEPATHNAME" => WaitType::PREEMPTIVE_OS_GETVOLUMEPATHNAME,
            "PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT" => WaitType::PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT,
            "PREEMPTIVE_OS_LIBRARYOPS" => WaitType::PREEMPTIVE_OS_LIBRARYOPS,
            "PREEMPTIVE_OS_LOADLIBRARY" => WaitType::PREEMPTIVE_OS_LOADLIBRARY,
            "PREEMPTIVE_OS_LOGONUSER" => WaitType::PREEMPTIVE_OS_LOGONUSER,
            "PREEMPTIVE_OS_LOOKUPACCOUNTSID" => WaitType::PREEMPTIVE_OS_LOOKUPACCOUNTSID,
            "PREEMPTIVE_OS_MESSAGEQUEUEOPS" => WaitType::PREEMPTIVE_OS_MESSAGEQUEUEOPS,
            "PREEMPTIVE_OS_MOVEFILE" => WaitType::PREEMPTIVE_OS_MOVEFILE,
            "PREEMPTIVE_OS_NETGROUPGETUSERS" => WaitType::PREEMPTIVE_OS_NETGROUPGETUSERS,
            "PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS" => WaitType::PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS,
            "PREEMPTIVE_OS_NETUSERGETGROUPS" => WaitType::PREEMPTIVE_OS_NETUSERGETGROUPS,
            "PREEMPTIVE_OS_NETUSERGETLOCALGROUPS" => WaitType::PREEMPTIVE_OS_NETUSERGETLOCALGROUPS,
            "PREEMPTIVE_OS_NETUSERMODALSGET" => WaitType::PREEMPTIVE_OS_NETUSERMODALSGET,
            "PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY" => WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY,
            "PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE" => WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE,
            "PREEMPTIVE_OS_OPENDIRECTORY" => WaitType::PREEMPTIVE_OS_OPENDIRECTORY,
            "PREEMPTIVE_OS_PDH_WMI_INIT" => WaitType::PREEMPTIVE_OS_PDH_WMI_INIT,
            "PREEMPTIVE_OS_PIPEOPS" => WaitType::PREEMPTIVE_OS_PIPEOPS,
            "PREEMPTIVE_OS_PROCESSOPS" => WaitType::PREEMPTIVE_OS_PROCESSOPS,
            "PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES" => WaitType::PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES,
            "PREEMPTIVE_OS_QUERYREGISTRY" => WaitType::PREEMPTIVE_OS_QUERYREGISTRY,
            "PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN" => WaitType::PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN,
            "PREEMPTIVE_OS_REMOVEDIRECTORY" => WaitType::PREEMPTIVE_OS_REMOVEDIRECTORY,
            "PREEMPTIVE_OS_REPORTEVENT" => WaitType::PREEMPTIVE_OS_REPORTEVENT,
            "PREEMPTIVE_OS_REVERTTOSELF" => WaitType::PREEMPTIVE_OS_REVERTTOSELF,
            "PREEMPTIVE_OS_RSFXDEVICEOPS" => WaitType::PREEMPTIVE_OS_RSFXDEVICEOPS,
            "PREEMPTIVE_OS_SECURITYOPS" => WaitType::PREEMPTIVE_OS_SECURITYOPS,
            "PREEMPTIVE_OS_SERVICEOPS" => WaitType::PREEMPTIVE_OS_SERVICEOPS,
            "PREEMPTIVE_OS_SETENDOFFILE" => WaitType::PREEMPTIVE_OS_SETENDOFFILE,
            "PREEMPTIVE_OS_SETFILEPOINTER" => WaitType::PREEMPTIVE_OS_SETFILEPOINTER,
            "PREEMPTIVE_OS_SETFILEVALIDDATA" => WaitType::PREEMPTIVE_OS_SETFILEVALIDDATA,
            "PREEMPTIVE_OS_SETNAMEDSECURITYINFO" => WaitType::PREEMPTIVE_OS_SETNAMEDSECURITYINFO,
            "PREEMPTIVE_OS_SQLCLROPS" => WaitType::PREEMPTIVE_OS_SQLCLROPS,
            "PREEMPTIVE_OS_SQMLAUNCH" => WaitType::PREEMPTIVE_OS_SQMLAUNCH,
            "PREEMPTIVE_OS_VERIFYSIGNATURE" => WaitType::PREEMPTIVE_OS_VERIFYSIGNATURE,
            "PREEMPTIVE_OS_VERIFYTRUST" => WaitType::PREEMPTIVE_OS_VERIFYTRUST,
            "PREEMPTIVE_OS_VSSOPS" => WaitType::PREEMPTIVE_OS_VSSOPS,
            "PREEMPTIVE_OS_WAITFORSINGLEOBJECT" => WaitType::PREEMPTIVE_OS_WAITFORSINGLEOBJECT,
            "PREEMPTIVE_OS_WINSOCKOPS" => WaitType::PREEMPTIVE_OS_WINSOCKOPS,
            "PREEMPTIVE_OS_WRITEFILE" => WaitType::PREEMPTIVE_OS_WRITEFILE,
            "PREEMPTIVE_OS_WRITEFILEGATHER" => WaitType::PREEMPTIVE_OS_WRITEFILEGATHER,
            "PREEMPTIVE_OS_WSASETLASTERROR" => WaitType::PREEMPTIVE_OS_WSASETLASTERROR,
            "PREEMPTIVE_REENLIST" => WaitType::PREEMPTIVE_REENLIST,
            "PREEMPTIVE_RESIZELOG" => WaitType::PREEMPTIVE_RESIZELOG,
            "PREEMPTIVE_ROLLFORWARDREDO" => WaitType::PREEMPTIVE_ROLLFORWARDREDO,
            "PREEMPTIVE_ROLLFORWARDUNDO" => WaitType::PREEMPTIVE_ROLLFORWARDUNDO,
            "PREEMPTIVE_SB_STOPENDPOINT" => WaitType::PREEMPTIVE_SB_STOPENDPOINT,
            "PREEMPTIVE_SERVER_STARTUP" => WaitType::PREEMPTIVE_SERVER_STARTUP,
            "PREEMPTIVE_SETRMINFO" => WaitType::PREEMPTIVE_SETRMINFO,
            "PREEMPTIVE_SHAREDMEM_GETDATA" => WaitType::PREEMPTIVE_SHAREDMEM_GETDATA,
            "PREEMPTIVE_SNIOPEN" => WaitType::PREEMPTIVE_SNIOPEN,
            "PREEMPTIVE_SOSHOST" => WaitType::PREEMPTIVE_SOSHOST,
            "PREEMPTIVE_SOSTESTING" => WaitType::PREEMPTIVE_SOSTESTING,
            "PREEMPTIVE_SP_SERVER_DIAGNOSTICS" => WaitType::PREEMPTIVE_SP_SERVER_DIAGNOSTICS,
            "PREEMPTIVE_STARTRM" => WaitType::PREEMPTIVE_STARTRM,
            "PREEMPTIVE_STREAMFCB_CHECKPOINT" => WaitType::PREEMPTIVE_STREAMFCB_CHECKPOINT,
            "PREEMPTIVE_STREAMFCB_RECOVER" => WaitType::PREEMPTIVE_STREAMFCB_RECOVER,
            "PREEMPTIVE_STRESSDRIVER" => WaitType::PREEMPTIVE_STRESSDRIVER,
            "PREEMPTIVE_TESTING" => WaitType::PREEMPTIVE_TESTING,
            "PREEMPTIVE_TRANSIMPORT" => WaitType::PREEMPTIVE_TRANSIMPORT,
            "PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN" => WaitType::PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN,
            "PREEMPTIVE_VSS_CREATESNAPSHOT" => WaitType::PREEMPTIVE_VSS_CREATESNAPSHOT,
            "PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT" => WaitType::PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT,
            "PREEMPTIVE_XETESTING" => WaitType::PREEMPTIVE_XETESTING,
            "PREEMPTIVE_XE_CALLBACKEXECUTE" => WaitType::PREEMPTIVE_XE_CALLBACKEXECUTE,
            "PREEMPTIVE_XE_CX_FILE_OPEN" => WaitType::PREEMPTIVE_XE_CX_FILE_OPEN,
            "PREEMPTIVE_XE_CX_HTTP_CALL" => WaitType::PREEMPTIVE_XE_CX_HTTP_CALL,
            "PREEMPTIVE_XE_DISPATCHER" => WaitType::PREEMPTIVE_XE_DISPATCHER,
            "PREEMPTIVE_XE_ENGINEINIT" => WaitType::PREEMPTIVE_XE_ENGINEINIT,
            "PREEMPTIVE_XE_GETTARGETSTATE" => WaitType::PREEMPTIVE_XE_GETTARGETSTATE,
            "PREEMPTIVE_XE_SESSIONCOMMIT" => WaitType::PREEMPTIVE_XE_SESSIONCOMMIT,
            "PREEMPTIVE_XE_TARGETFINALIZE" => WaitType::PREEMPTIVE_XE_TARGETFINALIZE,
            "PREEMPTIVE_XE_TARGETINIT" => WaitType::PREEMPTIVE_XE_TARGETINIT,
            "PREEMPTIVE_XE_TIMERRUN" => WaitType::PREEMPTIVE_XE_TIMERRUN,
            "PRINT_ROLLBACK_PROGRESS" => WaitType::PRINT_ROLLBACK_PROGRESS,
            "PRU_ROLLBACK_DEFERRED" => WaitType::PRU_ROLLBACK_DEFERRED,
            "PVS_CLEANUP_LOCK" => WaitType::PVS_CLEANUP_LOCK,
            "PWAIT_ALL_COMPONENTS_INITIALIZED" => WaitType::PWAIT_ALL_COMPONENTS_INITIALIZED,
            "PWAIT_COOP_SCAN" => WaitType::PWAIT_COOP_SCAN,
            "PWAIT_DIRECTLOGCONSUMER_GETNEXT" => WaitType::PWAIT_DIRECTLOGCONSUMER_GETNEXT,
            "PWAIT_EVENT_SESSION_INIT_MUTEX" => WaitType::PWAIT_EVENT_SESSION_INIT_MUTEX,
            "PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS" => WaitType::PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS,
            "PWAIT_HADRSIM" => WaitType::PWAIT_HADRSIM,
            "PWAIT_HADR_ACTION_COMPLETED" => WaitType::PWAIT_HADR_ACTION_COMPLETED,
            "PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC" => WaitType::PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC,
            "PWAIT_HADR_CLUSTER_INTEGRATION" => WaitType::PWAIT_HADR_CLUSTER_INTEGRATION,
            "PWAIT_HADR_FAILOVER_COMPLETED" => WaitType::PWAIT_HADR_FAILOVER_COMPLETED,
            "PWAIT_HADR_JOIN" => WaitType::PWAIT_HADR_JOIN,
            "PWAIT_HADR_OFFLINE_COMPLETED" => WaitType::PWAIT_HADR_OFFLINE_COMPLETED,
            "PWAIT_HADR_ONLINE_COMPLETED" => WaitType::PWAIT_HADR_ONLINE_COMPLETED,
            "PWAIT_HADR_POST_ONLINE_COMPLETED" => WaitType::PWAIT_HADR_POST_ONLINE_COMPLETED,
            "PWAIT_HADR_SERVER_READY_CONNECTIONS" => WaitType::PWAIT_HADR_SERVER_READY_CONNECTIONS,
            "PWAIT_HADR_WORKITEM_COMPLETED" => WaitType::PWAIT_HADR_WORKITEM_COMPLETED,
            "PWAIT_LOG_CONSOLIDATION_IO" => WaitType::PWAIT_LOG_CONSOLIDATION_IO,
            "PWAIT_LOG_CONSOLIDATION_POLL" => WaitType::PWAIT_LOG_CONSOLIDATION_POLL,
            "PWAIT_MD_LOGIN_STATS" => WaitType::PWAIT_MD_LOGIN_STATS,
            "PWAIT_MD_RELATION_CACHE" => WaitType::PWAIT_MD_RELATION_CACHE,
            "PWAIT_MD_SERVER_CACHE" => WaitType::PWAIT_MD_SERVER_CACHE,
            "PWAIT_MD_UPGRADE_CONFIG" => WaitType::PWAIT_MD_UPGRADE_CONFIG,
            "PWAIT_PREEMPTIVE_APP_USAGE_TIMER" => WaitType::PWAIT_PREEMPTIVE_APP_USAGE_TIMER,
            "PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG" => WaitType::PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG,
            "PWAIT_QRY_BPMEMORY" => WaitType::PWAIT_QRY_BPMEMORY,
            "PWAIT_REPLICA_ONLINE_INIT_MUTEX" => WaitType::PWAIT_REPLICA_ONLINE_INIT_MUTEX,
            "PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC" => WaitType::PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC,
            "PWAIT_SBS_FILE_OPERATION" => WaitType::PWAIT_SBS_FILE_OPERATION,
            "PWAIT_XTP_FSSTORAGE_MAINTENANCE" => WaitType::PWAIT_XTP_FSSTORAGE_MAINTENANCE,
            "PWAIT_XTP_HOST_STORAGE_WAIT" => WaitType::PWAIT_XTP_HOST_STORAGE_WAIT,
            "QDS_ASYNC_CHECK_CONSISTENCY_TASK" => WaitType::QDS_ASYNC_CHECK_CONSISTENCY_TASK,
            "QDS_ASYNC_PERSIST_TASK" => WaitType::QDS_ASYNC_PERSIST_TASK,
            "QDS_ASYNC_PERSIST_TASK_START" => WaitType::QDS_ASYNC_PERSIST_TASK_START,
            "QDS_ASYNC_QUEUE" => WaitType::QDS_ASYNC_QUEUE,
            "QDS_BCKG_TASK" => WaitType::QDS_BCKG_TASK,
            "QDS_BLOOM_FILTER" => WaitType::QDS_BLOOM_FILTER,
            "QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP" => WaitType::QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP,
            "QDS_CTXS" => WaitType::QDS_CTXS,
            "QDS_DB_DISK" => WaitType::QDS_DB_DISK,
            "QDS_DYN_VECTOR" => WaitType::QDS_DYN_VECTOR,
            "QDS_EXCLUSIVE_ACCESS" => WaitType::QDS_EXCLUSIVE_ACCESS,
            "QDS_HOST_INIT" => WaitType::QDS_HOST_INIT,
            "QDS_LOADDB" => WaitType::QDS_LOADDB,
            "QDS_PERSIST_TASK_MAIN_LOOP_SLEEP" => WaitType::QDS_PERSIST_TASK_MAIN_LOOP_SLEEP,
            "QDS_QDS_CAPTURE_INIT" => WaitType::QDS_QDS_CAPTURE_INIT,
            "QDS_SHUTDOWN_QUEUE" => WaitType::QDS_SHUTDOWN_QUEUE,
            "QDS_STMT" => WaitType::QDS_STMT,
            "QDS_STMT_DISK" => WaitType::QDS_STMT_DISK,
            "QDS_TASK_SHUTDOWN" => WaitType::QDS_TASK_SHUTDOWN,
            "QDS_TASK_START" => WaitType::QDS_TASK_START,
            "QE_WARN_LIST_SYNC" => WaitType::QE_WARN_LIST_SYNC,
            "QPJOB_KILL" => WaitType::QPJOB_KILL,
            "QPJOB_WAITFOR_ABORT" => WaitType::QPJOB_WAITFOR_ABORT,
            "QRY_MEM_GRANT_INFO_MUTEX" => WaitType::QRY_MEM_GRANT_INFO_MUTEX,
            "QRY_PARALLEL_THREAD_MUTEX" => WaitType::QRY_PARALLEL_THREAD_MUTEX,
            "QRY_PROFILE_LIST_MUTEX" => WaitType::QRY_PROFILE_LIST_MUTEX,
            "QUERY_ERRHDL_SERVICE_DONE" => WaitType::QUERY_ERRHDL_SERVICE_DONE,
            "QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN" => WaitType::QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN,
            "QUERY_NOTIFICATION_MGR_MUTEX" => WaitType::QUERY_NOTIFICATION_MGR_MUTEX,
            "QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX" => WaitType::QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX,
            "QUERY_NOTIFICATION_TABLE_MGR_MUTEX" => WaitType::QUERY_NOTIFICATION_TABLE_MGR_MUTEX,
            "QUERY_NOTIFICATION_UNITTEST_MUTEX" => WaitType::QUERY_NOTIFICATION_UNITTEST_MUTEX,
            "QUERY_OPTIMIZER_PRINT_MUTEX" => WaitType::QUERY_OPTIMIZER_PRINT_MUTEX,
            "QUERY_TASK_ENQUEUE_MUTEX" => WaitType::QUERY_TASK_ENQUEUE_MUTEX,
            "QUERY_TRACEOUT" => WaitType::QUERY_TRACEOUT,
            "QUERY_WAIT_ERRHDL_SERVICE" => WaitType::QUERY_WAIT_ERRHDL_SERVICE,
            "RBIO_RG_DESTAGE" => WaitType::RBIO_RG_DESTAGE,
            "RBIO_RG_LOCALDESTAGE" => WaitType::RBIO_RG_LOCALDESTAGE,
            "RBIO_RG_REPLICA" => WaitType::RBIO_RG_REPLICA,
            "RBIO_RG_STORAGE" => WaitType::RBIO_RG_STORAGE,
            "RBIO_WAIT_VLF" => WaitType::RBIO_WAIT_VLF,
            "RECOVERY_MGR_LOCK" => WaitType::RECOVERY_MGR_LOCK,
            "RECOVER_CHANGEDB" => WaitType::RECOVER_CHANGEDB,
            "REDO_THREAD_PENDING_WORK" => WaitType::REDO_THREAD_PENDING_WORK,
            "REDO_THREAD_SYNC" => WaitType::REDO_THREAD_SYNC,
            "REMOTE_BLOCK_IO" => WaitType::REMOTE_BLOCK_IO,
            "REMOTE_DATA_ARCHIVE_MIGRATION_DMV" => WaitType::REMOTE_DATA_ARCHIVE_MIGRATION_DMV,
            "REMOTE_DATA_ARCHIVE_SCHEMA_DMV" => WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_DMV,
            "REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE" => WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE,
            "REPLICA_WRITES" => WaitType::REPLICA_WRITES,
            "REPL_CACHE_ACCESS" => WaitType::REPL_CACHE_ACCESS,
            "REPL_HISTORYCACHE_ACCESS" => WaitType::REPL_HISTORYCACHE_ACCESS,
            "REPL_SCHEMA_ACCESS" => WaitType::REPL_SCHEMA_ACCESS,
            "REPL_TRANFSINFO_ACCESS" => WaitType::REPL_TRANFSINFO_ACCESS,
            "REPL_TRANHASHTABLE_ACCESS" => WaitType::REPL_TRANHASHTABLE_ACCESS,
            "REPL_TRANTEXTINFO_ACCESS" => WaitType::REPL_TRANTEXTINFO_ACCESS,
            "REQUEST_DISPENSER_PAUSE" => WaitType::REQUEST_DISPENSER_PAUSE,
            "REQUEST_FOR_DEADLOCK_SEARCH" => WaitType::REQUEST_FOR_DEADLOCK_SEARCH,
            "RESERVED_MEMORY_ALLOCATION_EXT" => WaitType::RESERVED_MEMORY_ALLOCATION_EXT,
            "RESMGR_THROTTLED" => WaitType::RESMGR_THROTTLED,
            "RESOURCE_GOVERNOR_IDLE" => WaitType::RESOURCE_GOVERNOR_IDLE,
            "RESOURCE_QUEUE" => WaitType::RESOURCE_QUEUE,
            "RESOURCE_SEMAPHORE" => WaitType::RESOURCE_SEMAPHORE,
            "RESOURCE_SEMAPHORE_MUTEX" => WaitType::RESOURCE_SEMAPHORE_MUTEX,
            "RESOURCE_SEMAPHORE_QUERY_COMPILE" => WaitType::RESOURCE_SEMAPHORE_QUERY_COMPILE,
            "RESOURCE_SEMAPHORE_SMALL_QUERY" => WaitType::RESOURCE_SEMAPHORE_SMALL_QUERY,
            "RESTORE_FILEHANDLECACHE_ENTRYLOCK" => WaitType::RESTORE_FILEHANDLECACHE_ENTRYLOCK,
            "RESTORE_FILEHANDLECACHE_LOCK" => WaitType::RESTORE_FILEHANDLECACHE_LOCK,
            "RG_RECONFIG" => WaitType::RG_RECONFIG,
            "ROWGROUP_OP_STATS" => WaitType::ROWGROUP_OP_STATS,
            "ROWGROUP_VERSION" => WaitType::ROWGROUP_VERSION,
            "RTDATA_LIST" => WaitType::RTDATA_LIST,
            "SATELLITE_CARGO" => WaitType::SATELLITE_CARGO,
            "SATELLITE_SERVICE_SETUP" => WaitType::SATELLITE_SERVICE_SETUP,
            "SATELLITE_TASK" => WaitType::SATELLITE_TASK,
            "SBS_DISPATCH" => WaitType::SBS_DISPATCH,
            "SBS_RECEIVE_TRANSPORT" => WaitType::SBS_RECEIVE_TRANSPORT,
            "SBS_TRANSPORT" => WaitType::SBS_TRANSPORT,
            "SCAN_CHAR_HASH_ARRAY_INITIALIZATION" => WaitType::SCAN_CHAR_HASH_ARRAY_INITIALIZATION,
            "SECURITY_CNG_PROVIDER_MUTEX" => WaitType::SECURITY_CNG_PROVIDER_MUTEX,
            "SECURITY_CRYPTO_CONTEXT_MUTEX" => WaitType::SECURITY_CRYPTO_CONTEXT_MUTEX,
            "SECURITY_DBE_STATE_MUTEX" => WaitType::SECURITY_DBE_STATE_MUTEX,
            "SECURITY_KEYRING_RWLOCK" => WaitType::SECURITY_KEYRING_RWLOCK,
            "SECURITY_MUTEX" => WaitType::SECURITY_MUTEX,
            "SECURITY_RULETABLE_MUTEX" => WaitType::SECURITY_RULETABLE_MUTEX,
            "SEC_DROP_TEMP_KEY" => WaitType::SEC_DROP_TEMP_KEY,
            "SEMPLAT_DSI_BUILD" => WaitType::SEMPLAT_DSI_BUILD,
            "SEQUENCE_GENERATION" => WaitType::SEQUENCE_GENERATION,
            "SEQUENTIAL_GUID" => WaitType::SEQUENTIAL_GUID,
            "SERVER_IDLE_CHECK" => WaitType::SERVER_IDLE_CHECK,
            "SERVER_RECONFIGURE" => WaitType::SERVER_RECONFIGURE,
            "SESSION_WAIT_STATS_CHILDREN" => WaitType::SESSION_WAIT_STATS_CHILDREN,
            "SHARED_DELTASTORE_CREATION" => WaitType::SHARED_DELTASTORE_CREATION,
            "SHUTDOWN" => WaitType::SHUTDOWN,
            "SLEEP_BPOOL_FLUSH" => WaitType::SLEEP_BPOOL_FLUSH,
            "SLEEP_BUFFERPOOL_HELPLW" => WaitType::SLEEP_BUFFERPOOL_HELPLW,
            "SLEEP_DBSTARTUP" => WaitType::SLEEP_DBSTARTUP,
            "SLEEP_DCOMSTARTUP" => WaitType::SLEEP_DCOMSTARTUP,
            "SLEEP_MASTERDBREADY" => WaitType::SLEEP_MASTERDBREADY,
            "SLEEP_MASTERMDREADY" => WaitType::SLEEP_MASTERMDREADY,
            "SLEEP_MASTERUPGRADED" => WaitType::SLEEP_MASTERUPGRADED,
            "SLEEP_MEMORYPOOL_ALLOCATEPAGES" => WaitType::SLEEP_MEMORYPOOL_ALLOCATEPAGES,
            "SLEEP_MSDBSTARTUP" => WaitType::SLEEP_MSDBSTARTUP,
            "SLEEP_RETRY_VIRTUALALLOC" => WaitType::SLEEP_RETRY_VIRTUALALLOC,
            "SLEEP_SYSTEMTASK" => WaitType::SLEEP_SYSTEMTASK,
            "SLEEP_TASK" => WaitType::SLEEP_TASK,
            "SLEEP_TEMPDBSTARTUP" => WaitType::SLEEP_TEMPDBSTARTUP,
            "SLEEP_WORKSPACE_ALLOCATEPAGE" => WaitType::SLEEP_WORKSPACE_ALLOCATEPAGE,
            "SLO_UPDATE" => WaitType::SLO_UPDATE,
            "SMSYNC" => WaitType::SMSYNC,
            "SNI_CONN_DUP" => WaitType::SNI_CONN_DUP,
            "SNI_CRITICAL_SECTION" => WaitType::SNI_CRITICAL_SECTION,
            "SNI_HTTP_WAITFOR_0_DISCON" => WaitType::SNI_HTTP_WAITFOR_0_DISCON,
            "SNI_LISTENER_ACCESS" => WaitType::SNI_LISTENER_ACCESS,
            "SNI_TASK_COMPLETION" => WaitType::SNI_TASK_COMPLETION,
            "SNI_WRITE_ASYNC" => WaitType::SNI_WRITE_ASYNC,
            "SOAP_READ" => WaitType::SOAP_READ,
            "SOAP_WRITE" => WaitType::SOAP_WRITE,
            "SOCKETDUPLICATEQUEUE_CLEANUP" => WaitType::SOCKETDUPLICATEQUEUE_CLEANUP,
            "SOSHOST_EVENT" => WaitType::SOSHOST_EVENT,
            "SOSHOST_INTERNAL" => WaitType::SOSHOST_INTERNAL,
            "SOSHOST_MUTEX" => WaitType::SOSHOST_MUTEX,
            "SOSHOST_RWLOCK" => WaitType::SOSHOST_RWLOCK,
            "SOSHOST_SEMAPHORE" => WaitType::SOSHOST_SEMAPHORE,
            "SOSHOST_SLEEP" => WaitType::SOSHOST_SLEEP,
            "SOSHOST_TRACELOCK" => WaitType::SOSHOST_TRACELOCK,
            "SOSHOST_WAITFORDONE" => WaitType::SOSHOST_WAITFORDONE,
            "SOS_CALLBACK_REMOVAL" => WaitType::SOS_CALLBACK_REMOVAL,
            "SOS_DISPATCHER_MUTEX" => WaitType::SOS_DISPATCHER_MUTEX,
            "SOS_LOCALALLOCATORLIST" => WaitType::SOS_LOCALALLOCATORLIST,
            "SOS_MEMORY_TOPLEVELBLOCKALLOCATOR" => WaitType::SOS_MEMORY_TOPLEVELBLOCKALLOCATOR,
            "SOS_MEMORY_USAGE_ADJUSTMENT" => WaitType::SOS_MEMORY_USAGE_ADJUSTMENT,
            "SOS_OBJECT_STORE_DESTROY_MUTEX" => WaitType::SOS_OBJECT_STORE_DESTROY_MUTEX,
            "SOS_PHYS_PAGE_CACHE" => WaitType::SOS_PHYS_PAGE_CACHE,
            "SOS_PROCESS_AFFINITY_MUTEX" => WaitType::SOS_PROCESS_AFFINITY_MUTEX,
            "SOS_RESERVEDMEMBLOCKLIST" => WaitType::SOS_RESERVEDMEMBLOCKLIST,
            "SOS_SCHEDULER_YIELD" => WaitType::SOS_SCHEDULER_YIELD,
            "SOS_SMALL_PAGE_ALLOC" => WaitType::SOS_SMALL_PAGE_ALLOC,
            "SOS_STACKSTORE_INIT_MUTEX" => WaitType::SOS_STACKSTORE_INIT_MUTEX,
            "SOS_SYNC_TASK_ENQUEUE_EVENT" => WaitType::SOS_SYNC_TASK_ENQUEUE_EVENT,
            "SOS_VIRTUALMEMORY_LOW" => WaitType::SOS_VIRTUALMEMORY_LOW,
            "SOS_WORK_DISPATCHER" => WaitType::SOS_WORK_DISPATCHER,
            "SPINLOCK_EXT" => WaitType::SPINLOCK_EXT,
            "SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP" => WaitType::SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP,
            "SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS" => WaitType::SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS,
            "SP_SERVER_DIAGNOSTICS_INIT_MUTEX" => WaitType::SP_SERVER_DIAGNOSTICS_INIT_MUTEX,
            "SP_SERVER_DIAGNOSTICS_SLEEP" => WaitType::SP_SERVER_DIAGNOSTICS_SLEEP,
            "SQLCLR_APPDOMAIN" => WaitType::SQLCLR_APPDOMAIN,
            "SQLCLR_ASSEMBLY" => WaitType::SQLCLR_ASSEMBLY,
            "SQLCLR_DEADLOCK_DETECTION" => WaitType::SQLCLR_DEADLOCK_DETECTION,
            "SQLCLR_QUANTUM_PUNISHMENT" => WaitType::SQLCLR_QUANTUM_PUNISHMENT,
            "SQLSORT_NORMMUTEX" => WaitType::SQLSORT_NORMMUTEX,
            "SQLSORT_SORTMUTEX" => WaitType::SQLSORT_SORTMUTEX,
            "SQLTRACE_BUFFER_FLUSH" => WaitType::SQLTRACE_BUFFER_FLUSH,
            "SQLTRACE_FILE_BUFFER" => WaitType::SQLTRACE_FILE_BUFFER,
            "SQLTRACE_FILE_READ_IO_COMPLETION" => WaitType::SQLTRACE_FILE_READ_IO_COMPLETION,
            "SQLTRACE_FILE_WRITE_IO_COMPLETION" => WaitType::SQLTRACE_FILE_WRITE_IO_COMPLETION,
            "SQLTRACE_INCREMENTAL_FLUSH_SLEEP" => WaitType::SQLTRACE_INCREMENTAL_FLUSH_SLEEP,
            "SQLTRACE_LOCK" => WaitType::SQLTRACE_LOCK,
            "SQLTRACE_PENDING_BUFFER_WRITERS" => WaitType::SQLTRACE_PENDING_BUFFER_WRITERS,
            "SQLTRACE_SHUTDOWN" => WaitType::SQLTRACE_SHUTDOWN,
            "SQLTRACE_WAIT_ENTRIES" => WaitType::SQLTRACE_WAIT_ENTRIES,
            "SRVPROC_SHUTDOWN" => WaitType::SRVPROC_SHUTDOWN,
            "STARTUP_DEPENDENCY_MANAGER" => WaitType::STARTUP_DEPENDENCY_MANAGER,
            "TDS_BANDWIDTH_STATE" => WaitType::TDS_BANDWIDTH_STATE,
            "TDS_INIT" => WaitType::TDS_INIT,
            "TDS_PROXY_CONTAINER" => WaitType::TDS_PROXY_CONTAINER,
            "TEMPOBJ" => WaitType::TEMPOBJ,
            "TEMPORAL_BACKGROUND_PROCEED_CLEANUP" => WaitType::TEMPORAL_BACKGROUND_PROCEED_CLEANUP,
            "TERMINATE_LISTENER" => WaitType::TERMINATE_LISTENER,
            "THREADPOOL" => WaitType::THREADPOOL,
            "TIMEPRIV_TIMEPERIOD" => WaitType::TIMEPRIV_TIMEPERIOD,
            "TRACEWRITE" => WaitType::TRACEWRITE,
            "TRACE_EVTNOTIF" => WaitType::TRACE_EVTNOTIF,
            "TRANSACTION_MUTEX" => WaitType::TRANSACTION_MUTEX,
            "TRAN_MARKLATCH_DT" => WaitType::TRAN_MARKLATCH_DT,
            "TRAN_MARKLATCH_EX" => WaitType::TRAN_MARKLATCH_EX,
            "TRAN_MARKLATCH_KP" => WaitType::TRAN_MARKLATCH_KP,
            "TRAN_MARKLATCH_NL" => WaitType::TRAN_MARKLATCH_NL,
            "TRAN_MARKLATCH_SH" => WaitType::TRAN_MARKLATCH_SH,
            "TRAN_MARKLATCH_UP" => WaitType::TRAN_MARKLATCH_UP,
            "UCS_ENDPOINT_CHANGE" => WaitType::UCS_ENDPOINT_CHANGE,
            "UCS_MANAGER" => WaitType::UCS_MANAGER,
            "UCS_MEMORY_NOTIFICATION" => WaitType::UCS_MEMORY_NOTIFICATION,
            "UCS_SESSION_REGISTRATION" => WaitType::UCS_SESSION_REGISTRATION,
            "UCS_TRANSPORT" => WaitType::UCS_TRANSPORT,
            "UCS_TRANSPORT_STREAM_CHANGE" => WaitType::UCS_TRANSPORT_STREAM_CHANGE,
            "UTIL_PAGE_ALLOC" => WaitType::UTIL_PAGE_ALLOC,
            "VDI_CLIENT_COMPLETECOMMAND" => WaitType::VDI_CLIENT_COMPLETECOMMAND,
            "VDI_CLIENT_GETCOMMAND" => WaitType::VDI_CLIENT_GETCOMMAND,
            "VDI_CLIENT_OPERATION" => WaitType::VDI_CLIENT_OPERATION,
            "VDI_CLIENT_OTHER" => WaitType::VDI_CLIENT_OTHER,
            "VERSIONING_COMMITTING" => WaitType::VERSIONING_COMMITTING,
            "VIA_ACCEPT" => WaitType::VIA_ACCEPT,
            "VIEW_DEFINITION_MUTEX" => WaitType::VIEW_DEFINITION_MUTEX,
            "WAITFOR" => WaitType::WAITFOR,
            "WAITFOR_PER_QUEUE" => WaitType::WAITFOR_PER_QUEUE,
            "WAITFOR_TASKSHUTDOWN" => WaitType::WAITFOR_TASKSHUTDOWN,
            "WAITSTAT_MUTEX" => WaitType::WAITSTAT_MUTEX,
            "WAIT_FOR_RESULTS" => WaitType::WAIT_FOR_RESULTS,
            "WAIT_ON_SYNC_STATISTICS_REFRESH" => WaitType::WAIT_ON_SYNC_STATISTICS_REFRESH,
            "WAIT_SCRIPTDEPLOYMENT_REQUEST" => WaitType::WAIT_SCRIPTDEPLOYMENT_REQUEST,
            "WAIT_SCRIPTDEPLOYMENT_WORKER" => WaitType::WAIT_SCRIPTDEPLOYMENT_WORKER,
            "WAIT_XLOGREAD_SIGNAL" => WaitType::WAIT_XLOGREAD_SIGNAL,
            "WAIT_XTP_ASYNC_TX_COMPLETION" => WaitType::WAIT_XTP_ASYNC_TX_COMPLETION,
            "WAIT_XTP_CKPT_AGENT_WAKEUP" => WaitType::WAIT_XTP_CKPT_AGENT_WAKEUP,
            "WAIT_XTP_CKPT_CLOSE" => WaitType::WAIT_XTP_CKPT_CLOSE,
            "WAIT_XTP_CKPT_ENABLED" => WaitType::WAIT_XTP_CKPT_ENABLED,
            "WAIT_XTP_CKPT_STATE_LOCK" => WaitType::WAIT_XTP_CKPT_STATE_LOCK,
            "WAIT_XTP_COMPILE_WAIT" => WaitType::WAIT_XTP_COMPILE_WAIT,
            "WAIT_XTP_GUEST" => WaitType::WAIT_XTP_GUEST,
            "WAIT_XTP_HOST_WAIT" => WaitType::WAIT_XTP_HOST_WAIT,
            "WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO" => WaitType::WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO,
            "WAIT_XTP_OFFLINE_CKPT_LOG_IO" => WaitType::WAIT_XTP_OFFLINE_CKPT_LOG_IO,
            "WAIT_XTP_OFFLINE_CKPT_NEW_LOG" => WaitType::WAIT_XTP_OFFLINE_CKPT_NEW_LOG,
            "WAIT_XTP_PROCEDURE_ENTRY" => WaitType::WAIT_XTP_PROCEDURE_ENTRY,
            "WAIT_XTP_RECOVERY" => WaitType::WAIT_XTP_RECOVERY,
            "WAIT_XTP_SERIAL_RECOVERY" => WaitType::WAIT_XTP_SERIAL_RECOVERY,
            "WAIT_XTP_SWITCH_TO_INACTIVE" => WaitType::WAIT_XTP_SWITCH_TO_INACTIVE,
            "WAIT_XTP_TASK_SHUTDOWN" => WaitType::WAIT_XTP_TASK_SHUTDOWN,
            "WAIT_XTP_TRAN_DEPENDENCY" => WaitType::WAIT_XTP_TRAN_DEPENDENCY,
            "WCC" => WaitType::WCC,
            "WINDOW_AGGREGATES_MULTIPASS" => WaitType::WINDOW_AGGREGATES_MULTIPASS,
            "WINFAB_API_CALL" => WaitType::WINFAB_API_CALL,
            "WINFAB_REPLICA_BUILD_OPERATION" => WaitType::WINFAB_REPLICA_BUILD_OPERATION,
            "WINFAB_REPORT_FAULT" => WaitType::WINFAB_REPORT_FAULT,
            "WORKTBL_DROP" => WaitType::WORKTBL_DROP,
            "WRITELOG" => WaitType::WRITELOG,
            "WRITE_COMPLETION" => WaitType::WRITE_COMPLETION,
            "XACTLOCKINFO" => WaitType::XACTLOCKINFO,
            "XACTWORKSPACE_MUTEX" => WaitType::XACTWORKSPACE_MUTEX,
            "XACT_OWN_TRANSACTION" => WaitType::XACT_OWN_TRANSACTION,
            "XACT_RECLAIM_SESSION" => WaitType::XACT_RECLAIM_SESSION,
            "XDB_CONN_DUP_HASH" => WaitType::XDB_CONN_DUP_HASH,
            "XDESTSVERMGR" => WaitType::XDESTSVERMGR,
            "XDES_HISTORY" => WaitType::XDES_HISTORY,
            "XDES_OUT_OF_ORDER_LIST" => WaitType::XDES_OUT_OF_ORDER_LIST,
            "XDES_SNAPSHOT" => WaitType::XDES_SNAPSHOT,
            "XE_BUFFERMGR_ALLPROCESSED_EVENT" => WaitType::XE_BUFFERMGR_ALLPROCESSED_EVENT,
            "XE_BUFFERMGR_FREEBUF_EVENT" => WaitType::XE_BUFFERMGR_FREEBUF_EVENT,
            "XE_CALLBACK_LIST" => WaitType::XE_CALLBACK_LIST,
            "XE_CX_FILE_READ" => WaitType::XE_CX_FILE_READ,
            "XE_DISPATCHER_CONFIG_SESSION_LIST" => WaitType::XE_DISPATCHER_CONFIG_SESSION_LIST,
            "XE_DISPATCHER_JOIN" => WaitType::XE_DISPATCHER_JOIN,
            "XE_DISPATCHER_WAIT" => WaitType::XE_DISPATCHER_WAIT,
            "XE_FILE_TARGET_TVF" => WaitType::XE_FILE_TARGET_TVF,
            "XE_LIVE_TARGET_TVF" => WaitType::XE_LIVE_TARGET_TVF,
            "XE_MODULEMGR_SYNC" => WaitType::XE_MODULEMGR_SYNC,
            "XE_OLS_LOCK" => WaitType::XE_OLS_LOCK,
            "XE_PACKAGE_LOCK_BACKOFF" => WaitType::XE_PACKAGE_LOCK_BACKOFF,
            "XE_SERVICES_EVENTMANUAL" => WaitType::XE_SERVICES_EVENTMANUAL,
            "XE_SERVICES_MUTEX" => WaitType::XE_SERVICES_MUTEX,
            "XE_SERVICES_RWLOCK" => WaitType::XE_SERVICES_RWLOCK,
            "XE_SESSION_CREATE_SYNC" => WaitType::XE_SESSION_CREATE_SYNC,
            "XE_SESSION_FLUSH" => WaitType::XE_SESSION_FLUSH,
            "XE_SESSION_SYNC" => WaitType::XE_SESSION_SYNC,
            "XE_STM_CREATE" => WaitType::XE_STM_CREATE,
            "XE_TIMER_EVENT" => WaitType::XE_TIMER_EVENT,
            "XE_TIMER_MUTEX" => WaitType::XE_TIMER_MUTEX,
            "XE_TIMER_TASK_DONE" => WaitType::XE_TIMER_TASK_DONE,
            "XIO_CREDENTIAL_MGR_RWLOCK" => WaitType::XIO_CREDENTIAL_MGR_RWLOCK,
            "XIO_CREDENTIAL_RWLOCK" => WaitType::XIO_CREDENTIAL_RWLOCK,
            "XIO_EDS_MGR_RWLOCK" => WaitType::XIO_EDS_MGR_RWLOCK,
            "XIO_EDS_RWLOCK" => WaitType::XIO_EDS_RWLOCK,
            "XIO_IOSTATS_BLOBLIST_RWLOCK" => WaitType::XIO_IOSTATS_BLOBLIST_RWLOCK,
            "XIO_IOSTATS_FCBLIST_RWLOCK" => WaitType::XIO_IOSTATS_FCBLIST_RWLOCK,
            "XIO_LEASE_RENEW_MGR_RWLOCK" => WaitType::XIO_LEASE_RENEW_MGR_RWLOCK,
            "XTPPROC_CACHE_ACCESS" => WaitType::XTPPROC_CACHE_ACCESS,
            "XTPPROC_PARTITIONED_STACK_CREATE" => WaitType::XTPPROC_PARTITIONED_STACK_CREATE,
            "XTP_HOST_DB_COLLECTION" => WaitType::XTP_HOST_DB_COLLECTION,
            "XTP_HOST_LOG_ACTIVITY" => WaitType::XTP_HOST_LOG_ACTIVITY,
            "XTP_HOST_PARALLEL_RECOVERY" => WaitType::XTP_HOST_PARALLEL_RECOVERY,
            "XTP_PREEMPTIVE_TASK" => WaitType::XTP_PREEMPTIVE_TASK,
            "XTP_TRUNCATION_LSN" => WaitType::XTP_TRUNCATION_LSN,
            other => WaitType::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            WaitType::ABR => "ABR",
            WaitType::AM_INDBUILD_ALLOCATION => "AM_INDBUILD_ALLOCATION",
            WaitType::AM_SCHEMAMGR_UNSHARED_CACHE => "AM_SCHEMAMGR_UNSHARED_CACHE",
            WaitType::ASSEMBLY_FILTER_HASHTABLE => "ASSEMBLY_FILTER_HASHTABLE",
            WaitType::ASSEMBLY_LOAD => "ASSEMBLY_LOAD",
            WaitType::ASYNC_DISKPOOL_LOCK => "ASYNC_DISKPOOL_LOCK",
            WaitType::ASYNC_IO_COMPLETION => "ASYNC_IO_COMPLETION",
            WaitType::ASYNC_NETWORK_IO => "ASYNC_NETWORK_IO",
            WaitType::ASYNC_OP_COMPLETION => "ASYNC_OP_COMPLETION",
            WaitType::ASYNC_OP_CONTEXT_READ => "ASYNC_OP_CONTEXT_READ",
            WaitType::ASYNC_OP_CONTEXT_WRITE => "ASYNC_OP_CONTEXT_WRITE",
            WaitType::ASYNC_SOCKETDUP_IO => "ASYNC_SOCKETDUP_IO",
            WaitType::AUDIT_GROUPCACHE_LOCK => "AUDIT_GROUPCACHE_LOCK",
            WaitType::AUDIT_LOGINCACHE_LOCK => "AUDIT_LOGINCACHE_LOCK",
            WaitType::AUDIT_ON_DEMAND_TARGET_LOCK => "AUDIT_ON_DEMAND_TARGET_LOCK",
            WaitType::AUDIT_XE_SESSION_MGR => "AUDIT_XE_SESSION_MGR",
            WaitType::BACKUP => "BACKUP",
            WaitType::BACKUPBUFFER => "BACKUPBUFFER",
            WaitType::BACKUPIO => "BACKUPIO",
            WaitType::BACKUPTHREAD => "BACKUPTHREAD",
            WaitType::BACKUP_OPERATOR => "BACKUP_OPERATOR",
            WaitType::BAD_PAGE_PROCESS => "BAD_PAGE_PROCESS",
            WaitType::BLOB_METADATA => "BLOB_METADATA",
            WaitType::BMPALLOCATION => "BMPALLOCATION",
            WaitType::BMPBUILD => "BMPBUILD",
            WaitType::BMPREPARTITION => "BMPREPARTITION",
            WaitType::BMPREPLICATION => "BMPREPLICATION",
            WaitType::BPSORT => "BPSORT",
            WaitType::BROKER_CONNECTION_RECEIVE_TASK => "BROKER_CONNECTION_RECEIVE_TASK",
            WaitType::BROKER_DISPATCHER => "BROKER_DISPATCHER",
            WaitType::BROKER_ENDPOINT_STATE_MUTEX => "BROKER_ENDPOINT_STATE_MUTEX",
            WaitType::BROKER_EVENTHANDLER => "BROKER_EVENTHANDLER",
            WaitType::BROKER_FORWARDER => "BROKER_FORWARDER",
            WaitType::BROKER_INIT => "BROKER_INIT",
            WaitType::BROKER_MASTERSTART => "BROKER_MASTERSTART",
            WaitType::BROKER_RECEIVE_WAITFOR => "BROKER_RECEIVE_WAITFOR",
            WaitType::BROKER_REGISTERALLENDPOINTS => "BROKER_REGISTERALLENDPOINTS",
            WaitType::BROKER_SERVICE => "BROKER_SERVICE",
            WaitType::BROKER_SHUTDOWN => "BROKER_SHUTDOWN",
            WaitType::BROKER_START => "BROKER_START",
            WaitType::BROKER_TASK_SHUTDOWN => "BROKER_TASK_SHUTDOWN",
            WaitType::BROKER_TASK_STOP => "BROKER_TASK_STOP",
            WaitType::BROKER_TASK_SUBMIT => "BROKER_TASK_SUBMIT",
            WaitType::BROKER_TO_FLUSH => "BROKER_TO_FLUSH",
            WaitType::BROKER_TRANSMISSION_OBJECT => "BROKER_TRANSMISSION_OBJECT",
            WaitType::BROKER_TRANSMISSION_TABLE => "BROKER_TRANSMISSION_TABLE",
            WaitType::BROKER_TRANSMISSION_WORK => "BROKER_TRANSMISSION_WORK",
            WaitType::BROKER_TRANSMITTER => "BROKER_TRANSMITTER",
            WaitType::BUFFERPOOL_SCAN => "BUFFERPOOL_SCAN",
            WaitType::BUILTIN_HASHKEY_MUTEX => "BUILTIN_HASHKEY_MUTEX",
            WaitType::CHANGE_TRACKING_WAITFORCHANGES => "CHANGE_TRACKING_WAITFORCHANGES",
            WaitType::CHECKPOINT_QUEUE => "CHECKPOINT_QUEUE",
            WaitType::CHECK_PRINT_RECORD => "CHECK_PRINT_RECORD",
            WaitType::CHECK_SCANNER_MUTEX => "CHECK_SCANNER_MUTEX",
            WaitType::CHECK_TABLES_INITIALIZATION => "CHECK_TABLES_INITIALIZATION",
            WaitType::CHECK_TABLES_SINGLE_SCAN => "CHECK_TABLES_SINGLE_SCAN",
            WaitType::CHECK_TABLES_THREAD_BARRIER => "CHECK_TABLES_THREAD_BARRIER",
            WaitType::CHKPT => "CHKPT",
            WaitType::CLEAR_DB => "CLEAR_DB",
            WaitType::CLRHOST_STATE_ACCESS => "CLRHOST_STATE_ACCESS",
            WaitType::CLR_AUTO_EVENT => "CLR_AUTO_EVENT",
            WaitType::CLR_CRST => "CLR_CRST",
            WaitType::CLR_JOIN => "CLR_JOIN",
            WaitType::CLR_MANUAL_EVENT => "CLR_MANUAL_EVENT",
            WaitType::CLR_MEMORY_SPY => "CLR_MEMORY_SPY",
            WaitType::CLR_MONITOR => "CLR_MONITOR",
            WaitType::CLR_RWLOCK_READER => "CLR_RWLOCK_READER",
            WaitType::CLR_RWLOCK_WRITER => "CLR_RWLOCK_WRITER",
            WaitType::CLR_SEMAPHORE => "CLR_SEMAPHORE",
            WaitType::CLR_TASK_START => "CLR_TASK_START",
            WaitType::CMEMPARTITIONED => "CMEMPARTITIONED",
            WaitType::CMEMTHREAD => "CMEMTHREAD",
            WaitType::COLUMNSTORE_BUILD_THROTTLE => "COLUMNSTORE_BUILD_THROTTLE",
            WaitType::COLUMNSTORE_COLUMNDATASET_SESSION_LIST => "COLUMNSTORE_COLUMNDATASET_SESSION_LIST",
            WaitType::COMMIT_TABLE => "COMMIT_TABLE",
            WaitType::CONNECTION_ENDPOINT_LOCK => "CONNECTION_ENDPOINT_LOCK",
            WaitType::COUNTRECOVERYMGR => "COUNTRECOVERYMGR",
            WaitType::CREATE_DATINISERVICE => "CREATE_DATINISERVICE",
            WaitType::CXCONSUMER => "CXCONSUMER",
            WaitType::CXPACKET => "CXPACKET",
            WaitType::CXROWSET_SYNC => "CXROWSET_SYNC",
            WaitType::CXSYNC_CONSUMER => "CXSYNC_CONSUMER",
            WaitType::CXSYNC_PORT => "CXSYNC_PORT",
            WaitType::DAC_INIT => "DAC_INIT",
            WaitType::DBCC_SCALE_OUT_EXPR_CACHE => "DBCC_SCALE_OUT_EXPR_CACHE",
            WaitType::DBMIRRORING_CMD => "DBMIRRORING_CMD",
            WaitType::DBMIRROR_DBM_EVENT => "DBMIRROR_DBM_EVENT",
            WaitType::DBMIRROR_DBM_MUTEX => "DBMIRROR_DBM_MUTEX",
            WaitType::DBMIRROR_EVENTS_QUEUE => "DBMIRROR_EVENTS_QUEUE",
            WaitType::DBMIRROR_SEND => "DBMIRROR_SEND",
            WaitType::DBMIRROR_WORKER_QUEUE => "DBMIRROR_WORKER_QUEUE",
            WaitType::DBSEEDING_FLOWCONTROL => "DBSEEDING_FLOWCONTROL",
            WaitType::DBSEEDING_OPERATION => "DBSEEDING_OPERATION",
            WaitType::DEADLOCK_ENUM_MUTEX => "DEADLOCK_ENUM_MUTEX",
            WaitType::DEADLOCK_TASK_SEARCH => "DEADLOCK_TASK_SEARCH",
            WaitType::DEBUG => "DEBUG",
            WaitType::DIRECTLOGCONSUMER_LIST => "DIRECTLOGCONSUMER_LIST",
            WaitType::DIRTY_PAGE_POLL => "DIRTY_PAGE_POLL",
            WaitType::DIRTY_PAGE_SYNC => "DIRTY_PAGE_SYNC",
            WaitType::DIRTY_PAGE_TABLE_LOCK => "DIRTY_PAGE_TABLE_LOCK",
            WaitType::DISABLE_VERSIONING => "DISABLE_VERSIONING",
            WaitType::DISKIO_SUSPEND => "DISKIO_SUSPEND",
            WaitType::DISPATCHER_PRIORITY_QUEUE_SEMAPHORE => "DISPATCHER_PRIORITY_QUEUE_SEMAPHORE",
            WaitType::DISPATCHER_QUEUE_SEMAPHORE => "DISPATCHER_QUEUE_SEMAPHORE",
            WaitType::DLL_LOADING_MUTEX => "DLL_LOADING_MUTEX",
            WaitType::DPT_ENTRY_LOCK => "DPT_ENTRY_LOCK",
            WaitType::DROPTEMP => "DROPTEMP",
            WaitType::DROP_DATABASE_TIMER_TASK => "DROP_DATABASE_TIMER_TASK",
            WaitType::DTC => "DTC",
            WaitType::DTCNEW_ENLIST => "DTCNEW_ENLIST",
            WaitType::DTCNEW_PREPARE => "DTCNEW_PREPARE",
            WaitType::DTCNEW_RECOVERY => "DTCNEW_RECOVERY",
            WaitType::DTCNEW_TM => "DTCNEW_TM",
            WaitType::DTCNEW_TRANSACTION_ENLISTMENT => "DTCNEW_TRANSACTION_ENLISTMENT",
            WaitType::DTCPNTSYNC => "DTCPNTSYNC",
            WaitType::DTC_ABORT_REQUEST => "DTC_ABORT_REQUEST",
            WaitType::DTC_RESOLVE => "DTC_RESOLVE",
            WaitType::DTC_STATE => "DTC_STATE",
            WaitType::DTC_TMDOWN_REQUEST => "DTC_TMDOWN_REQUEST",
            WaitType::DTC_WAITFOR_OUTCOME => "DTC_WAITFOR_OUTCOME",
            WaitType::DUMPTRIGGER => "DUMPTRIGGER",
            WaitType::DUMP_LOG_COORDINATOR => "DUMP_LOG_COORDINATOR",
            WaitType::DUMP_LOG_COORDINATOR_QUEUE => "DUMP_LOG_COORDINATOR_QUEUE",
            WaitType::EC => "EC",
            WaitType::EE_PMOLOCK => "EE_PMOLOCK",
            WaitType::EE_SPECPROC_MAP_INIT => "EE_SPECPROC_MAP_INIT",
            WaitType::ENABLE_EMPTY_VERSIONING => "ENABLE_EMPTY_VERSIONING",
            WaitType::ENABLE_VERSIONING => "ENABLE_VERSIONING",
            WaitType::ERROR_REPORTING_MANAGER => "ERROR_REPORTING_MANAGER",
            WaitType::EXCHANGE => "EXCHANGE",
            WaitType::EXECSYNC => "EXECSYNC",
            WaitType::EXECUTION_PIPE_EVENT_INTERNAL => "EXECUTION_PIPE_EVENT_INTERNAL",
            WaitType::EXTERNAL_RG_UPDATE => "EXTERNAL_RG_UPDATE",
            WaitType::EXTERNAL_SCRIPT_NETWORK_IO => "EXTERNAL_SCRIPT_NETWORK_IO",
            WaitType::EXTERNAL_SCRIPT_PREPARE_SERVICE => "EXTERNAL_SCRIPT_PREPARE_SERVICE",
            WaitType::EXTERNAL_SCRIPT_SHUTDOWN => "EXTERNAL_SCRIPT_SHUTDOWN",
            WaitType::EXTERNAL_WAIT_ON_LAUNCHER => "EXTERNAL_WAIT_ON_LAUNCHER",
            WaitType::FABRIC_HADR_TRANSPORT_CONNECTION => "FABRIC_HADR_TRANSPORT_CONNECTION",
            WaitType::FABRIC_REPLICA_CONTROLLER_LIST => "FABRIC_REPLICA_CONTROLLER_LIST",
            WaitType::FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG => "FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG",
            WaitType::FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH => "FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH",
            WaitType::FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST => "FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST",
            WaitType::FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING => "FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING",
            WaitType::FAILPOINT => "FAILPOINT",
            WaitType::FCB_REPLICA_READ => "FCB_REPLICA_READ",
            WaitType::FCB_REPLICA_WRITE => "FCB_REPLICA_WRITE",
            WaitType::FEATURE_SWITCHES_UPDATE => "FEATURE_SWITCHES_UPDATE",
            WaitType::FFT_NSO_DB_KILL_FLAG => "FFT_NSO_DB_KILL_FLAG",
            WaitType::FFT_NSO_DB_LIST => "FFT_NSO_DB_LIST",
            WaitType::FFT_NSO_FCB => "FFT_NSO_FCB",
            WaitType::FFT_NSO_FCB_FIND => "FFT_NSO_FCB_FIND",
            WaitType::FFT_NSO_FCB_PARENT => "FFT_NSO_FCB_PARENT",
            WaitType::FFT_NSO_FCB_RELEASE_CACHED_ENTRIES => "FFT_NSO_FCB_RELEASE_CACHED_ENTRIES",
            WaitType::FFT_NSO_FCB_STATE => "FFT_NSO_FCB_STATE",
            WaitType::FFT_NSO_FILEOBJECT => "FFT_NSO_FILEOBJECT",
            WaitType::FFT_NSO_TABLE_LIST => "FFT_NSO_TABLE_LIST",
            WaitType::FFT_NTFS_STORE => "FFT_NTFS_STORE",
            WaitType::FFT_RECOVERY => "FFT_RECOVERY",
            WaitType::FFT_RSFX_COMM => "FFT_RSFX_COMM",
            WaitType::FFT_RSFX_WAIT_FOR_MEMORY => "FFT_RSFX_WAIT_FOR_MEMORY",
            WaitType::FFT_STARTUP_SHUTDOWN => "FFT_STARTUP_SHUTDOWN",
            WaitType::FFT_STORE_DB => "FFT_STORE_DB",
            WaitType::FFT_STORE_ROWSET_LIST => "FFT_STORE_ROWSET_LIST",
            WaitType::FFT_STORE_TABLE => "FFT_STORE_TABLE",
            WaitType::FILESTREAM_CACHE => "FILESTREAM_CACHE",
            WaitType::FILESTREAM_CHUNKER => "FILESTREAM_CHUNKER",
            WaitType::FILESTREAM_CHUNKER_INIT => "FILESTREAM_CHUNKER_INIT",
            WaitType::FILESTREAM_FCB => "FILESTREAM_FCB",
            WaitType::FILESTREAM_FILE_OBJECT => "FILESTREAM_FILE_OBJECT",
            WaitType::FILESTREAM_WORKITEM_QUEUE => "FILESTREAM_WORKITEM_QUEUE",
            WaitType::FILETABLE_SHUTDOWN => "FILETABLE_SHUTDOWN",
            WaitType::FILE_VALIDATION_THREADS => "FILE_VALIDATION_THREADS",
            WaitType::FOREIGN_REDO => "FOREIGN_REDO",
            WaitType::FORWARDER_TRANSITION => "FORWARDER_TRANSITION",
            WaitType::FSAGENT => "FSAGENT",
            WaitType::FSA_FORCE_OWN_XACT => "FSA_FORCE_OWN_XACT",
            WaitType::FSTR_CONFIG_MUTEX => "FSTR_CONFIG_MUTEX",
            WaitType::FSTR_CONFIG_RWLOCK => "FSTR_CONFIG_RWLOCK",
            WaitType::FS_FC_RWLOCK => "FS_FC_RWLOCK",
            WaitType::FS_GARBAGE_COLLECTOR_SHUTDOWN => "FS_GARBAGE_COLLECTOR_SHUTDOWN",
            WaitType::FS_HEADER_RWLOCK => "FS_HEADER_RWLOCK",
            WaitType::FS_LOGTRUNC_RWLOCK => "FS_LOGTRUNC_RWLOCK",
            WaitType::FT_COMPROWSET_RWLOCK => "FT_COMPROWSET_RWLOCK",
            WaitType::FT_IFTSHC_MUTEX => "FT_IFTSHC_MUTEX",
            WaitType::FT_IFTSISM_MUTEX => "FT_IFTSISM_MUTEX",
            WaitType::FT_IFTS_ASYNC_WRITE_PIPE => "FT_IFTS_ASYNC_WRITE_PIPE",
            WaitType::FT_IFTS_BLOB_HASH => "FT_IFTS_BLOB_HASH",
            WaitType::FT_IFTS_CATEALOG_SOURCE => "FT_IFTS_CATEALOG_SOURCE",
            WaitType::FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER => "FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER",
            WaitType::FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST => "FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST",
            WaitType::FT_IFTS_COMP_DESC_MANAGER => "FT_IFTS_COMP_DESC_MANAGER",
            WaitType::FT_IFTS_CONSUMER_PLUGIN => "FT_IFTS_CONSUMER_PLUGIN",
            WaitType::FT_IFTS_CRAWL_BATCH_LIST => "FT_IFTS_CRAWL_BATCH_LIST",
            WaitType::FT_IFTS_CRAWL_CHILDREN => "FT_IFTS_CRAWL_CHILDREN",
            WaitType::FT_IFTS_DOCID_INTERFACE_LIST => "FT_IFTS_DOCID_INTERFACE_LIST",
            WaitType::FT_IFTS_DOCID_LIST => "FT_IFTS_DOCID_LIST",
            WaitType::FT_IFTS_FP_INFO_LIST => "FT_IFTS_FP_INFO_LIST",
            WaitType::FT_IFTS_HOST_CONTROLLER => "FT_IFTS_HOST_CONTROLLER",
            WaitType::FT_IFTS_MASTER_MERGE_TASK_LIST => "FT_IFTS_MASTER_MERGE_TASK_LIST",
            WaitType::FT_IFTS_MEMREGPOOL => "FT_IFTS_MEMREGPOOL",
            WaitType::FT_IFTS_MERGE_FRAGMENT_SYNC => "FT_IFTS_MERGE_FRAGMENT_SYNC",
            WaitType::FT_IFTS_NOISE_WORDS_COLLECTION_CACHE => "FT_IFTS_NOISE_WORDS_COLLECTION_CACHE",
            WaitType::FT_IFTS_NOISE_WORDS_RESOURCE => "FT_IFTS_NOISE_WORDS_RESOURCE",
            WaitType::FT_IFTS_OCCURRENCE_BUFFER_POOL => "FT_IFTS_OCCURRENCE_BUFFER_POOL",
            WaitType::FT_IFTS_PIPELINE => "FT_IFTS_PIPELINE",
            WaitType::FT_IFTS_PIPELINE_LIST => "FT_IFTS_PIPELINE_LIST",
            WaitType::FT_IFTS_PIPELINE_MANAGER => "FT_IFTS_PIPELINE_MANAGER",
            WaitType::FT_IFTS_PROJECT_FD_INFO_MAP => "FT_IFTS_PROJECT_FD_INFO_MAP",
            WaitType::FT_IFTS_RWLOCK => "FT_IFTS_RWLOCK",
            WaitType::FT_IFTS_SCHEDULER => "FT_IFTS_SCHEDULER",
            WaitType::FT_IFTS_SCHEDULER_IDLE_WAIT => "FT_IFTS_SCHEDULER_IDLE_WAIT",
            WaitType::FT_IFTS_SHARED_MEMORY => "FT_IFTS_SHARED_MEMORY",
            WaitType::FT_IFTS_SHUTDOWN_PIPE => "FT_IFTS_SHUTDOWN_PIPE",
            WaitType::FT_IFTS_SRCH_FD_MANAGER => "FT_IFTS_SRCH_FD_MANAGER",
            WaitType::FT_IFTS_SRCH_FD_SERVICE => "FT_IFTS_SRCH_FD_SERVICE",
            WaitType::FT_IFTS_STOPLIST_CACHE_MANAGER => "FT_IFTS_STOPLIST_CACHE_MANAGER",
            WaitType::FT_IFTS_THESAURUS => "FT_IFTS_THESAURUS",
            WaitType::FT_IFTS_VERSION_MANAGER => "FT_IFTS_VERSION_MANAGER",
            WaitType::FT_IFTS_WORK_QUEUE => "FT_IFTS_WORK_QUEUE",
            WaitType::FT_MASTER_MERGE => "FT_MASTER_MERGE",
            WaitType::FT_MASTER_MERGE_COORDINATOR => "FT_MASTER_MERGE_COORDINATOR",
            WaitType::FT_METADATA_MUTEX => "FT_METADATA_MUTEX",
            WaitType::FT_PROPERTYLIST_CACHE => "FT_PROPERTYLIST_CACHE",
            WaitType::FT_RESTART_CRAWL => "FT_RESTART_CRAWL",
            WaitType::FULLTEXT_GATHERER => "FULLTEXT GATHERER",
            WaitType::GDMA_GET_RESOURCE_OWNER => "GDMA_GET_RESOURCE_OWNER",
            WaitType::GHOSTCLEANUPSYNCMGR => "GHOSTCLEANUPSYNCMGR",
            WaitType::GHOSTCLEANUP_UPDATE_STATS => "GHOSTCLEANUP_UPDATE_STATS",
            WaitType::GLOBAL_QUERY_CANCEL => "GLOBAL_QUERY_CANCEL",
            WaitType::GLOBAL_QUERY_CLOSE => "GLOBAL_QUERY_CLOSE",
            WaitType::GLOBAL_QUERY_CONSUMER => "GLOBAL_QUERY_CONSUMER",
            WaitType::GLOBAL_QUERY_PRODUCER => "GLOBAL_QUERY_PRODUCER",
            WaitType::GLOBAL_TRAN_CREATE => "GLOBAL_TRAN_CREATE",
            WaitType::GLOBAL_TRAN_UCS_SESSION => "GLOBAL_TRAN_UCS_SESSION",
            WaitType::GUARDIAN => "GUARDIAN",
            WaitType::HADR_AG_MUTEX => "HADR_AG_MUTEX",
            WaitType::HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST => "HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST",
            WaitType::HADR_AR_CRITICAL_SECTION_ENTRY => "HADR_AR_CRITICAL_SECTION_ENTRY",
            WaitType::HADR_AR_MANAGER_MUTEX => "HADR_AR_MANAGER_MUTEX",
            WaitType::HADR_AR_UNLOAD_COMPLETED => "HADR_AR_UNLOAD_COMPLETED",
            WaitType::HADR_BACKUP_BULK_LOCK => "HADR_BACKUP_BULK_LOCK",
            WaitType::HADR_BACKUP_QUEUE => "HADR_BACKUP_QUEUE",
            WaitType::HADR_CLUSAPI_CALL => "HADR_CLUSAPI_CALL",
            WaitType::HADR_COMPRESSED_CACHE_SYNC => "HADR_COMPRESSED_CACHE_SYNC",
            WaitType::HADR_CONNECTIVITY_INFO => "HADR_CONNECTIVITY_INFO",
            WaitType::HADR_DATABASE_FLOW_CONTROL => "HADR_DATABASE_FLOW_CONTROL",
            WaitType::HADR_DATABASE_VERSIONING_STATE => "HADR_DATABASE_VERSIONING_STATE",
            WaitType::HADR_DATABASE_WAIT_FOR_RECOVERY => "HADR_DATABASE_WAIT_FOR_RECOVERY",
            WaitType::HADR_DATABASE_WAIT_FOR_RESTART => "HADR_DATABASE_WAIT_FOR_RESTART",
            WaitType::HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING => "HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING",
            WaitType::HADR_DBR_SUBSCRIBER => "HADR_DBR_SUBSCRIBER",
            WaitType::HADR_DBR_SUBSCRIBER_FILTER_LIST => "HADR_DBR_SUBSCRIBER_FILTER_LIST",
            WaitType::HADR_DBSEEDING => "HADR_DBSEEDING",
            WaitType::HADR_DBSEEDING_LIST => "HADR_DBSEEDING_LIST",
            WaitType::HADR_DBSTATECHANGE_SYNC => "HADR_DBSTATECHANGE_SYNC",
            WaitType::HADR_DB_COMMAND => "HADR_DB_COMMAND",
            WaitType::HADR_DB_OP_COMPLETION_SYNC => "HADR_DB_OP_COMPLETION_SYNC",
            WaitType::HADR_DB_OP_START_SYNC => "HADR_DB_OP_START_SYNC",
            WaitType::HADR_FABRIC_CALLBACK => "HADR_FABRIC_CALLBACK",
            WaitType::HADR_FILESTREAM_BLOCK_FLUSH => "HADR_FILESTREAM_BLOCK_FLUSH",
            WaitType::HADR_FILESTREAM_FILE_CLOSE => "HADR_FILESTREAM_FILE_CLOSE",
            WaitType::HADR_FILESTREAM_FILE_REQUEST => "HADR_FILESTREAM_FILE_REQUEST",
            WaitType::HADR_FILESTREAM_IOMGR => "HADR_FILESTREAM_IOMGR",
            WaitType::HADR_FILESTREAM_IOMGR_IOCOMPLETION => "HADR_FILESTREAM_IOMGR_IOCOMPLETION",
            WaitType::HADR_FILESTREAM_MANAGER => "HADR_FILESTREAM_MANAGER",
            WaitType::HADR_FILESTREAM_PREPROC => "HADR_FILESTREAM_PREPROC",
            WaitType::HADR_GROUP_COMMIT => "HADR_GROUP_COMMIT",
            WaitType::HADR_LOGCAPTURE_SYNC => "HADR_LOGCAPTURE_SYNC",
            WaitType::HADR_LOGCAPTURE_WAIT => "HADR_LOGCAPTURE_WAIT",
            WaitType::HADR_LOGPROGRESS_SYNC => "HADR_LOGPROGRESS_SYNC",
            WaitType::HADR_NOTIFICATION_DEQUEUE => "HADR_NOTIFICATION_DEQUEUE",
            WaitType::HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS => "HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS",
            WaitType::HADR_NOTIFICATION_WORKER_STARTUP_SYNC => "HADR_NOTIFICATION_WORKER_STARTUP_SYNC",
            WaitType::HADR_NOTIFICATION_WORKER_TERMINATION_SYNC => "HADR_NOTIFICATION_WORKER_TERMINATION_SYNC",
            WaitType::HADR_PARTNER_SYNC => "HADR_PARTNER_SYNC",
            WaitType::HADR_READ_ALL_NETWORKS => "HADR_READ_ALL_NETWORKS",
            WaitType::HADR_RECOVERY_WAIT_FOR_CONNECTION => "HADR_RECOVERY_WAIT_FOR_CONNECTION",
            WaitType::HADR_RECOVERY_WAIT_FOR_UNDO => "HADR_RECOVERY_WAIT_FOR_UNDO",
            WaitType::HADR_REPLICAINFO_SYNC => "HADR_REPLICAINFO_SYNC",
            WaitType::HADR_SEEDING_CANCELLATION => "HADR_SEEDING_CANCELLATION",
            WaitType::HADR_SEEDING_FILE_LIST => "HADR_SEEDING_FILE_LIST",
            WaitType::HADR_SEEDING_LIMIT_BACKUPS => "HADR_SEEDING_LIMIT_BACKUPS",
            WaitType::HADR_SEEDING_SYNC_COMPLETION => "HADR_SEEDING_SYNC_COMPLETION",
            WaitType::HADR_SEEDING_TIMEOUT_TASK => "HADR_SEEDING_TIMEOUT_TASK",
            WaitType::HADR_SEEDING_WAIT_FOR_COMPLETION => "HADR_SEEDING_WAIT_FOR_COMPLETION",
            WaitType::HADR_SYNCHRONIZING_THROTTLE => "HADR_SYNCHRONIZING_THROTTLE",
            WaitType::HADR_SYNC_COMMIT => "HADR_SYNC_COMMIT",
            WaitType::HADR_TDS_LISTENER_SYNC => "HADR_TDS_LISTENER_SYNC",
            WaitType::HADR_TDS_LISTENER_SYNC_PROCESSING => "HADR_TDS_LISTENER_SYNC_PROCESSING",
            WaitType::HADR_THROTTLE_LOG_RATE_GOVERNOR => "HADR_THROTTLE_LOG_RATE_GOVERNOR",
            WaitType::HADR_THROTTLE_LOG_RATE_LOG_SIZE => "HADR_THROTTLE_LOG_RATE_LOG_SIZE",
            WaitType::HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO => "HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO",
            WaitType::HADR_THROTTLE_LOG_RATE_SEEDING => "HADR_THROTTLE_LOG_RATE_SEEDING",
            WaitType::HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE => "HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE",
            WaitType::HADR_TIMER_TASK => "HADR_TIMER_TASK",
            WaitType::HADR_TRANSPORT_DBRLIST => "HADR_TRANSPORT_DBRLIST",
            WaitType::HADR_TRANSPORT_FLOW_CONTROL => "HADR_TRANSPORT_FLOW_CONTROL",
            WaitType::HADR_TRANSPORT_SESSION => "HADR_TRANSPORT_SESSION",
            WaitType::HADR_WORK_POOL => "HADR_WORK_POOL",
            WaitType::HADR_WORK_QUEUE => "HADR_WORK_QUEUE",
            WaitType::HADR_XRF_STACK_ACCESS => "HADR_XRF_STACK_ACCESS",
            WaitType::HCCO_CACHE => "HCCO_CACHE",
            WaitType::HKCS_PARALLEL_MIGRATION => "HKCS_PARALLEL_MIGRATION",
            WaitType::HKCS_PARALLEL_RECOVERY => "HKCS_PARALLEL_RECOVERY",
            WaitType::HK_RESTORE_FILEMAP => "HK_RESTORE_FILEMAP",
            WaitType::HTBUILD => "HTBUILD",
            WaitType::HTBUILD_AGG => "HTBUILD_AGG",
            WaitType::HTBUILD_JOIN => "HTBUILD_JOIN",
            WaitType::HTDELETE => "HTDELETE",
            WaitType::HTDELETE_AGG => "HTDELETE_AGG",
            WaitType::HTDELETE_JOIN => "HTDELETE_JOIN",
            WaitType::HTMEMO => "HTMEMO",
            WaitType::HTREINIT => "HTREINIT",
            WaitType::HTREPARTITION => "HTREPARTITION",
            WaitType::HTTP_ENUMERATION => "HTTP_ENUMERATION",
            WaitType::HTTP_START => "HTTP_START",
            WaitType::HTTP_STORAGE_CONNECTION => "HTTP_STORAGE_CONNECTION",
            WaitType::IMPPROV_IOWAIT => "IMPPROV_IOWAIT",
            WaitType::INSTANCE_LOG_RATE_GOVERNOR => "INSTANCE_LOG_RATE_GOVERNOR",
            WaitType::INTERNAL_TESTING => "INTERNAL_TESTING",
            WaitType::IOAFF_RANGE_QUEUE => "IOAFF_RANGE_QUEUE",
            WaitType::IO_AUDIT_MUTEX => "IO_AUDIT_MUTEX",
            WaitType::IO_COMPLETION => "IO_COMPLETION",
            WaitType::IO_QUEUE_LIMIT => "IO_QUEUE_LIMIT",
            WaitType::IO_RETRY => "IO_RETRY",
            WaitType::KSOURCE_WAKEUP => "KSOURCE_WAKEUP",
            WaitType::KTM_ENLISTMENT => "KTM_ENLISTMENT",
            WaitType::KTM_RECOVERY_MANAGER => "KTM_RECOVERY_MANAGER",
            WaitType::KTM_RECOVERY_RESOLUTION => "KTM_RECOVERY_RESOLUTION",
            WaitType::LATCH_DT => "LATCH_DT",
            WaitType::LATCH_EX => "LATCH_EX",
            WaitType::LATCH_KP => "LATCH_KP",
            WaitType::LATCH_NL => "LATCH_NL",
            WaitType::LATCH_SH => "LATCH_SH",
            WaitType::LATCH_UP => "LATCH_UP",
            WaitType::LAZYWRITER_SLEEP => "LAZYWRITER_SLEEP",
            WaitType::LCK_M_BU => "LCK_M_BU",
            WaitType::LCK_M_BU_ABORT_BLOCKERS => "LCK_M_BU_ABORT_BLOCKERS",
            WaitType::LCK_M_BU_LOW_PRIORITY => "LCK_M_BU_LOW_PRIORITY",
            WaitType::LCK_M_IS => "LCK_M_IS",
            WaitType::LCK_M_IS_ABORT_BLOCKERS => "LCK_M_IS_ABORT_BLOCKERS",
            WaitType::LCK_M_IS_LOW_PRIORITY => "LCK_M_IS_LOW_PRIORITY",
            WaitType::LCK_M_IU => "LCK_M_IU",
            WaitType::LCK_M_IU_ABORT_BLOCKERS => "LCK_M_IU_ABORT_BLOCKERS",
            WaitType::LCK_M_IU_LOW_PRIORITY => "LCK_M_IU_LOW_PRIORITY",
            WaitType::LCK_M_IX => "LCK_M_IX",
            WaitType::LCK_M_IX_ABORT_BLOCKERS => "LCK_M_IX_ABORT_BLOCKERS",
            WaitType::LCK_M_IX_LOW_PRIORITY => "LCK_M_IX_LOW_PRIORITY",
            WaitType::LCK_M_RIn_NL => "LCK_M_RIn_NL",
            WaitType::LCK_M_RIn_NL_ABORT_BLOCKERS => "LCK_M_RIn_NL_ABORT_BLOCKERS",
            WaitType::LCK_M_RIn_NL_LOW_PRIORITY => "LCK_M_RIn_NL_LOW_PRIORITY",
            WaitType::LCK_M_RIn_S => "LCK_M_RIn_S",
            WaitType::LCK_M_RIn_S_ABORT_BLOCKERS => "LCK_M_RIn_S_ABORT_BLOCKERS",
            WaitType::LCK_M_RIn_S_LOW_PRIORITY => "LCK_M_RIn_S_LOW_PRIORITY",
            WaitType::LCK_M_RIn_U => "LCK_M_RIn_U",
            WaitType::LCK_M_RIn_U_ABORT_BLOCKERS => "LCK_M_RIn_U_ABORT_BLOCKERS",
            WaitType::LCK_M_RIn_U_LOW_PRIORITY => "LCK_M_RIn_U_LOW_PRIORITY",
            WaitType::LCK_M_RIn_X => "LCK_M_RIn_X",
            WaitType::LCK_M_RIn_X_ABORT_BLOCKERS => "LCK_M_RIn_X_ABORT_BLOCKERS",
            WaitType::LCK_M_RIn_X_LOW_PRIORITY => "LCK_M_RIn_X_LOW_PRIORITY",
            WaitType::LCK_M_RS_S => "LCK_M_RS_S",
            WaitType::LCK_M_RS_S_ABORT_BLOCKERS => "LCK_M_RS_S_ABORT_BLOCKERS",
            WaitType::LCK_M_RS_S_LOW_PRIORITY => "LCK_M_RS_S_LOW_PRIORITY",
            WaitType::LCK_M_RS_U => "LCK_M_RS_U",
            WaitType::LCK_M_RS_U_ABORT_BLOCKERS => "LCK_M_RS_U_ABORT_BLOCKERS",
            WaitType::LCK_M_RS_U_LOW_PRIORITY => "LCK_M_RS_U_LOW_PRIORITY",
            WaitType::LCK_M_RX_S => "LCK_M_RX_S",
            WaitType::LCK_M_RX_S_ABORT_BLOCKERS => "LCK_M_RX_S_ABORT_BLOCKERS",
            WaitType::LCK_M_RX_S_LOW_PRIORITY => "LCK_M_RX_S_LOW_PRIORITY",
            WaitType::LCK_M_RX_U => "LCK_M_RX_U",
            WaitType::LCK_M_RX_U_ABORT_BLOCKERS => "LCK_M_RX_U_ABORT_BLOCKERS",
            WaitType::LCK_M_RX_U_LOW_PRIORITY => "LCK_M_RX_U_LOW_PRIORITY",
            WaitType::LCK_M_RX_X => "LCK_M_RX_X",
            WaitType::LCK_M_RX_X_ABORT_BLOCKERS => "LCK_M_RX_X_ABORT_BLOCKERS",
            WaitType::LCK_M_RX_X_LOW_PRIORITY => "LCK_M_RX_X_LOW_PRIORITY",
            WaitType::LCK_M_S => "LCK_M_S",
            WaitType::LCK_M_SCH_M => "LCK_M_SCH_M",
            WaitType::LCK_M_SCH_M_ABORT_BLOCKERS => "LCK_M_SCH_M_ABORT_BLOCKERS",
            WaitType::LCK_M_SCH_M_LOW_PRIORITY => "LCK_M_SCH_M_LOW_PRIORITY",
            WaitType::LCK_M_SCH_S => "LCK_M_SCH_S",
            WaitType::LCK_M_SCH_S_ABORT_BLOCKERS => "LCK_M_SCH_S_ABORT_BLOCKERS",
            WaitType::LCK_M_SCH_S_LOW_PRIORITY => "LCK_M_SCH_S_LOW_PRIORITY",
            WaitType::LCK_M_SIU => "LCK_M_SIU",
            WaitType::LCK_M_SIU_ABORT_BLOCKERS => "LCK_M_SIU_ABORT_BLOCKERS",
            WaitType::LCK_M_SIU_LOW_PRIORITY => "LCK_M_SIU_LOW_PRIORITY",
            WaitType::LCK_M_SIX => "LCK_M_SIX",
            WaitType::LCK_M_SIX_ABORT_BLOCKERS => "LCK_M_SIX_ABORT_BLOCKERS",
            WaitType::LCK_M_SIX_LOW_PRIORITY => "LCK_M_SIX_LOW_PRIORITY",
            WaitType::LCK_M_S_ABORT_BLOCKERS => "LCK_M_S_ABORT_BLOCKERS",
            WaitType::LCK_M_S_LOW_PRIORITY => "LCK_M_S_LOW_PRIORITY",
            WaitType::LCK_M_S_XACT => "LCK_M_S_XACT",
            WaitType::LCK_M_S_XACT_MODIFY => "LCK_M_S_XACT_MODIFY",
            WaitType::LCK_M_S_XACT_READ => "LCK_M_S_XACT_READ",
            WaitType::LCK_M_U => "LCK_M_U",
            WaitType::LCK_M_UIX => "LCK_M_UIX",
            WaitType::LCK_M_UIX_ABORT_BLOCKERS => "LCK_M_UIX_ABORT_BLOCKERS",
            WaitType::LCK_M_UIX_LOW_PRIORITY => "LCK_M_UIX_LOW_PRIORITY",
            WaitType::LCK_M_U_ABORT_BLOCKERS => "LCK_M_U_ABORT_BLOCKERS",
            WaitType::LCK_M_U_LOW_PRIORITY => "LCK_M_U_LOW_PRIORITY",
            WaitType::LCK_M_X => "LCK_M_X",
            WaitType::LCK_M_X_ABORT_BLOCKERS => "LCK_M_X_ABORT_BLOCKERS",
            WaitType::LCK_M_X_LOW_PRIORITY => "LCK_M_X_LOW_PRIORITY",
            WaitType::LOGBUFFER => "LOGBUFFER",
            WaitType::LOGCAPTURE_LOGPOOLTRUNCPOINT => "LOGCAPTURE_LOGPOOLTRUNCPOINT",
            WaitType::LOGGENERATION => "LOGGENERATION",
            WaitType::LOGMGR => "LOGMGR",
            WaitType::LOGMGR_FLUSH => "LOGMGR_FLUSH",
            WaitType::LOGMGR_PMM_LOG => "LOGMGR_PMM_LOG",
            WaitType::LOGMGR_QUEUE => "LOGMGR_QUEUE",
            WaitType::LOGMGR_RESERVE_APPEND => "LOGMGR_RESERVE_APPEND",
            WaitType::LOGPOOLREFCOUNTEDOBJECT_REFDONE => "LOGPOOLREFCOUNTEDOBJECT_REFDONE",
            WaitType::LOGPOOL_CACHESIZE => "LOGPOOL_CACHESIZE",
            WaitType::LOGPOOL_CONSUMER => "LOGPOOL_CONSUMER",
            WaitType::LOGPOOL_CONSUMERSET => "LOGPOOL_CONSUMERSET",
            WaitType::LOGPOOL_FREEPOOLS => "LOGPOOL_FREEPOOLS",
            WaitType::LOGPOOL_MGRSET => "LOGPOOL_MGRSET",
            WaitType::LOGPOOL_REPLACEMENTSET => "LOGPOOL_REPLACEMENTSET",
            WaitType::LOG_POOL_SCAN => "LOG_POOL_SCAN",
            WaitType::LOG_RATE_GOVERNOR => "LOG_RATE_GOVERNOR",
            WaitType::LOWFAIL_MEMMGR_QUEUE => "LOWFAIL_MEMMGR_QUEUE",
            WaitType::MD_AGENT_YIELD => "MD_AGENT_YIELD",
            WaitType::MD_LAZYCACHE_RWLOCK => "MD_LAZYCACHE_RWLOCK",
            WaitType::MEMORY_ALLOCATION_EXT => "MEMORY_ALLOCATION_EXT",
            WaitType::MEMORY_GRANT_UPDATE => "MEMORY_GRANT_UPDATE",
            WaitType::METADATA_LAZYCACHE_RWLOCK => "METADATA_LAZYCACHE_RWLOCK",
            WaitType::MIGRATIONBUFFER => "MIGRATIONBUFFER",
            WaitType::MISCELLANEOUS => "MISCELLANEOUS",
            WaitType::MSQL_DQ => "MSQL_DQ",
            WaitType::MSQL_XACT_MGR_MUTEX => "MSQL_XACT_MGR_MUTEX",
            WaitType::MSQL_XACT_MUTEX => "MSQL_XACT_MUTEX",
            WaitType::MSQL_XP => "MSQL_XP",
            WaitType::MSSEARCH => "MSSEARCH",
            WaitType::NETWORKSXMLMGRLOAD => "NETWORKSXMLMGRLOAD",
            WaitType::NET_WAITFOR_PACKET => "NET_WAITFOR_PACKET",
            WaitType::NODE_CACHE_MUTEX => "NODE_CACHE_MUTEX",
            WaitType::OLEDB => "OLEDB",
            WaitType::ONDEMAND_TASK_QUEUE => "ONDEMAND_TASK_QUEUE",
            WaitType::PAGEIOLATCH_DT => "PAGEIOLATCH_DT",
            WaitType::PAGEIOLATCH_EX => "PAGEIOLATCH_EX",
            WaitType::PAGEIOLATCH_KP => "PAGEIOLATCH_KP",
            WaitType::PAGEIOLATCH_NL => "PAGEIOLATCH_NL",
            WaitType::PAGEIOLATCH_SH => "PAGEIOLATCH_SH",
            WaitType::PAGEIOLATCH_UP => "PAGEIOLATCH_UP",
            WaitType::PAGELATCH_DT => "PAGELATCH_DT",
            WaitType::PAGELATCH_EX => "PAGELATCH_EX",
            WaitType::PAGELATCH_KP => "PAGELATCH_KP",
            WaitType::PAGELATCH_NL => "PAGELATCH_NL",
            WaitType::PAGELATCH_SH => "PAGELATCH_SH",
            WaitType::PAGELATCH_UP => "PAGELATCH_UP",
            WaitType::PARALLEL_BACKUP_QUEUE => "PARALLEL_BACKUP_QUEUE",
            WaitType::PARALLEL_REDO_DRAIN_WORKER => "PARALLEL_REDO_DRAIN_WORKER",
            WaitType::PARALLEL_REDO_FLOW_CONTROL => "PARALLEL_REDO_FLOW_CONTROL",
            WaitType::PARALLEL_REDO_LOG_CACHE => "PARALLEL_REDO_LOG_CACHE",
            WaitType::PARALLEL_REDO_TRAN_LIST => "PARALLEL_REDO_TRAN_LIST",
            WaitType::PARALLEL_REDO_TRAN_TURN => "PARALLEL_REDO_TRAN_TURN",
            WaitType::PARALLEL_REDO_WORKER_SYNC => "PARALLEL_REDO_WORKER_SYNC",
            WaitType::PARALLEL_REDO_WORKER_WAIT_WORK => "PARALLEL_REDO_WORKER_WAIT_WORK",
            WaitType::PERFORMANCE_COUNTERS_RWLOCK => "PERFORMANCE_COUNTERS_RWLOCK",
            WaitType::PHYSICAL_SEEDING_DMV => "PHYSICAL_SEEDING_DMV",
            WaitType::POOL_LOG_RATE_GOVERNOR => "POOL_LOG_RATE_GOVERNOR",
            WaitType::PREEMPTIVE_ABR => "PREEMPTIVE_ABR",
            WaitType::PREEMPTIVE_AUDIT_ACCESS_EVENTLOG => "PREEMPTIVE_AUDIT_ACCESS_EVENTLOG",
            WaitType::PREEMPTIVE_AUDIT_ACCESS_SECLOG => "PREEMPTIVE_AUDIT_ACCESS_SECLOG",
            WaitType::PREEMPTIVE_CLOSEBACKUPMEDIA => "PREEMPTIVE_CLOSEBACKUPMEDIA",
            WaitType::PREEMPTIVE_CLOSEBACKUPTAPE => "PREEMPTIVE_CLOSEBACKUPTAPE",
            WaitType::PREEMPTIVE_CLOSEBACKUPVDIDEVICE => "PREEMPTIVE_CLOSEBACKUPVDIDEVICE",
            WaitType::PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL => "PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL",
            WaitType::PREEMPTIVE_COM_COCREATEINSTANCE => "PREEMPTIVE_COM_COCREATEINSTANCE",
            WaitType::PREEMPTIVE_COM_COGETCLASSOBJECT => "PREEMPTIVE_COM_COGETCLASSOBJECT",
            WaitType::PREEMPTIVE_COM_CREATEACCESSOR => "PREEMPTIVE_COM_CREATEACCESSOR",
            WaitType::PREEMPTIVE_COM_DELETEROWS => "PREEMPTIVE_COM_DELETEROWS",
            WaitType::PREEMPTIVE_COM_GETCOMMANDTEXT => "PREEMPTIVE_COM_GETCOMMANDTEXT",
            WaitType::PREEMPTIVE_COM_GETDATA => "PREEMPTIVE_COM_GETDATA",
            WaitType::PREEMPTIVE_COM_GETNEXTROWS => "PREEMPTIVE_COM_GETNEXTROWS",
            WaitType::PREEMPTIVE_COM_GETRESULT => "PREEMPTIVE_COM_GETRESULT",
            WaitType::PREEMPTIVE_COM_GETROWSBYBOOKMARK => "PREEMPTIVE_COM_GETROWSBYBOOKMARK",
            WaitType::PREEMPTIVE_COM_LBFLUSH => "PREEMPTIVE_COM_LBFLUSH",
            WaitType::PREEMPTIVE_COM_LBLOCKREGION => "PREEMPTIVE_COM_LBLOCKREGION",
            WaitType::PREEMPTIVE_COM_LBREADAT => "PREEMPTIVE_COM_LBREADAT",
            WaitType::PREEMPTIVE_COM_LBSETSIZE => "PREEMPTIVE_COM_LBSETSIZE",
            WaitType::PREEMPTIVE_COM_LBSTAT => "PREEMPTIVE_COM_LBSTAT",
            WaitType::PREEMPTIVE_COM_LBUNLOCKREGION => "PREEMPTIVE_COM_LBUNLOCKREGION",
            WaitType::PREEMPTIVE_COM_LBWRITEAT => "PREEMPTIVE_COM_LBWRITEAT",
            WaitType::PREEMPTIVE_COM_QUERYINTERFACE => "PREEMPTIVE_COM_QUERYINTERFACE",
            WaitType::PREEMPTIVE_COM_RELEASE => "PREEMPTIVE_COM_RELEASE",
            WaitType::PREEMPTIVE_COM_RELEASEACCESSOR => "PREEMPTIVE_COM_RELEASEACCESSOR",
            WaitType::PREEMPTIVE_COM_RELEASEROWS => "PREEMPTIVE_COM_RELEASEROWS",
            WaitType::PREEMPTIVE_COM_RELEASESESSION => "PREEMPTIVE_COM_RELEASESESSION",
            WaitType::PREEMPTIVE_COM_RESTARTPOSITION => "PREEMPTIVE_COM_RESTARTPOSITION",
            WaitType::PREEMPTIVE_COM_SEQSTRMREAD => "PREEMPTIVE_COM_SEQSTRMREAD",
            WaitType::PREEMPTIVE_COM_SEQSTRMREADANDWRITE => "PREEMPTIVE_COM_SEQSTRMREADANDWRITE",
            WaitType::PREEMPTIVE_COM_SETDATAFAILURE => "PREEMPTIVE_COM_SETDATAFAILURE",
            WaitType::PREEMPTIVE_COM_SETPARAMETERINFO => "PREEMPTIVE_COM_SETPARAMETERINFO",
            WaitType::PREEMPTIVE_COM_SETPARAMETERPROPERTIES => "PREEMPTIVE_COM_SETPARAMETERPROPERTIES",
            WaitType::PREEMPTIVE_COM_STRMLOCKREGION => "PREEMPTIVE_COM_STRMLOCKREGION",
            WaitType::PREEMPTIVE_COM_STRMSEEKANDREAD => "PREEMPTIVE_COM_STRMSEEKANDREAD",
            WaitType::PREEMPTIVE_COM_STRMSEEKANDWRITE => "PREEMPTIVE_COM_STRMSEEKANDWRITE",
            WaitType::PREEMPTIVE_COM_STRMSETSIZE => "PREEMPTIVE_COM_STRMSETSIZE",
            WaitType::PREEMPTIVE_COM_STRMSTAT => "PREEMPTIVE_COM_STRMSTAT",
            WaitType::PREEMPTIVE_COM_STRMUNLOCKREGION => "PREEMPTIVE_COM_STRMUNLOCKREGION",
            WaitType::PREEMPTIVE_CONSOLEWRITE => "PREEMPTIVE_CONSOLEWRITE",
            WaitType::PREEMPTIVE_CREATEPARAM => "PREEMPTIVE_CREATEPARAM",
            WaitType::PREEMPTIVE_DEBUG => "PREEMPTIVE_DEBUG",
            WaitType::PREEMPTIVE_DFSADDLINK => "PREEMPTIVE_DFSADDLINK",
            WaitType::PREEMPTIVE_DFSLINKEXISTCHECK => "PREEMPTIVE_DFSLINKEXISTCHECK",
            WaitType::PREEMPTIVE_DFSLINKHEALTHCHECK => "PREEMPTIVE_DFSLINKHEALTHCHECK",
            WaitType::PREEMPTIVE_DFSREMOVELINK => "PREEMPTIVE_DFSREMOVELINK",
            WaitType::PREEMPTIVE_DFSREMOVEROOT => "PREEMPTIVE_DFSREMOVEROOT",
            WaitType::PREEMPTIVE_DFSROOTFOLDERCHECK => "PREEMPTIVE_DFSROOTFOLDERCHECK",
            WaitType::PREEMPTIVE_DFSROOTINIT => "PREEMPTIVE_DFSROOTINIT",
            WaitType::PREEMPTIVE_DFSROOTSHARECHECK => "PREEMPTIVE_DFSROOTSHARECHECK",
            WaitType::PREEMPTIVE_DTC_ABORT => "PREEMPTIVE_DTC_ABORT",
            WaitType::PREEMPTIVE_DTC_ABORTREQUESTDONE => "PREEMPTIVE_DTC_ABORTREQUESTDONE",
            WaitType::PREEMPTIVE_DTC_BEGINTRANSACTION => "PREEMPTIVE_DTC_BEGINTRANSACTION",
            WaitType::PREEMPTIVE_DTC_COMMITREQUESTDONE => "PREEMPTIVE_DTC_COMMITREQUESTDONE",
            WaitType::PREEMPTIVE_DTC_ENLIST => "PREEMPTIVE_DTC_ENLIST",
            WaitType::PREEMPTIVE_DTC_PREPAREREQUESTDONE => "PREEMPTIVE_DTC_PREPAREREQUESTDONE",
            WaitType::PREEMPTIVE_FILESIZEGET => "PREEMPTIVE_FILESIZEGET",
            WaitType::PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION => "PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION",
            WaitType::PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION => "PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION",
            WaitType::PREEMPTIVE_FSAOLEDB_STARTTRANSACTION => "PREEMPTIVE_FSAOLEDB_STARTTRANSACTION",
            WaitType::PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO => "PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO",
            WaitType::PREEMPTIVE_GETRMINFO => "PREEMPTIVE_GETRMINFO",
            WaitType::PREEMPTIVE_HADR_LEASE_MECHANISM => "PREEMPTIVE_HADR_LEASE_MECHANISM",
            WaitType::PREEMPTIVE_HTTP_EVENT_WAIT => "PREEMPTIVE_HTTP_EVENT_WAIT",
            WaitType::PREEMPTIVE_HTTP_REQUEST => "PREEMPTIVE_HTTP_REQUEST",
            WaitType::PREEMPTIVE_LOCKMONITOR => "PREEMPTIVE_LOCKMONITOR",
            WaitType::PREEMPTIVE_MSS_RELEASE => "PREEMPTIVE_MSS_RELEASE",
            WaitType::PREEMPTIVE_ODBCOPS => "PREEMPTIVE_ODBCOPS",
            WaitType::PREEMPTIVE_OLEDBOPS => "PREEMPTIVE_OLEDBOPS",
            WaitType::PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN => "PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN",
            WaitType::PREEMPTIVE_OLEDB_ABORTTRAN => "PREEMPTIVE_OLEDB_ABORTTRAN",
            WaitType::PREEMPTIVE_OLEDB_GETDATASOURCE => "PREEMPTIVE_OLEDB_GETDATASOURCE",
            WaitType::PREEMPTIVE_OLEDB_GETLITERALINFO => "PREEMPTIVE_OLEDB_GETLITERALINFO",
            WaitType::PREEMPTIVE_OLEDB_GETPROPERTIES => "PREEMPTIVE_OLEDB_GETPROPERTIES",
            WaitType::PREEMPTIVE_OLEDB_GETPROPERTYINFO => "PREEMPTIVE_OLEDB_GETPROPERTYINFO",
            WaitType::PREEMPTIVE_OLEDB_GETSCHEMALOCK => "PREEMPTIVE_OLEDB_GETSCHEMALOCK",
            WaitType::PREEMPTIVE_OLEDB_JOINTRANSACTION => "PREEMPTIVE_OLEDB_JOINTRANSACTION",
            WaitType::PREEMPTIVE_OLEDB_RELEASE => "PREEMPTIVE_OLEDB_RELEASE",
            WaitType::PREEMPTIVE_OLEDB_SETPROPERTIES => "PREEMPTIVE_OLEDB_SETPROPERTIES",
            WaitType::PREEMPTIVE_OLE_UNINIT => "PREEMPTIVE_OLE_UNINIT",
            WaitType::PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT => "PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT",
            WaitType::PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE => "PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE",
            WaitType::PREEMPTIVE_OS_AUTHENTICATIONOPS => "PREEMPTIVE_OS_AUTHENTICATIONOPS",
            WaitType::PREEMPTIVE_OS_AUTHORIZATIONOPS => "PREEMPTIVE_OS_AUTHORIZATIONOPS",
            WaitType::PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT => "PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT",
            WaitType::PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID => "PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID",
            WaitType::PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER => "PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER",
            WaitType::PREEMPTIVE_OS_BACKUPREAD => "PREEMPTIVE_OS_BACKUPREAD",
            WaitType::PREEMPTIVE_OS_CLOSEHANDLE => "PREEMPTIVE_OS_CLOSEHANDLE",
            WaitType::PREEMPTIVE_OS_CLUSTEROPS => "PREEMPTIVE_OS_CLUSTEROPS",
            WaitType::PREEMPTIVE_OS_COMOPS => "PREEMPTIVE_OS_COMOPS",
            WaitType::PREEMPTIVE_OS_COMPLETEAUTHTOKEN => "PREEMPTIVE_OS_COMPLETEAUTHTOKEN",
            WaitType::PREEMPTIVE_OS_COPYFILE => "PREEMPTIVE_OS_COPYFILE",
            WaitType::PREEMPTIVE_OS_CREATEDIRECTORY => "PREEMPTIVE_OS_CREATEDIRECTORY",
            WaitType::PREEMPTIVE_OS_CREATEFILE => "PREEMPTIVE_OS_CREATEFILE",
            WaitType::PREEMPTIVE_OS_CRYPTACQUIRECONTEXT => "PREEMPTIVE_OS_CRYPTACQUIRECONTEXT",
            WaitType::PREEMPTIVE_OS_CRYPTIMPORTKEY => "PREEMPTIVE_OS_CRYPTIMPORTKEY",
            WaitType::PREEMPTIVE_OS_CRYPTOPS => "PREEMPTIVE_OS_CRYPTOPS",
            WaitType::PREEMPTIVE_OS_DECRYPTMESSAGE => "PREEMPTIVE_OS_DECRYPTMESSAGE",
            WaitType::PREEMPTIVE_OS_DELETEFILE => "PREEMPTIVE_OS_DELETEFILE",
            WaitType::PREEMPTIVE_OS_DELETESECURITYCONTEXT => "PREEMPTIVE_OS_DELETESECURITYCONTEXT",
            WaitType::PREEMPTIVE_OS_DEVICEIOCONTROL => "PREEMPTIVE_OS_DEVICEIOCONTROL",
            WaitType::PREEMPTIVE_OS_DEVICEOPS => "PREEMPTIVE_OS_DEVICEOPS",
            WaitType::PREEMPTIVE_OS_DIRSVC_NETWORKOPS => "PREEMPTIVE_OS_DIRSVC_NETWORKOPS",
            WaitType::PREEMPTIVE_OS_DISCONNECTNAMEDPIPE => "PREEMPTIVE_OS_DISCONNECTNAMEDPIPE",
            WaitType::PREEMPTIVE_OS_DOMAINSERVICESOPS => "PREEMPTIVE_OS_DOMAINSERVICESOPS",
            WaitType::PREEMPTIVE_OS_DSGETDCNAME => "PREEMPTIVE_OS_DSGETDCNAME",
            WaitType::PREEMPTIVE_OS_DTCOPS => "PREEMPTIVE_OS_DTCOPS",
            WaitType::PREEMPTIVE_OS_ENCRYPTMESSAGE => "PREEMPTIVE_OS_ENCRYPTMESSAGE",
            WaitType::PREEMPTIVE_OS_FILEOPS => "PREEMPTIVE_OS_FILEOPS",
            WaitType::PREEMPTIVE_OS_FINDFILE => "PREEMPTIVE_OS_FINDFILE",
            WaitType::PREEMPTIVE_OS_FLUSHFILEBUFFERS => "PREEMPTIVE_OS_FLUSHFILEBUFFERS",
            WaitType::PREEMPTIVE_OS_FORMATMESSAGE => "PREEMPTIVE_OS_FORMATMESSAGE",
            WaitType::PREEMPTIVE_OS_FREECREDENTIALSHANDLE => "PREEMPTIVE_OS_FREECREDENTIALSHANDLE",
            WaitType::PREEMPTIVE_OS_FREELIBRARY => "PREEMPTIVE_OS_FREELIBRARY",
            WaitType::PREEMPTIVE_OS_GENERICOPS => "PREEMPTIVE_OS_GENERICOPS",
            WaitType::PREEMPTIVE_OS_GETADDRINFO => "PREEMPTIVE_OS_GETADDRINFO",
            WaitType::PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE => "PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE",
            WaitType::PREEMPTIVE_OS_GETDISKFREESPACE => "PREEMPTIVE_OS_GETDISKFREESPACE",
            WaitType::PREEMPTIVE_OS_GETFILEATTRIBUTES => "PREEMPTIVE_OS_GETFILEATTRIBUTES",
            WaitType::PREEMPTIVE_OS_GETFILESIZE => "PREEMPTIVE_OS_GETFILESIZE",
            WaitType::PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE => "PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE",
            WaitType::PREEMPTIVE_OS_GETLONGPATHNAME => "PREEMPTIVE_OS_GETLONGPATHNAME",
            WaitType::PREEMPTIVE_OS_GETPROCADDRESS => "PREEMPTIVE_OS_GETPROCADDRESS",
            WaitType::PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT => "PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT",
            WaitType::PREEMPTIVE_OS_GETVOLUMEPATHNAME => "PREEMPTIVE_OS_GETVOLUMEPATHNAME",
            WaitType::PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT => "PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT",
            WaitType::PREEMPTIVE_OS_LIBRARYOPS => "PREEMPTIVE_OS_LIBRARYOPS",
            WaitType::PREEMPTIVE_OS_LOADLIBRARY => "PREEMPTIVE_OS_LOADLIBRARY",
            WaitType::PREEMPTIVE_OS_LOGONUSER => "PREEMPTIVE_OS_LOGONUSER",
            WaitType::PREEMPTIVE_OS_LOOKUPACCOUNTSID => "PREEMPTIVE_OS_LOOKUPACCOUNTSID",
            WaitType::PREEMPTIVE_OS_MESSAGEQUEUEOPS => "PREEMPTIVE_OS_MESSAGEQUEUEOPS",
            WaitType::PREEMPTIVE_OS_MOVEFILE => "PREEMPTIVE_OS_MOVEFILE",
            WaitType::PREEMPTIVE_OS_NETGROUPGETUSERS => "PREEMPTIVE_OS_NETGROUPGETUSERS",
            WaitType::PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS => "PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS",
            WaitType::PREEMPTIVE_OS_NETUSERGETGROUPS => "PREEMPTIVE_OS_NETUSERGETGROUPS",
            WaitType::PREEMPTIVE_OS_NETUSERGETLOCALGROUPS => "PREEMPTIVE_OS_NETUSERGETLOCALGROUPS",
            WaitType::PREEMPTIVE_OS_NETUSERMODALSGET => "PREEMPTIVE_OS_NETUSERMODALSGET",
            WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY => "PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY",
            WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE => "PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE",
            WaitType::PREEMPTIVE_OS_OPENDIRECTORY => "PREEMPTIVE_OS_OPENDIRECTORY",
            WaitType::PREEMPTIVE_OS_PDH_WMI_INIT => "PREEMPTIVE_OS_PDH_WMI_INIT",
            WaitType::PREEMPTIVE_OS_PIPEOPS => "PREEMPTIVE_OS_PIPEOPS",
            WaitType::PREEMPTIVE_OS_PROCESSOPS => "PREEMPTIVE_OS_PROCESSOPS",
            WaitType::PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES => "PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES",
            WaitType::PREEMPTIVE_OS_QUERYREGISTRY => "PREEMPTIVE_OS_QUERYREGISTRY",
            WaitType::PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN => "PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN",
            WaitType::PREEMPTIVE_OS_REMOVEDIRECTORY => "PREEMPTIVE_OS_REMOVEDIRECTORY",
            WaitType::PREEMPTIVE_OS_REPORTEVENT => "PREEMPTIVE_OS_REPORTEVENT",
            WaitType::PREEMPTIVE_OS_REVERTTOSELF => "PREEMPTIVE_OS_REVERTTOSELF",
            WaitType::PREEMPTIVE_OS_RSFXDEVICEOPS => "PREEMPTIVE_OS_RSFXDEVICEOPS",
            WaitType::PREEMPTIVE_OS_SECURITYOPS => "PREEMPTIVE_OS_SECURITYOPS",
            WaitType::PREEMPTIVE_OS_SERVICEOPS => "PREEMPTIVE_OS_SERVICEOPS",
            WaitType::PREEMPTIVE_OS_SETENDOFFILE => "PREEMPTIVE_OS_SETENDOFFILE",
            WaitType::PREEMPTIVE_OS_SETFILEPOINTER => "PREEMPTIVE_OS_SETFILEPOINTER",
            WaitType::PREEMPTIVE_OS_SETFILEVALIDDATA => "PREEMPTIVE_OS_SETFILEVALIDDATA",
            WaitType::PREEMPTIVE_OS_SETNAMEDSECURITYINFO => "PREEMPTIVE_OS_SETNAMEDSECURITYINFO",
            WaitType::PREEMPTIVE_OS_SQLCLROPS => "PREEMPTIVE_OS_SQLCLROPS",
            WaitType::PREEMPTIVE_OS_SQMLAUNCH => "PREEMPTIVE_OS_SQMLAUNCH",
            WaitType::PREEMPTIVE_OS_VERIFYSIGNATURE => "PREEMPTIVE_OS_VERIFYSIGNATURE",
            WaitType::PREEMPTIVE_OS_VERIFYTRUST => "PREEMPTIVE_OS_VERIFYTRUST",
            WaitType::PREEMPTIVE_OS_VSSOPS => "PREEMPTIVE_OS_VSSOPS",
            WaitType::PREEMPTIVE_OS_WAITFORSINGLEOBJECT => "PREEMPTIVE_OS_WAITFORSINGLEOBJECT",
            WaitType::PREEMPTIVE_OS_WINSOCKOPS => "PREEMPTIVE_OS_WINSOCKOPS",
            WaitType::PREEMPTIVE_OS_WRITEFILE => "PREEMPTIVE_OS_WRITEFILE",
            WaitType::PREEMPTIVE_OS_WRITEFILEGATHER => "PREEMPTIVE_OS_WRITEFILEGATHER",
            WaitType::PREEMPTIVE_OS_WSASETLASTERROR => "PREEMPTIVE_OS_WSASETLASTERROR",
            WaitType::PREEMPTIVE_REENLIST => "PREEMPTIVE_REENLIST",
            WaitType::PREEMPTIVE_RESIZELOG => "PREEMPTIVE_RESIZELOG",
            WaitType::PREEMPTIVE_ROLLFORWARDREDO => "PREEMPTIVE_ROLLFORWARDREDO",
            WaitType::PREEMPTIVE_ROLLFORWARDUNDO => "PREEMPTIVE_ROLLFORWARDUNDO",
            WaitType::PREEMPTIVE_SB_STOPENDPOINT => "PREEMPTIVE_SB_STOPENDPOINT",
            WaitType::PREEMPTIVE_SERVER_STARTUP => "PREEMPTIVE_SERVER_STARTUP",
            WaitType::PREEMPTIVE_SETRMINFO => "PREEMPTIVE_SETRMINFO",
            WaitType::PREEMPTIVE_SHAREDMEM_GETDATA => "PREEMPTIVE_SHAREDMEM_GETDATA",
            WaitType::PREEMPTIVE_SNIOPEN => "PREEMPTIVE_SNIOPEN",
            WaitType::PREEMPTIVE_SOSHOST => "PREEMPTIVE_SOSHOST",
            WaitType::PREEMPTIVE_SOSTESTING => "PREEMPTIVE_SOSTESTING",
            WaitType::PREEMPTIVE_SP_SERVER_DIAGNOSTICS => "PREEMPTIVE_SP_SERVER_DIAGNOSTICS",
            WaitType::PREEMPTIVE_STARTRM => "PREEMPTIVE_STARTRM",
            WaitType::PREEMPTIVE_STREAMFCB_CHECKPOINT => "PREEMPTIVE_STREAMFCB_CHECKPOINT",
            WaitType::PREEMPTIVE_STREAMFCB_RECOVER => "PREEMPTIVE_STREAMFCB_RECOVER",
            WaitType::PREEMPTIVE_STRESSDRIVER => "PREEMPTIVE_STRESSDRIVER",
            WaitType::PREEMPTIVE_TESTING => "PREEMPTIVE_TESTING",
            WaitType::PREEMPTIVE_TRANSIMPORT => "PREEMPTIVE_TRANSIMPORT",
            WaitType::PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN => "PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN",
            WaitType::PREEMPTIVE_VSS_CREATESNAPSHOT => "PREEMPTIVE_VSS_CREATESNAPSHOT",
            WaitType::PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT => "PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT",
            WaitType::PREEMPTIVE_XETESTING => "PREEMPTIVE_XETESTING",
            WaitType::PREEMPTIVE_XE_CALLBACKEXECUTE => "PREEMPTIVE_XE_CALLBACKEXECUTE",
            WaitType::PREEMPTIVE_XE_CX_FILE_OPEN => "PREEMPTIVE_XE_CX_FILE_OPEN",
            WaitType::PREEMPTIVE_XE_CX_HTTP_CALL => "PREEMPTIVE_XE_CX_HTTP_CALL",
            WaitType::PREEMPTIVE_XE_DISPATCHER => "PREEMPTIVE_XE_DISPATCHER",
            WaitType::PREEMPTIVE_XE_ENGINEINIT => "PREEMPTIVE_XE_ENGINEINIT",
            WaitType::PREEMPTIVE_XE_GETTARGETSTATE => "PREEMPTIVE_XE_GETTARGETSTATE",
            WaitType::PREEMPTIVE_XE_SESSIONCOMMIT => "PREEMPTIVE_XE_SESSIONCOMMIT",
            WaitType::PREEMPTIVE_XE_TARGETFINALIZE => "PREEMPTIVE_XE_TARGETFINALIZE",
            WaitType::PREEMPTIVE_XE_TARGETINIT => "PREEMPTIVE_XE_TARGETINIT",
            WaitType::PREEMPTIVE_XE_TIMERRUN => "PREEMPTIVE_XE_TIMERRUN",
            WaitType::PRINT_ROLLBACK_PROGRESS => "PRINT_ROLLBACK_PROGRESS",
            WaitType::PRU_ROLLBACK_DEFERRED => "PRU_ROLLBACK_DEFERRED",
            WaitType::PVS_CLEANUP_LOCK => "PVS_CLEANUP_LOCK",
            WaitType::PWAIT_ALL_COMPONENTS_INITIALIZED => "PWAIT_ALL_COMPONENTS_INITIALIZED",
            WaitType::PWAIT_COOP_SCAN => "PWAIT_COOP_SCAN",
            WaitType::PWAIT_DIRECTLOGCONSUMER_GETNEXT => "PWAIT_DIRECTLOGCONSUMER_GETNEXT",
            WaitType::PWAIT_EVENT_SESSION_INIT_MUTEX => "PWAIT_EVENT_SESSION_INIT_MUTEX",
            WaitType::PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS => "PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS",
            WaitType::PWAIT_HADRSIM => "PWAIT_HADRSIM",
            WaitType::PWAIT_HADR_ACTION_COMPLETED => "PWAIT_HADR_ACTION_COMPLETED",
            WaitType::PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC => "PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC",
            WaitType::PWAIT_HADR_CLUSTER_INTEGRATION => "PWAIT_HADR_CLUSTER_INTEGRATION",
            WaitType::PWAIT_HADR_FAILOVER_COMPLETED => "PWAIT_HADR_FAILOVER_COMPLETED",
            WaitType::PWAIT_HADR_JOIN => "PWAIT_HADR_JOIN",
            WaitType::PWAIT_HADR_OFFLINE_COMPLETED => "PWAIT_HADR_OFFLINE_COMPLETED",
            WaitType::PWAIT_HADR_ONLINE_COMPLETED => "PWAIT_HADR_ONLINE_COMPLETED",
            WaitType::PWAIT_HADR_POST_ONLINE_COMPLETED => "PWAIT_HADR_POST_ONLINE_COMPLETED",
            WaitType::PWAIT_HADR_SERVER_READY_CONNECTIONS => "PWAIT_HADR_SERVER_READY_CONNECTIONS",
            WaitType::PWAIT_HADR_WORKITEM_COMPLETED => "PWAIT_HADR_WORKITEM_COMPLETED",
            WaitType::PWAIT_LOG_CONSOLIDATION_IO => "PWAIT_LOG_CONSOLIDATION_IO",
            WaitType::PWAIT_LOG_CONSOLIDATION_POLL => "PWAIT_LOG_CONSOLIDATION_POLL",
            WaitType::PWAIT_MD_LOGIN_STATS => "PWAIT_MD_LOGIN_STATS",
            WaitType::PWAIT_MD_RELATION_CACHE => "PWAIT_MD_RELATION_CACHE",
            WaitType::PWAIT_MD_SERVER_CACHE => "PWAIT_MD_SERVER_CACHE",
            WaitType::PWAIT_MD_UPGRADE_CONFIG => "PWAIT_MD_UPGRADE_CONFIG",
            WaitType::PWAIT_PREEMPTIVE_APP_USAGE_TIMER => "PWAIT_PREEMPTIVE_APP_USAGE_TIMER",
            WaitType::PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG => "PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG",
            WaitType::PWAIT_QRY_BPMEMORY => "PWAIT_QRY_BPMEMORY",
            WaitType::PWAIT_REPLICA_ONLINE_INIT_MUTEX => "PWAIT_REPLICA_ONLINE_INIT_MUTEX",
            WaitType::PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC => "PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC",
            WaitType::PWAIT_SBS_FILE_OPERATION => "PWAIT_SBS_FILE_OPERATION",
            WaitType::PWAIT_XTP_FSSTORAGE_MAINTENANCE => "PWAIT_XTP_FSSTORAGE_MAINTENANCE",
            WaitType::PWAIT_XTP_HOST_STORAGE_WAIT => "PWAIT_XTP_HOST_STORAGE_WAIT",
            WaitType::QDS_ASYNC_CHECK_CONSISTENCY_TASK => "QDS_ASYNC_CHECK_CONSISTENCY_TASK",
            WaitType::QDS_ASYNC_PERSIST_TASK => "QDS_ASYNC_PERSIST_TASK",
            WaitType::QDS_ASYNC_PERSIST_TASK_START => "QDS_ASYNC_PERSIST_TASK_START",
            WaitType::QDS_ASYNC_QUEUE => "QDS_ASYNC_QUEUE",
            WaitType::QDS_BCKG_TASK => "QDS_BCKG_TASK",
            WaitType::QDS_BLOOM_FILTER => "QDS_BLOOM_FILTER",
            WaitType::QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP => "QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP",
            WaitType::QDS_CTXS => "QDS_CTXS",
            WaitType::QDS_DB_DISK => "QDS_DB_DISK",
            WaitType::QDS_DYN_VECTOR => "QDS_DYN_VECTOR",
            WaitType::QDS_EXCLUSIVE_ACCESS => "QDS_EXCLUSIVE_ACCESS",
            WaitType::QDS_HOST_INIT => "QDS_HOST_INIT",
            WaitType::QDS_LOADDB => "QDS_LOADDB",
            WaitType::QDS_PERSIST_TASK_MAIN_LOOP_SLEEP => "QDS_PERSIST_TASK_MAIN_LOOP_SLEEP",
            WaitType::QDS_QDS_CAPTURE_INIT => "QDS_QDS_CAPTURE_INIT",
            WaitType::QDS_SHUTDOWN_QUEUE => "QDS_SHUTDOWN_QUEUE",
            WaitType::QDS_STMT => "QDS_STMT",
            WaitType::QDS_STMT_DISK => "QDS_STMT_DISK",
            WaitType::QDS_TASK_SHUTDOWN => "QDS_TASK_SHUTDOWN",
            WaitType::QDS_TASK_START => "QDS_TASK_START",
            WaitType::QE_WARN_LIST_SYNC => "QE_WARN_LIST_SYNC",
            WaitType::QPJOB_KILL => "QPJOB_KILL",
            WaitType::QPJOB_WAITFOR_ABORT => "QPJOB_WAITFOR_ABORT",
            WaitType::QRY_MEM_GRANT_INFO_MUTEX => "QRY_MEM_GRANT_INFO_MUTEX",
            WaitType::QRY_PARALLEL_THREAD_MUTEX => "QRY_PARALLEL_THREAD_MUTEX",
            WaitType::QRY_PROFILE_LIST_MUTEX => "QRY_PROFILE_LIST_MUTEX",
            WaitType::QUERY_ERRHDL_SERVICE_DONE => "QUERY_ERRHDL_SERVICE_DONE",
            WaitType::QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN => "QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN",
            WaitType::QUERY_NOTIFICATION_MGR_MUTEX => "QUERY_NOTIFICATION_MGR_MUTEX",
            WaitType::QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX => "QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX",
            WaitType::QUERY_NOTIFICATION_TABLE_MGR_MUTEX => "QUERY_NOTIFICATION_TABLE_MGR_MUTEX",
            WaitType::QUERY_NOTIFICATION_UNITTEST_MUTEX => "QUERY_NOTIFICATION_UNITTEST_MUTEX",
            WaitType::QUERY_OPTIMIZER_PRINT_MUTEX => "QUERY_OPTIMIZER_PRINT_MUTEX",
            WaitType::QUERY_TASK_ENQUEUE_MUTEX => "QUERY_TASK_ENQUEUE_MUTEX",
            WaitType::QUERY_TRACEOUT => "QUERY_TRACEOUT",
            WaitType::QUERY_WAIT_ERRHDL_SERVICE => "QUERY_WAIT_ERRHDL_SERVICE",
            WaitType::RBIO_RG_DESTAGE => "RBIO_RG_DESTAGE",
            WaitType::RBIO_RG_LOCALDESTAGE => "RBIO_RG_LOCALDESTAGE",
            WaitType::RBIO_RG_REPLICA => "RBIO_RG_REPLICA",
            WaitType::RBIO_RG_STORAGE => "RBIO_RG_STORAGE",
            WaitType::RBIO_WAIT_VLF => "RBIO_WAIT_VLF",
            WaitType::RECOVERY_MGR_LOCK => "RECOVERY_MGR_LOCK",
            WaitType::RECOVER_CHANGEDB => "RECOVER_CHANGEDB",
            WaitType::REDO_THREAD_PENDING_WORK => "REDO_THREAD_PENDING_WORK",
            WaitType::REDO_THREAD_SYNC => "REDO_THREAD_SYNC",
            WaitType::REMOTE_BLOCK_IO => "REMOTE_BLOCK_IO",
            WaitType::REMOTE_DATA_ARCHIVE_MIGRATION_DMV => "REMOTE_DATA_ARCHIVE_MIGRATION_DMV",
            WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_DMV => "REMOTE_DATA_ARCHIVE_SCHEMA_DMV",
            WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE => "REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE",
            WaitType::REPLICA_WRITES => "REPLICA_WRITES",
            WaitType::REPL_CACHE_ACCESS => "REPL_CACHE_ACCESS",
            WaitType::REPL_HISTORYCACHE_ACCESS => "REPL_HISTORYCACHE_ACCESS",
            WaitType::REPL_SCHEMA_ACCESS => "REPL_SCHEMA_ACCESS",
            WaitType::REPL_TRANFSINFO_ACCESS => "REPL_TRANFSINFO_ACCESS",
            WaitType::REPL_TRANHASHTABLE_ACCESS => "REPL_TRANHASHTABLE_ACCESS",
            WaitType::REPL_TRANTEXTINFO_ACCESS => "REPL_TRANTEXTINFO_ACCESS",
            WaitType::REQUEST_DISPENSER_PAUSE => "REQUEST_DISPENSER_PAUSE",
            WaitType::REQUEST_FOR_DEADLOCK_SEARCH => "REQUEST_FOR_DEADLOCK_SEARCH",
            WaitType::RESERVED_MEMORY_ALLOCATION_EXT => "RESERVED_MEMORY_ALLOCATION_EXT",
            WaitType::RESMGR_THROTTLED => "RESMGR_THROTTLED",
            WaitType::RESOURCE_GOVERNOR_IDLE => "RESOURCE_GOVERNOR_IDLE",
            WaitType::RESOURCE_QUEUE => "RESOURCE_QUEUE",
            WaitType::RESOURCE_SEMAPHORE => "RESOURCE_SEMAPHORE",
            WaitType::RESOURCE_SEMAPHORE_MUTEX => "RESOURCE_SEMAPHORE_MUTEX",
            WaitType::RESOURCE_SEMAPHORE_QUERY_COMPILE => "RESOURCE_SEMAPHORE_QUERY_COMPILE",
            WaitType::RESOURCE_SEMAPHORE_SMALL_QUERY => "RESOURCE_SEMAPHORE_SMALL_QUERY",
            WaitType::RESTORE_FILEHANDLECACHE_ENTRYLOCK => "RESTORE_FILEHANDLECACHE_ENTRYLOCK",
            WaitType::RESTORE_FILEHANDLECACHE_LOCK => "RESTORE_FILEHANDLECACHE_LOCK",
            WaitType::RG_RECONFIG => "RG_RECONFIG",
            WaitType::ROWGROUP_OP_STATS => "ROWGROUP_OP_STATS",
            WaitType::ROWGROUP_VERSION => "ROWGROUP_VERSION",
            WaitType::RTDATA_LIST => "RTDATA_LIST",
            WaitType::SATELLITE_CARGO => "SATELLITE_CARGO",
            WaitType::SATELLITE_SERVICE_SETUP => "SATELLITE_SERVICE_SETUP",
            WaitType::SATELLITE_TASK => "SATELLITE_TASK",
            WaitType::SBS_DISPATCH => "SBS_DISPATCH",
            WaitType::SBS_RECEIVE_TRANSPORT => "SBS_RECEIVE_TRANSPORT",
            WaitType::SBS_TRANSPORT => "SBS_TRANSPORT",
            WaitType::SCAN_CHAR_HASH_ARRAY_INITIALIZATION => "SCAN_CHAR_HASH_ARRAY_INITIALIZATION",
            WaitType::SECURITY_CNG_PROVIDER_MUTEX => "SECURITY_CNG_PROVIDER_MUTEX",
            WaitType::SECURITY_CRYPTO_CONTEXT_MUTEX => "SECURITY_CRYPTO_CONTEXT_MUTEX",
            WaitType::SECURITY_DBE_STATE_MUTEX => "SECURITY_DBE_STATE_MUTEX",
            WaitType::SECURITY_KEYRING_RWLOCK => "SECURITY_KEYRING_RWLOCK",
            WaitType::SECURITY_MUTEX => "SECURITY_MUTEX",
            WaitType::SECURITY_RULETABLE_MUTEX => "SECURITY_RULETABLE_MUTEX",
            WaitType::SEC_DROP_TEMP_KEY => "SEC_DROP_TEMP_KEY",
            WaitType::SEMPLAT_DSI_BUILD => "SEMPLAT_DSI_BUILD",
            WaitType::SEQUENCE_GENERATION => "SEQUENCE_GENERATION",
            WaitType::SEQUENTIAL_GUID => "SEQUENTIAL_GUID",
            WaitType::SERVER_IDLE_CHECK => "SERVER_IDLE_CHECK",
            WaitType::SERVER_RECONFIGURE => "SERVER_RECONFIGURE",
            WaitType::SESSION_WAIT_STATS_CHILDREN => "SESSION_WAIT_STATS_CHILDREN",
            WaitType::SHARED_DELTASTORE_CREATION => "SHARED_DELTASTORE_CREATION",
            WaitType::SHUTDOWN => "SHUTDOWN",
            WaitType::SLEEP_BPOOL_FLUSH => "SLEEP_BPOOL_FLUSH",
            WaitType::SLEEP_BUFFERPOOL_HELPLW => "SLEEP_BUFFERPOOL_HELPLW",
            WaitType::SLEEP_DBSTARTUP => "SLEEP_DBSTARTUP",
            WaitType::SLEEP_DCOMSTARTUP => "SLEEP_DCOMSTARTUP",
            WaitType::SLEEP_MASTERDBREADY => "SLEEP_MASTERDBREADY",
            WaitType::SLEEP_MASTERMDREADY => "SLEEP_MASTERMDREADY",
            WaitType::SLEEP_MASTERUPGRADED => "SLEEP_MASTERUPGRADED",
            WaitType::SLEEP_MEMORYPOOL_ALLOCATEPAGES => "SLEEP_MEMORYPOOL_ALLOCATEPAGES",
            WaitType::SLEEP_MSDBSTARTUP => "SLEEP_MSDBSTARTUP",
            WaitType::SLEEP_RETRY_VIRTUALALLOC => "SLEEP_RETRY_VIRTUALALLOC",
            WaitType::SLEEP_SYSTEMTASK => "SLEEP_SYSTEMTASK",
            WaitType::SLEEP_TASK => "SLEEP_TASK",
            WaitType::SLEEP_TEMPDBSTARTUP => "SLEEP_TEMPDBSTARTUP",
            WaitType::SLEEP_WORKSPACE_ALLOCATEPAGE => "SLEEP_WORKSPACE_ALLOCATEPAGE",
            WaitType::SLO_UPDATE => "SLO_UPDATE",
            WaitType::SMSYNC => "SMSYNC",
            WaitType::SNI_CONN_DUP => "SNI_CONN_DUP",
            WaitType::SNI_CRITICAL_SECTION => "SNI_CRITICAL_SECTION",
            WaitType::SNI_HTTP_WAITFOR_0_DISCON => "SNI_HTTP_WAITFOR_0_DISCON",
            WaitType::SNI_LISTENER_ACCESS => "SNI_LISTENER_ACCESS",
            WaitType::SNI_TASK_COMPLETION => "SNI_TASK_COMPLETION",
            WaitType::SNI_WRITE_ASYNC => "SNI_WRITE_ASYNC",
            WaitType::SOAP_READ => "SOAP_READ",
            WaitType::SOAP_WRITE => "SOAP_WRITE",
            WaitType::SOCKETDUPLICATEQUEUE_CLEANUP => "SOCKETDUPLICATEQUEUE_CLEANUP",
            WaitType::SOSHOST_EVENT => "SOSHOST_EVENT",
            WaitType::SOSHOST_INTERNAL => "SOSHOST_INTERNAL",
            WaitType::SOSHOST_MUTEX => "SOSHOST_MUTEX",
            WaitType::SOSHOST_RWLOCK => "SOSHOST_RWLOCK",
            WaitType::SOSHOST_SEMAPHORE => "SOSHOST_SEMAPHORE",
            WaitType::SOSHOST_SLEEP => "SOSHOST_SLEEP",
            WaitType::SOSHOST_TRACELOCK => "SOSHOST_TRACELOCK",
            WaitType::SOSHOST_WAITFORDONE => "SOSHOST_WAITFORDONE",
            WaitType::SOS_CALLBACK_REMOVAL => "SOS_CALLBACK_REMOVAL",
            WaitType::SOS_DISPATCHER_MUTEX => "SOS_DISPATCHER_MUTEX",
            WaitType::SOS_LOCALALLOCATORLIST => "SOS_LOCALALLOCATORLIST",
            WaitType::SOS_MEMORY_TOPLEVELBLOCKALLOCATOR => "SOS_MEMORY_TOPLEVELBLOCKALLOCATOR",
            WaitType::SOS_MEMORY_USAGE_ADJUSTMENT => "SOS_MEMORY_USAGE_ADJUSTMENT",
            WaitType::SOS_OBJECT_STORE_DESTROY_MUTEX => "SOS_OBJECT_STORE_DESTROY_MUTEX",
            WaitType::SOS_PHYS_PAGE_CACHE => "SOS_PHYS_PAGE_CACHE",
            WaitType::SOS_PROCESS_AFFINITY_MUTEX => "SOS_PROCESS_AFFINITY_MUTEX",
            WaitType::SOS_RESERVEDMEMBLOCKLIST => "SOS_RESERVEDMEMBLOCKLIST",
            WaitType::SOS_SCHEDULER_YIELD => "SOS_SCHEDULER_YIELD",
            WaitType::SOS_SMALL_PAGE_ALLOC => "SOS_SMALL_PAGE_ALLOC",
            WaitType::SOS_STACKSTORE_INIT_MUTEX => "SOS_STACKSTORE_INIT_MUTEX",
            WaitType::SOS_SYNC_TASK_ENQUEUE_EVENT => "SOS_SYNC_TASK_ENQUEUE_EVENT",
            WaitType::SOS_VIRTUALMEMORY_LOW => "SOS_VIRTUALMEMORY_LOW",
            WaitType::SOS_WORK_DISPATCHER => "SOS_WORK_DISPATCHER",
            WaitType::SPINLOCK_EXT => "SPINLOCK_EXT",
            WaitType::SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP => "SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP",
            WaitType::SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS => "SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS",
            WaitType::SP_SERVER_DIAGNOSTICS_INIT_MUTEX => "SP_SERVER_DIAGNOSTICS_INIT_MUTEX",
            WaitType::SP_SERVER_DIAGNOSTICS_SLEEP => "SP_SERVER_DIAGNOSTICS_SLEEP",
            WaitType::SQLCLR_APPDOMAIN => "SQLCLR_APPDOMAIN",
            WaitType::SQLCLR_ASSEMBLY => "SQLCLR_ASSEMBLY",
            WaitType::SQLCLR_DEADLOCK_DETECTION => "SQLCLR_DEADLOCK_DETECTION",
            WaitType::SQLCLR_QUANTUM_PUNISHMENT => "SQLCLR_QUANTUM_PUNISHMENT",
            WaitType::SQLSORT_NORMMUTEX => "SQLSORT_NORMMUTEX",
            WaitType::SQLSORT_SORTMUTEX => "SQLSORT_SORTMUTEX",
            WaitType::SQLTRACE_BUFFER_FLUSH => "SQLTRACE_BUFFER_FLUSH",
            WaitType::SQLTRACE_FILE_BUFFER => "SQLTRACE_FILE_BUFFER",
            WaitType::SQLTRACE_FILE_READ_IO_COMPLETION => "SQLTRACE_FILE_READ_IO_COMPLETION",
            WaitType::SQLTRACE_FILE_WRITE_IO_COMPLETION => "SQLTRACE_FILE_WRITE_IO_COMPLETION",
            WaitType::SQLTRACE_INCREMENTAL_FLUSH_SLEEP => "SQLTRACE_INCREMENTAL_FLUSH_SLEEP",
            WaitType::SQLTRACE_LOCK => "SQLTRACE_LOCK",
            WaitType::SQLTRACE_PENDING_BUFFER_WRITERS => "SQLTRACE_PENDING_BUFFER_WRITERS",
            WaitType::SQLTRACE_SHUTDOWN => "SQLTRACE_SHUTDOWN",
            WaitType::SQLTRACE_WAIT_ENTRIES => "SQLTRACE_WAIT_ENTRIES",
            WaitType::SRVPROC_SHUTDOWN => "SRVPROC_SHUTDOWN",
            WaitType::STARTUP_DEPENDENCY_MANAGER => "STARTUP_DEPENDENCY_MANAGER",
            WaitType::TDS_BANDWIDTH_STATE => "TDS_BANDWIDTH_STATE",
            WaitType::TDS_INIT => "TDS_INIT",
            WaitType::TDS_PROXY_CONTAINER => "TDS_PROXY_CONTAINER",
            WaitType::TEMPOBJ => "TEMPOBJ",
            WaitType::TEMPORAL_BACKGROUND_PROCEED_CLEANUP => "TEMPORAL_BACKGROUND_PROCEED_CLEANUP",
            WaitType::TERMINATE_LISTENER => "TERMINATE_LISTENER",
            WaitType::THREADPOOL => "THREADPOOL",
            WaitType::TIMEPRIV_TIMEPERIOD => "TIMEPRIV_TIMEPERIOD",
            WaitType::TRACEWRITE => "TRACEWRITE",
            WaitType::TRACE_EVTNOTIF => "TRACE_EVTNOTIF",
            WaitType::TRANSACTION_MUTEX => "TRANSACTION_MUTEX",
            WaitType::TRAN_MARKLATCH_DT => "TRAN_MARKLATCH_DT",
            WaitType::TRAN_MARKLATCH_EX => "TRAN_MARKLATCH_EX",
            WaitType::TRAN_MARKLATCH_KP => "TRAN_MARKLATCH_KP",
            WaitType::TRAN_MARKLATCH_NL => "TRAN_MARKLATCH_NL",
            WaitType::TRAN_MARKLATCH_SH => "TRAN_MARKLATCH_SH",
            WaitType::TRAN_MARKLATCH_UP => "TRAN_MARKLATCH_UP",
            WaitType::UCS_ENDPOINT_CHANGE => "UCS_ENDPOINT_CHANGE",
            WaitType::UCS_MANAGER => "UCS_MANAGER",
            WaitType::UCS_MEMORY_NOTIFICATION => "UCS_MEMORY_NOTIFICATION",
            WaitType::UCS_SESSION_REGISTRATION => "UCS_SESSION_REGISTRATION",
            WaitType::UCS_TRANSPORT => "UCS_TRANSPORT",
            WaitType::UCS_TRANSPORT_STREAM_CHANGE => "UCS_TRANSPORT_STREAM_CHANGE",
            WaitType::UTIL_PAGE_ALLOC => "UTIL_PAGE_ALLOC",
            WaitType::VDI_CLIENT_COMPLETECOMMAND => "VDI_CLIENT_COMPLETECOMMAND",
            WaitType::VDI_CLIENT_GETCOMMAND => "VDI_CLIENT_GETCOMMAND",
            WaitType::VDI_CLIENT_OPERATION => "VDI_CLIENT_OPERATION",
            WaitType::VDI_CLIENT_OTHER => "VDI_CLIENT_OTHER",
            WaitType::VERSIONING_COMMITTING => "VERSIONING_COMMITTING",
            WaitType::VIA_ACCEPT => "VIA_ACCEPT",
            WaitType::VIEW_DEFINITION_MUTEX => "VIEW_DEFINITION_MUTEX",
            WaitType::WAITFOR => "WAITFOR",
            WaitType::WAITFOR_PER_QUEUE => "WAITFOR_PER_QUEUE",
            WaitType::WAITFOR_TASKSHUTDOWN => "WAITFOR_TASKSHUTDOWN",
            WaitType::WAITSTAT_MUTEX => "WAITSTAT_MUTEX",
            WaitType::WAIT_FOR_RESULTS => "WAIT_FOR_RESULTS",
            WaitType::WAIT_ON_SYNC_STATISTICS_REFRESH => "WAIT_ON_SYNC_STATISTICS_REFRESH",
            WaitType::WAIT_SCRIPTDEPLOYMENT_REQUEST => "WAIT_SCRIPTDEPLOYMENT_REQUEST",
            WaitType::WAIT_SCRIPTDEPLOYMENT_WORKER => "WAIT_SCRIPTDEPLOYMENT_WORKER",
            WaitType::WAIT_XLOGREAD_SIGNAL => "WAIT_XLOGREAD_SIGNAL",
            WaitType::WAIT_XTP_ASYNC_TX_COMPLETION => "WAIT_XTP_ASYNC_TX_COMPLETION",
            WaitType::WAIT_XTP_CKPT_AGENT_WAKEUP => "WAIT_XTP_CKPT_AGENT_WAKEUP",
            WaitType::WAIT_XTP_CKPT_CLOSE => "WAIT_XTP_CKPT_CLOSE",
            WaitType::WAIT_XTP_CKPT_ENABLED => "WAIT_XTP_CKPT_ENABLED",
            WaitType::WAIT_XTP_CKPT_STATE_LOCK => "WAIT_XTP_CKPT_STATE_LOCK",
            WaitType::WAIT_XTP_COMPILE_WAIT => "WAIT_XTP_COMPILE_WAIT",
            WaitType::WAIT_XTP_GUEST => "WAIT_XTP_GUEST",
            WaitType::WAIT_XTP_HOST_WAIT => "WAIT_XTP_HOST_WAIT",
            WaitType::WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO => "WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO",
            WaitType::WAIT_XTP_OFFLINE_CKPT_LOG_IO => "WAIT_XTP_OFFLINE_CKPT_LOG_IO",
            WaitType::WAIT_XTP_OFFLINE_CKPT_NEW_LOG => "WAIT_XTP_OFFLINE_CKPT_NEW_LOG",
            WaitType::WAIT_XTP_PROCEDURE_ENTRY => "WAIT_XTP_PROCEDURE_ENTRY",
            WaitType::WAIT_XTP_RECOVERY => "WAIT_XTP_RECOVERY",
            WaitType::WAIT_XTP_SERIAL_RECOVERY => "WAIT_XTP_SERIAL_RECOVERY",
            WaitType::WAIT_XTP_SWITCH_TO_INACTIVE => "WAIT_XTP_SWITCH_TO_INACTIVE",
            WaitType::WAIT_XTP_TASK_SHUTDOWN => "WAIT_XTP_TASK_SHUTDOWN",
            WaitType::WAIT_XTP_TRAN_DEPENDENCY => "WAIT_XTP_TRAN_DEPENDENCY",
            WaitType::WCC => "WCC",
            WaitType::WINDOW_AGGREGATES_MULTIPASS => "WINDOW_AGGREGATES_MULTIPASS",
            WaitType::WINFAB_API_CALL => "WINFAB_API_CALL",
            WaitType::WINFAB_REPLICA_BUILD_OPERATION => "WINFAB_REPLICA_BUILD_OPERATION",
            WaitType::WINFAB_REPORT_FAULT => "WINFAB_REPORT_FAULT",
            WaitType::WORKTBL_DROP => "WORKTBL_DROP",
            WaitType::WRITELOG => "WRITELOG",
            WaitType::WRITE_COMPLETION => "WRITE_COMPLETION",
            WaitType::XACTLOCKINFO => "XACTLOCKINFO",
            WaitType::XACTWORKSPACE_MUTEX => "XACTWORKSPACE_MUTEX",
            WaitType::XACT_OWN_TRANSACTION => "XACT_OWN_TRANSACTION",
            WaitType::XACT_RECLAIM_SESSION => "XACT_RECLAIM_SESSION",
            WaitType::XDB_CONN_DUP_HASH => "XDB_CONN_DUP_HASH",
            WaitType::XDESTSVERMGR => "XDESTSVERMGR",
            WaitType::XDES_HISTORY => "XDES_HISTORY",
            WaitType::XDES_OUT_OF_ORDER_LIST => "XDES_OUT_OF_ORDER_LIST",
            WaitType::XDES_SNAPSHOT => "XDES_SNAPSHOT",
            WaitType::XE_BUFFERMGR_ALLPROCESSED_EVENT => "XE_BUFFERMGR_ALLPROCESSED_EVENT",
            WaitType::XE_BUFFERMGR_FREEBUF_EVENT => "XE_BUFFERMGR_FREEBUF_EVENT",
            WaitType::XE_CALLBACK_LIST => "XE_CALLBACK_LIST",
            WaitType::XE_CX_FILE_READ => "XE_CX_FILE_READ",
            WaitType::XE_DISPATCHER_CONFIG_SESSION_LIST => "XE_DISPATCHER_CONFIG_SESSION_LIST",
            WaitType::XE_DISPATCHER_JOIN => "XE_DISPATCHER_JOIN",
            WaitType::XE_DISPATCHER_WAIT => "XE_DISPATCHER_WAIT",
            WaitType::XE_FILE_TARGET_TVF => "XE_FILE_TARGET_TVF",
            WaitType::XE_LIVE_TARGET_TVF => "XE_LIVE_TARGET_TVF",
            WaitType::XE_MODULEMGR_SYNC => "XE_MODULEMGR_SYNC",
            WaitType::XE_OLS_LOCK => "XE_OLS_LOCK",
            WaitType::XE_PACKAGE_LOCK_BACKOFF => "XE_PACKAGE_LOCK_BACKOFF",
            WaitType::XE_SERVICES_EVENTMANUAL => "XE_SERVICES_EVENTMANUAL",
            WaitType::XE_SERVICES_MUTEX => "XE_SERVICES_MUTEX",
            WaitType::XE_SERVICES_RWLOCK => "XE_SERVICES_RWLOCK",
            WaitType::XE_SESSION_CREATE_SYNC => "XE_SESSION_CREATE_SYNC",
            WaitType::XE_SESSION_FLUSH => "XE_SESSION_FLUSH",
            WaitType::XE_SESSION_SYNC => "XE_SESSION_SYNC",
            WaitType::XE_STM_CREATE => "XE_STM_CREATE",
            WaitType::XE_TIMER_EVENT => "XE_TIMER_EVENT",
            WaitType::XE_TIMER_MUTEX => "XE_TIMER_MUTEX",
            WaitType::XE_TIMER_TASK_DONE => "XE_TIMER_TASK_DONE",
            WaitType::XIO_CREDENTIAL_MGR_RWLOCK => "XIO_CREDENTIAL_MGR_RWLOCK",
            WaitType::XIO_CREDENTIAL_RWLOCK => "XIO_CREDENTIAL_RWLOCK",
            WaitType::XIO_EDS_MGR_RWLOCK => "XIO_EDS_MGR_RWLOCK",
            WaitType::XIO_EDS_RWLOCK => "XIO_EDS_RWLOCK",
            WaitType::XIO_IOSTATS_BLOBLIST_RWLOCK => "XIO_IOSTATS_BLOBLIST_RWLOCK",
            WaitType::XIO_IOSTATS_FCBLIST_RWLOCK => "XIO_IOSTATS_FCBLIST_RWLOCK",
            WaitType::XIO_LEASE_RENEW_MGR_RWLOCK => "XIO_LEASE_RENEW_MGR_RWLOCK",
            WaitType::XTPPROC_CACHE_ACCESS => "XTPPROC_CACHE_ACCESS",
            WaitType::XTPPROC_PARTITIONED_STACK_CREATE => "XTPPROC_PARTITIONED_STACK_CREATE",
            WaitType::XTP_HOST_DB_COLLECTION => "XTP_HOST_DB_COLLECTION",
            WaitType::XTP_HOST_LOG_ACTIVITY => "XTP_HOST_LOG_ACTIVITY",
            WaitType::XTP_HOST_PARALLEL_RECOVERY => "XTP_HOST_PARALLEL_RECOVERY",
            WaitType::XTP_PREEMPTIVE_TASK => "XTP_PREEMPTIVE_TASK",
            WaitType::XTP_TRUNCATION_LSN => "XTP_TRUNCATION_LSN",
            WaitType::Unknown(s) => s,
        }
    }

    /// Verbatim description from the SQL Server docs table (including
    /// the "Applies to" note where the table has one).
    pub fn description(&self) -> &'static str {
        match self {
            WaitType::ABR | WaitType::CHECK_PRINT_RECORD | WaitType::DBMIRROR_DBM_EVENT | WaitType::DBMIRROR_DBM_MUTEX | WaitType::DUMPTRIGGER | WaitType::EC | WaitType::FAILPOINT | WaitType::GUARDIAN | WaitType::INTERNAL_TESTING | WaitType::IOAFF_RANGE_QUEUE | WaitType::KTM_ENLISTMENT | WaitType::KTM_RECOVERY_MANAGER | WaitType::KTM_RECOVERY_RESOLUTION | WaitType::LATCH_NL | WaitType::LOGGENERATION | WaitType::LOGMGR_FLUSH | WaitType::MISCELLANEOUS | WaitType::PAGEIOLATCH_NL | WaitType::PAGELATCH_NL | WaitType::PREEMPTIVE_ABR | WaitType::PREEMPTIVE_SOSTESTING | WaitType::PREEMPTIVE_STRESSDRIVER | WaitType::PREEMPTIVE_TESTING | WaitType::PREEMPTIVE_XETESTING | WaitType::QUERY_NOTIFICATION_UNITTEST_MUTEX | WaitType::QUERY_TRACEOUT | WaitType::TRAN_MARKLATCH_NL | WaitType::WAITFOR_TASKSHUTDOWN | WaitType::WCC | WaitType::XE_MODULEMGR_SYNC | WaitType::XE_OLS_LOCK => "Identified for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::AM_INDBUILD_ALLOCATION | WaitType::AM_SCHEMAMGR_UNSHARED_CACHE | WaitType::BROKER_DISPATCHER | WaitType::BROKER_FORWARDER | WaitType::BROKER_TRANSMISSION_OBJECT | WaitType::BROKER_TRANSMISSION_TABLE | WaitType::BROKER_TRANSMISSION_WORK | WaitType::CHANGE_TRACKING_WAITFORCHANGES | WaitType::COUNTRECOVERYMGR | WaitType::CREATE_DATINISERVICE | WaitType::DBCC_SCALE_OUT_EXPR_CACHE | WaitType::DIRTY_PAGE_POLL | WaitType::DIRTY_PAGE_SYNC | WaitType::DISPATCHER_PRIORITY_QUEUE_SEMAPHORE | WaitType::DTCPNTSYNC | WaitType::ENABLE_EMPTY_VERSIONING | WaitType::FFT_NSO_DB_KILL_FLAG | WaitType::FFT_NSO_DB_LIST | WaitType::FFT_NSO_FCB | WaitType::FFT_NSO_FCB_FIND | WaitType::FFT_NSO_FCB_PARENT | WaitType::FFT_NSO_FCB_RELEASE_CACHED_ENTRIES | WaitType::FFT_NSO_FILEOBJECT | WaitType::FFT_NSO_TABLE_LIST | WaitType::FFT_NTFS_STORE | WaitType::FFT_RECOVERY | WaitType::FFT_RSFX_COMM | WaitType::FFT_RSFX_WAIT_FOR_MEMORY | WaitType::FFT_STARTUP_SHUTDOWN | WaitType::FFT_STORE_DB | WaitType::FFT_STORE_ROWSET_LIST | WaitType::FFT_STORE_TABLE | WaitType::FILESTREAM_CACHE | WaitType::FILESTREAM_CHUNKER | WaitType::FILESTREAM_CHUNKER_INIT | WaitType::FILESTREAM_FCB | WaitType::FILESTREAM_FILE_OBJECT | WaitType::FILESTREAM_WORKITEM_QUEUE | WaitType::FILETABLE_SHUTDOWN | WaitType::FT_MASTER_MERGE_COORDINATOR | WaitType::FT_PROPERTYLIST_CACHE | WaitType::GDMA_GET_RESOURCE_OWNER | WaitType::GHOSTCLEANUPSYNCMGR | WaitType::HADR_AR_UNLOAD_COMPLETED | WaitType::HADR_CONNECTIVITY_INFO | WaitType::LOGCAPTURE_LOGPOOLTRUNCPOINT | WaitType::LOGPOOLREFCOUNTEDOBJECT_REFDONE | WaitType::LOGPOOL_CACHESIZE | WaitType::LOGPOOL_CONSUMER | WaitType::LOGPOOL_CONSUMERSET | WaitType::LOGPOOL_FREEPOOLS | WaitType::LOGPOOL_MGRSET | WaitType::LOGPOOL_REPLACEMENTSET | WaitType::MD_AGENT_YIELD | WaitType::MD_LAZYCACHE_RWLOCK | WaitType::PREEMPTIVE_OS_PDH_WMI_INIT | WaitType::PREEMPTIVE_OS_QUERYCONTEXTATTRIBUTES | WaitType::PREEMPTIVE_SP_SERVER_DIAGNOSTICS | WaitType::PRU_ROLLBACK_DEFERRED | WaitType::PWAIT_ALL_COMPONENTS_INITIALIZED | WaitType::PWAIT_COOP_SCAN | WaitType::PWAIT_EVENT_SESSION_INIT_MUTEX | WaitType::PWAIT_HADR_ACTION_COMPLETED | WaitType::PWAIT_HADR_FAILOVER_COMPLETED | WaitType::PWAIT_HADR_SERVER_READY_CONNECTIONS | WaitType::PWAIT_PREEMPTIVE_AUDIT_ACCESS_WINDOWSLOG | WaitType::PWAIT_QRY_BPMEMORY | WaitType::PWAIT_REPLICA_ONLINE_INIT_MUTEX | WaitType::PWAIT_RESOURCE_SEMAPHORE_FT_PARALLEL_QUERY_SYNC | WaitType::QRY_PARALLEL_THREAD_MUTEX | WaitType::QUERY_TASK_ENQUEUE_MUTEX | WaitType::REDO_THREAD_PENDING_WORK | WaitType::REDO_THREAD_SYNC | WaitType::REPL_TRANFSINFO_ACCESS | WaitType::REPL_TRANTEXTINFO_ACCESS | WaitType::RESOURCE_GOVERNOR_IDLE | WaitType::SCAN_CHAR_HASH_ARRAY_INITIALIZATION | WaitType::SECURITY_CRYPTO_CONTEXT_MUTEX | WaitType::SECURITY_KEYRING_RWLOCK | WaitType::SECURITY_RULETABLE_MUTEX | WaitType::SEMPLAT_DSI_BUILD | WaitType::SEQUENCE_GENERATION | WaitType::SERVER_RECONFIGURE | WaitType::SLEEP_MASTERDBREADY | WaitType::SLEEP_MASTERMDREADY | WaitType::SLEEP_MASTERUPGRADED | WaitType::SOS_MEMORY_TOPLEVELBLOCKALLOCATOR | WaitType::SP_PREEMPTIVE_SERVER_DIAGNOSTICS_SLEEP | WaitType::SP_SERVER_DIAGNOSTICS_BUFFER_ACCESS | WaitType::SP_SERVER_DIAGNOSTICS_INIT_MUTEX | WaitType::SP_SERVER_DIAGNOSTICS_SLEEP | WaitType::SQLTRACE_FILE_READ_IO_COMPLETION | WaitType::SQLTRACE_FILE_WRITE_IO_COMPLETION | WaitType::SQLTRACE_INCREMENTAL_FLUSH_SLEEP | WaitType::SQLTRACE_PENDING_BUFFER_WRITERS | WaitType::STARTUP_DEPENDENCY_MANAGER | WaitType::TERMINATE_LISTENER | WaitType::UCS_ENDPOINT_CHANGE | WaitType::UCS_MANAGER | WaitType::UCS_MEMORY_NOTIFICATION | WaitType::UCS_SESSION_REGISTRATION | WaitType::UCS_TRANSPORT | WaitType::UCS_TRANSPORT_STREAM_CHANGE | WaitType::VERSIONING_COMMITTING | WaitType::WAITFOR_PER_QUEUE | WaitType::XDESTSVERMGR | WaitType::XDES_HISTORY | WaitType::XDES_OUT_OF_ORDER_LIST | WaitType::XDES_SNAPSHOT | WaitType::XE_CALLBACK_LIST | WaitType::XE_LIVE_TARGET_TVF => "Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::ASSEMBLY_FILTER_HASHTABLE | WaitType::ASYNC_SOCKETDUP_IO | WaitType::BLOB_METADATA | WaitType::BROKER_START | WaitType::CHECK_SCANNER_MUTEX | WaitType::CHECK_TABLES_INITIALIZATION | WaitType::CHECK_TABLES_SINGLE_SCAN | WaitType::CHECK_TABLES_THREAD_BARRIER | WaitType::COLUMNSTORE_COLUMNDATASET_SESSION_LIST | WaitType::CONNECTION_ENDPOINT_LOCK | WaitType::DIRECTLOGCONSUMER_LIST | WaitType::DIRTY_PAGE_TABLE_LOCK | WaitType::DPT_ENTRY_LOCK | WaitType::DTCNEW_ENLIST | WaitType::DTCNEW_PREPARE | WaitType::DTCNEW_RECOVERY | WaitType::DTCNEW_TM | WaitType::DTCNEW_TRANSACTION_ENLISTMENT | WaitType::EXTERNAL_RG_UPDATE | WaitType::EXTERNAL_SCRIPT_PREPARE_SERVICE | WaitType::EXTERNAL_SCRIPT_SHUTDOWN | WaitType::EXTERNAL_WAIT_ON_LAUNCHER | WaitType::FILE_VALIDATION_THREADS | WaitType::FORWARDER_TRANSITION | WaitType::GLOBAL_QUERY_CLOSE | WaitType::GLOBAL_TRAN_CREATE | WaitType::GLOBAL_TRAN_UCS_SESSION | WaitType::HADR_DATABASE_WAIT_FOR_RECOVERY | WaitType::HADR_FILESTREAM_PREPROC | WaitType::HADR_SEEDING_CANCELLATION | WaitType::HADR_SEEDING_FILE_LIST | WaitType::HADR_SEEDING_LIMIT_BACKUPS | WaitType::HADR_SEEDING_SYNC_COMPLETION | WaitType::HADR_SEEDING_TIMEOUT_TASK | WaitType::HADR_SEEDING_WAIT_FOR_COMPLETION | WaitType::HADR_THROTTLE_LOG_RATE_GOVERNOR | WaitType::HCCO_CACHE | WaitType::HKCS_PARALLEL_MIGRATION | WaitType::HKCS_PARALLEL_RECOVERY | WaitType::HK_RESTORE_FILEMAP | WaitType::INSTANCE_LOG_RATE_GOVERNOR | WaitType::IO_QUEUE_LIMIT | WaitType::LOGMGR_PMM_LOG | WaitType::LOG_POOL_SCAN | WaitType::LOG_RATE_GOVERNOR | WaitType::NETWORKSXMLMGRLOAD | WaitType::PARALLEL_REDO_DRAIN_WORKER | WaitType::PARALLEL_REDO_FLOW_CONTROL | WaitType::PARALLEL_REDO_LOG_CACHE | WaitType::PARALLEL_REDO_TRAN_LIST | WaitType::PARALLEL_REDO_TRAN_TURN | WaitType::PARALLEL_REDO_WORKER_SYNC | WaitType::PARALLEL_REDO_WORKER_WAIT_WORK | WaitType::POOL_LOG_RATE_GOVERNOR | WaitType::PREEMPTIVE_HTTP_EVENT_WAIT | WaitType::PREEMPTIVE_HTTP_REQUEST | WaitType::PREEMPTIVE_OS_GETFINALFILEPATHBYHANDLE | WaitType::PREEMPTIVE_OS_VERIFYTRUST | WaitType::PWAIT_DIRECTLOGCONSUMER_GETNEXT | WaitType::PWAIT_FABRIC_REPLICA_CONTROLLER_DATA_LOSS | WaitType::PWAIT_HADRSIM | WaitType::QDS_ASYNC_QUEUE | WaitType::QDS_BLOOM_FILTER | WaitType::QDS_EXCLUSIVE_ACCESS | WaitType::QDS_QDS_CAPTURE_INIT | WaitType::QRY_PROFILE_LIST_MUTEX | WaitType::RECOVERY_MGR_LOCK | WaitType::REMOTE_DATA_ARCHIVE_MIGRATION_DMV | WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_DMV | WaitType::REMOTE_DATA_ARCHIVE_SCHEMA_TASK_QUEUE | WaitType::RESERVED_MEMORY_ALLOCATION_EXT | WaitType::RESTORE_FILEHANDLECACHE_ENTRYLOCK | WaitType::RESTORE_FILEHANDLECACHE_LOCK | WaitType::ROWGROUP_OP_STATS | WaitType::ROWGROUP_VERSION | WaitType::SATELLITE_CARGO | WaitType::SATELLITE_SERVICE_SETUP | WaitType::SATELLITE_TASK | WaitType::SECURITY_DBE_STATE_MUTEX | WaitType::SESSION_WAIT_STATS_CHILDREN | WaitType::SHARED_DELTASTORE_CREATION | WaitType::SLEEP_BUFFERPOOL_HELPLW | WaitType::SLEEP_MEMORYPOOL_ALLOCATEPAGES | WaitType::SLEEP_RETRY_VIRTUALALLOC | WaitType::SLEEP_WORKSPACE_ALLOCATEPAGE | WaitType::SMSYNC | WaitType::SOCKETDUPLICATEQUEUE_CLEANUP | WaitType::TDS_BANDWIDTH_STATE | WaitType::TDS_INIT | WaitType::TDS_PROXY_CONTAINER | WaitType::WAIT_XTP_SWITCH_TO_INACTIVE | WaitType::WINDOW_AGGREGATES_MULTIPASS | WaitType::WINFAB_REPORT_FAULT | WaitType::XDB_CONN_DUP_HASH | WaitType::XE_FILE_TARGET_TVF | WaitType::XIO_CREDENTIAL_MGR_RWLOCK | WaitType::XIO_CREDENTIAL_RWLOCK | WaitType::XIO_LEASE_RENEW_MGR_RWLOCK | WaitType::XTP_HOST_PARALLEL_RECOVERY | WaitType::XTP_PREEMPTIVE_TASK | WaitType::XTP_TRUNCATION_LSN => "Internal use only.\n\nApplies to: SQL Server 2016 (13.x) and later versions.",
            WaitType::ASSEMBLY_LOAD => "Occurs during exclusive access to assembly loading.",
            WaitType::ASYNC_DISKPOOL_LOCK => "Occurs when there's an attempt to synchronize parallel threads that are performing tasks such as creating or initializing a file.",
            WaitType::ASYNC_IO_COMPLETION => "Occurs when a task is waiting for asynchronous non-data I/Os to finish. Examples include I/O involved in warm standby log shipping, database mirroring, some bulk import related operations.",
            WaitType::ASYNC_NETWORK_IO => "Occurs on network writes when the task is blocked waiting for the client application to acknowledge that it has processed all the data sent to it. Verify that the client application is processing data from the server as fast as possible or that no network delays exist. Reasons the client application can't consume data fast enough include: application design issues like writing results to a file while the results arrive, waiting for user input, client-side filtering on a large dataset instead of server-side filtering, or an intentional wait introduced. Also the client computer might be experiencing slow response due to issues like low virtual/physical memory, 100% CPU consumption, etc. Network delays can also lead to this wait - typically caused by network adapter driver issues, filter drivers, firewalls, or misconfigured routers.",
            WaitType::ASYNC_OP_COMPLETION | WaitType::ASYNC_OP_CONTEXT_READ | WaitType::ASYNC_OP_CONTEXT_WRITE | WaitType::BROKER_TASK_SHUTDOWN | WaitType::BROKER_TASK_SUBMIT | WaitType::CMEMPARTITIONED | WaitType::COLUMNSTORE_BUILD_THROTTLE | WaitType::DBSEEDING_FLOWCONTROL | WaitType::DBSEEDING_OPERATION | WaitType::DROP_DATABASE_TIMER_TASK | WaitType::FABRIC_HADR_TRANSPORT_CONNECTION | WaitType::FABRIC_REPLICA_CONTROLLER_LIST | WaitType::FABRIC_REPLICA_CONTROLLER_STATE_AND_CONFIG | WaitType::FABRIC_REPLICA_PUBLISHER_EVENT_PUBLISH | WaitType::FABRIC_REPLICA_PUBLISHER_SUBSCRIBER_LIST | WaitType::FABRIC_WAIT_FOR_BUILD_REPLICA_EVENT_PROCESSING | WaitType::FEATURE_SWITCHES_UPDATE | WaitType::FFT_NSO_FCB_STATE | WaitType::HADR_DBSEEDING | WaitType::HADR_DBSEEDING_LIST | WaitType::HADR_FABRIC_CALLBACK | WaitType::HTTP_STORAGE_CONNECTION | WaitType::PHYSICAL_SEEDING_DMV | WaitType::PREEMPTIVE_XE_CX_FILE_OPEN | WaitType::PREEMPTIVE_XE_CX_HTTP_CALL | WaitType::PWAIT_HADR_JOIN | WaitType::PWAIT_LOG_CONSOLIDATION_IO | WaitType::PWAIT_LOG_CONSOLIDATION_POLL | WaitType::PWAIT_XTP_FSSTORAGE_MAINTENANCE | WaitType::PWAIT_XTP_HOST_STORAGE_WAIT | WaitType::QDS_ASYNC_CHECK_CONSISTENCY_TASK | WaitType::QDS_ASYNC_PERSIST_TASK | WaitType::QDS_ASYNC_PERSIST_TASK_START | WaitType::QDS_BCKG_TASK | WaitType::QDS_CLEANUP_STALE_QUERIES_TASK_MAIN_LOOP_SLEEP | WaitType::QDS_CTXS | WaitType::QDS_DB_DISK | WaitType::QDS_DYN_VECTOR | WaitType::QDS_LOADDB | WaitType::QDS_PERSIST_TASK_MAIN_LOOP_SLEEP | WaitType::QDS_SHUTDOWN_QUEUE | WaitType::QDS_STMT | WaitType::QDS_STMT_DISK | WaitType::QDS_TASK_SHUTDOWN | WaitType::QDS_TASK_START | WaitType::QE_WARN_LIST_SYNC | WaitType::RTDATA_LIST | WaitType::SLO_UPDATE | WaitType::SNI_CONN_DUP | WaitType::VDI_CLIENT_COMPLETECOMMAND | WaitType::VDI_CLIENT_GETCOMMAND | WaitType::VDI_CLIENT_OPERATION | WaitType::VDI_CLIENT_OTHER | WaitType::WAIT_SCRIPTDEPLOYMENT_REQUEST | WaitType::WAIT_SCRIPTDEPLOYMENT_WORKER | WaitType::WAIT_XTP_ASYNC_TX_COMPLETION | WaitType::WAIT_XTP_CKPT_AGENT_WAKEUP | WaitType::WAIT_XTP_OFFLINE_CKPT_BEFORE_REDO | WaitType::WINFAB_API_CALL | WaitType::WINFAB_REPLICA_BUILD_OPERATION | WaitType::XE_CX_FILE_READ | WaitType::XTP_HOST_DB_COLLECTION | WaitType::XTP_HOST_LOG_ACTIVITY => "Internal use only.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::AUDIT_GROUPCACHE_LOCK => "Occurs when there's a wait on a lock that controls access to a special cache. The cache contains information about which audits are being used to audit each audit action group.",
            WaitType::AUDIT_LOGINCACHE_LOCK => "Occurs when there's a wait on a lock that controls access to a special cache. The cache contains information about which audits are being used to audit login audit action groups.",
            WaitType::AUDIT_ON_DEMAND_TARGET_LOCK => "Occurs when there's a wait on a lock that is used to ensure single initialization of audit related Extended Event targets.",
            WaitType::AUDIT_XE_SESSION_MGR => "Occurs when there's a wait on a lock that is used to synchronize the starting and stopping of audit related Extended Events sessions.",
            WaitType::BACKUP => "Occurs when a task is blocked as part of backup processing.",
            WaitType::BACKUPBUFFER | WaitType::BACKUPIO => "Occurs when a backup task is waiting for data, or is waiting for a buffer in which to store data. This type isn't typical, except when a task is waiting for a tape mount.",
            WaitType::BACKUPTHREAD => "Occurs when a task is waiting for a backup task to finish. Wait times might be long, from several minutes to several hours. If the task that is being waited on is in an I/O process, this type doesn't indicate a problem.",
            WaitType::BACKUP_OPERATOR => "Occurs when a task is waiting for a tape mount. To view the tape status, query sys.dm_io_backup_tapes. If a mount operation isn't pending, this wait type might indicate a hardware problem with the tape drive.",
            WaitType::BAD_PAGE_PROCESS => "Occurs when the background suspect page logger is trying to avoid running more than every five seconds. Excessive suspect pages cause the logger to run frequently.",
            WaitType::BMPALLOCATION => "Occurs with parallel batch-mode plans when synchronizing the allocation of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::BMPBUILD => "Occurs with parallel batch-mode plans when synchronizing the building of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::BMPREPARTITION => "Occurs with parallel batch-mode plans when synchronizing the repartitioning of a large bitmap filter. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::BMPREPLICATION => "Occurs with parallel batch-mode plans when synchronizing the replication of a large bitmap filter across worker threads. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::BPSORT => "Occurs with parallel batch-mode plans when synchronizing the sorting of a dataset across multiple threads. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2016 (13.x) and later versions.",
            WaitType::BROKER_CONNECTION_RECEIVE_TASK => "Occurs when waiting for access to receive a message on a connection endpoint. Receive access to the endpoint is serialized.",
            WaitType::BROKER_ENDPOINT_STATE_MUTEX => "Occurs when there's contention to access the state of a Service Broker connection endpoint. Access to the state for changes is serialized.",
            WaitType::BROKER_EVENTHANDLER => "Occurs when a task is waiting in the primary event handler of the Service Broker. This should occur very briefly.",
            WaitType::BROKER_INIT => "Occurs when initializing Service Broker in each active database. This should occur infrequently.",
            WaitType::BROKER_MASTERSTART => "Occurs when a task is waiting for the primary event handler of the Service Broker to start. This should occur very briefly.",
            WaitType::BROKER_RECEIVE_WAITFOR => "Occurs when the RECEIVE WAITFOR is waiting. This might mean that either no messages are ready to be received in the queue or a lock contention is preventing it from receiving messages from the queue.",
            WaitType::BROKER_REGISTERALLENDPOINTS => "Occurs during the initialization of a Service Broker connection endpoint. This should occur very briefly.",
            WaitType::BROKER_SERVICE => "Occurs when the Service Broker destination list that is associated with a target service is updated or reprioritized.",
            WaitType::BROKER_SHUTDOWN => "Occurs when there's a planned shutdown of Service Broker. This should occur very briefly, if at all.",
            WaitType::BROKER_TASK_STOP => "Occurs when the Service Broker queue task handler tries to shut down the task. The state check is serialized and must be in a running state beforehand.",
            WaitType::BROKER_TO_FLUSH => "Occurs when the Service Broker lazy flusher flushes the in-memory transmission objects to a work table.",
            WaitType::BROKER_TRANSMITTER => "Occurs when the Service Broker transmitter is waiting for work. Service Broker has a component known as the Transmitter, which schedules messages from multiple dialogs to be sent across the wire over one or more connection endpoints. The transmitter has two dedicated threads for this purpose. This wait type is charged when these transmitter threads are waiting for dialog messages to be sent using the transport connections. High values of waiting_tasks_count for this wait type point to intermittent work for these transmitter threads and aren't indications of any performance problem. If service broker isn't used at all, waiting_tasks_count should be 2 (for the two transmitter threads), and wait_time_ms should be twice the duration since instance startup. See Service broker wait stats.",
            WaitType::BUFFERPOOL_SCAN => "Might occur when the buffer pool scan runs in parallel and the main task waits for the scan to complete. For more information, see Operations that trigger a buffer pool scan may run slowly on large-memory computers.\n\nApplies to: SQL Server 2022 (16.x) and later versions.",
            WaitType::BUILTIN_HASHKEY_MUTEX => "Might occur after startup of instance, while internal data structures are initializing. Doesn't recur once data structures have initialized.",
            WaitType::CHECKPOINT_QUEUE => "Occurs while the checkpoint task is waiting for the next checkpoint request.",
            WaitType::CHKPT => "Occurs at server startup to tell the checkpoint thread that it can start.",
            WaitType::CLEAR_DB => "Occurs during operations that change the state of a database, such as opening or closing a database.",
            WaitType::CLRHOST_STATE_ACCESS => "Occurs where there's a wait to acquire exclusive access to the CLR-hosting data structures. This wait type occurs while setting up or tearing down the CLR runtime.",
            WaitType::CLR_AUTO_EVENT => "Occurs when a task is currently performing common language runtime (CLR) execution and is waiting for a particular autoevent to be initiated. Long waits are typical, and don't indicate a problem.",
            WaitType::CLR_CRST => "Occurs when a task is currently performing CLR execution, and is waiting to enter a critical section of the task that is currently being used by another task.",
            WaitType::CLR_JOIN => "Occurs when a task is currently performing CLR execution, and is waiting for another task to end. This wait state occurs when there's a join between tasks.",
            WaitType::CLR_MANUAL_EVENT => "Occurs when a task is currently performing CLR execution, and is waiting for a specific manual event to be initiated.",
            WaitType::CLR_MEMORY_SPY => "Occurs during a wait on lock acquisition for a data structure that is used to record all virtual memory allocations that come from CLR. The data structure is locked to maintain its integrity if there's parallel access.",
            WaitType::CLR_MONITOR => "Occurs when a task is currently performing CLR execution, and is waiting to obtain a lock on the monitor.",
            WaitType::CLR_RWLOCK_READER => "Occurs when a task is currently performing CLR execution, and is waiting for a reader lock.",
            WaitType::CLR_RWLOCK_WRITER => "Occurs when a task is currently performing CLR execution, and is waiting for a writer lock.",
            WaitType::CLR_SEMAPHORE => "Occurs when a task is currently performing CLR execution, and is waiting for a semaphore.",
            WaitType::CLR_TASK_START => "Occurs while waiting for a CLR task to complete startup.",
            WaitType::CMEMTHREAD => "Occurs when a task is waiting on a thread-safe memory object. The wait time might increase when there's contention caused by multiple tasks trying to allocate memory from the same memory object.",
            WaitType::COMMIT_TABLE | WaitType::DUMP_LOG_COORDINATOR_QUEUE | WaitType::NODE_CACHE_MUTEX | WaitType::PERFORMANCE_COUNTERS_RWLOCK | WaitType::PREEMPTIVE_COM_COGETCLASSOBJECT | WaitType::PREEMPTIVE_COM_CREATEACCESSOR | WaitType::PREEMPTIVE_COM_DELETEROWS | WaitType::PREEMPTIVE_COM_GETCOMMANDTEXT | WaitType::PREEMPTIVE_COM_GETDATA | WaitType::PREEMPTIVE_COM_GETNEXTROWS | WaitType::PREEMPTIVE_COM_GETRESULT | WaitType::PREEMPTIVE_COM_GETROWSBYBOOKMARK | WaitType::PREEMPTIVE_COM_LBFLUSH | WaitType::PREEMPTIVE_COM_LBLOCKREGION | WaitType::PREEMPTIVE_COM_LBREADAT | WaitType::PREEMPTIVE_COM_LBSETSIZE | WaitType::PREEMPTIVE_COM_LBSTAT | WaitType::PREEMPTIVE_COM_LBUNLOCKREGION | WaitType::PREEMPTIVE_COM_LBWRITEAT | WaitType::PREEMPTIVE_COM_QUERYINTERFACE | WaitType::PREEMPTIVE_COM_RELEASE | WaitType::PREEMPTIVE_COM_RELEASEACCESSOR | WaitType::PREEMPTIVE_COM_RELEASEROWS | WaitType::PREEMPTIVE_COM_RELEASESESSION | WaitType::PREEMPTIVE_COM_RESTARTPOSITION | WaitType::PREEMPTIVE_COM_SEQSTRMREAD | WaitType::PREEMPTIVE_COM_SEQSTRMREADANDWRITE | WaitType::PREEMPTIVE_COM_SETDATAFAILURE | WaitType::PREEMPTIVE_COM_SETPARAMETERINFO | WaitType::PREEMPTIVE_COM_SETPARAMETERPROPERTIES | WaitType::PREEMPTIVE_COM_STRMLOCKREGION | WaitType::PREEMPTIVE_COM_STRMSEEKANDREAD | WaitType::PREEMPTIVE_COM_STRMSEEKANDWRITE | WaitType::PREEMPTIVE_COM_STRMSETSIZE | WaitType::PREEMPTIVE_COM_STRMSTAT | WaitType::PREEMPTIVE_COM_STRMUNLOCKREGION | WaitType::PREEMPTIVE_CONSOLEWRITE | WaitType::PREEMPTIVE_CREATEPARAM | WaitType::PREEMPTIVE_DEBUG | WaitType::PREEMPTIVE_DFSADDLINK | WaitType::PREEMPTIVE_DFSLINKEXISTCHECK | WaitType::PREEMPTIVE_DFSLINKHEALTHCHECK | WaitType::PREEMPTIVE_DFSREMOVELINK | WaitType::PREEMPTIVE_DFSREMOVEROOT | WaitType::PREEMPTIVE_DFSROOTFOLDERCHECK | WaitType::PREEMPTIVE_DFSROOTINIT | WaitType::PREEMPTIVE_DFSROOTSHARECHECK | WaitType::PREEMPTIVE_DTC_ABORT | WaitType::PREEMPTIVE_DTC_ABORTREQUESTDONE | WaitType::PREEMPTIVE_DTC_BEGINTRANSACTION | WaitType::PREEMPTIVE_DTC_COMMITREQUESTDONE | WaitType::PREEMPTIVE_DTC_ENLIST | WaitType::PREEMPTIVE_DTC_PREPAREREQUESTDONE | WaitType::PREEMPTIVE_FILESIZEGET | WaitType::PREEMPTIVE_FSAOLEDB_ABORTTRANSACTION | WaitType::PREEMPTIVE_FSAOLEDB_COMMITTRANSACTION | WaitType::PREEMPTIVE_FSAOLEDB_STARTTRANSACTION | WaitType::PREEMPTIVE_FSRECOVER_UNCONDITIONALUNDO | WaitType::PREEMPTIVE_GETRMINFO | WaitType::PREEMPTIVE_LOCKMONITOR | WaitType::PREEMPTIVE_MSS_RELEASE | WaitType::PREEMPTIVE_ODBCOPS | WaitType::PREEMPTIVE_OLEDBOPS | WaitType::PREEMPTIVE_OLEDB_ABORTORCOMMITTRAN | WaitType::PREEMPTIVE_OLEDB_ABORTTRAN | WaitType::PREEMPTIVE_OLEDB_GETDATASOURCE | WaitType::PREEMPTIVE_OLEDB_GETLITERALINFO | WaitType::PREEMPTIVE_OLEDB_GETPROPERTIES | WaitType::PREEMPTIVE_OLEDB_GETPROPERTYINFO | WaitType::PREEMPTIVE_OLEDB_GETSCHEMALOCK | WaitType::PREEMPTIVE_OLEDB_JOINTRANSACTION | WaitType::PREEMPTIVE_OLEDB_RELEASE | WaitType::PREEMPTIVE_OLEDB_SETPROPERTIES | WaitType::PREEMPTIVE_OLE_UNINIT | WaitType::PREEMPTIVE_OS_ACCEPTSECURITYCONTEXT | WaitType::PREEMPTIVE_OS_ACQUIRECREDENTIALSHANDLE | WaitType::PREEMPTIVE_OS_AUTHENTICATIONOPS | WaitType::PREEMPTIVE_OS_AUTHORIZATIONOPS | WaitType::PREEMPTIVE_OS_AUTHZGETINFORMATIONFROMCONTEXT | WaitType::PREEMPTIVE_OS_AUTHZINITIALIZECONTEXTFROMSID | WaitType::PREEMPTIVE_OS_AUTHZINITIALIZERESOURCEMANAGER | WaitType::PREEMPTIVE_OS_BACKUPREAD | WaitType::PREEMPTIVE_OS_CLOSEHANDLE | WaitType::PREEMPTIVE_OS_CLUSTEROPS | WaitType::PREEMPTIVE_OS_COMOPS | WaitType::PREEMPTIVE_OS_COMPLETEAUTHTOKEN | WaitType::PREEMPTIVE_OS_COPYFILE | WaitType::PREEMPTIVE_OS_CREATEDIRECTORY | WaitType::PREEMPTIVE_OS_CREATEFILE | WaitType::PREEMPTIVE_OS_CRYPTACQUIRECONTEXT | WaitType::PREEMPTIVE_OS_CRYPTIMPORTKEY | WaitType::PREEMPTIVE_OS_CRYPTOPS | WaitType::PREEMPTIVE_OS_DECRYPTMESSAGE | WaitType::PREEMPTIVE_OS_DELETEFILE | WaitType::PREEMPTIVE_OS_DELETESECURITYCONTEXT | WaitType::PREEMPTIVE_OS_DEVICEIOCONTROL | WaitType::PREEMPTIVE_OS_DEVICEOPS | WaitType::PREEMPTIVE_OS_DIRSVC_NETWORKOPS | WaitType::PREEMPTIVE_OS_DISCONNECTNAMEDPIPE | WaitType::PREEMPTIVE_OS_DOMAINSERVICESOPS | WaitType::PREEMPTIVE_OS_DSGETDCNAME | WaitType::PREEMPTIVE_OS_DTCOPS | WaitType::PREEMPTIVE_OS_ENCRYPTMESSAGE | WaitType::PREEMPTIVE_OS_FILEOPS | WaitType::PREEMPTIVE_OS_FINDFILE | WaitType::PREEMPTIVE_OS_FLUSHFILEBUFFERS | WaitType::PREEMPTIVE_OS_FORMATMESSAGE | WaitType::PREEMPTIVE_OS_FREECREDENTIALSHANDLE | WaitType::PREEMPTIVE_OS_FREELIBRARY | WaitType::PREEMPTIVE_OS_GENERICOPS | WaitType::PREEMPTIVE_OS_GETADDRINFO | WaitType::PREEMPTIVE_OS_GETCOMPRESSEDFILESIZE | WaitType::PREEMPTIVE_OS_GETDISKFREESPACE | WaitType::PREEMPTIVE_OS_GETFILEATTRIBUTES | WaitType::PREEMPTIVE_OS_GETFILESIZE | WaitType::PREEMPTIVE_OS_GETLONGPATHNAME | WaitType::PREEMPTIVE_OS_GETPROCADDRESS | WaitType::PREEMPTIVE_OS_GETVOLUMENAMEFORVOLUMEMOUNTPOINT | WaitType::PREEMPTIVE_OS_GETVOLUMEPATHNAME | WaitType::PREEMPTIVE_OS_INITIALIZESECURITYCONTEXT | WaitType::PREEMPTIVE_OS_LIBRARYOPS | WaitType::PREEMPTIVE_OS_LOADLIBRARY | WaitType::PREEMPTIVE_OS_LOGONUSER | WaitType::PREEMPTIVE_OS_LOOKUPACCOUNTSID | WaitType::PREEMPTIVE_OS_MESSAGEQUEUEOPS | WaitType::PREEMPTIVE_OS_MOVEFILE | WaitType::PREEMPTIVE_OS_NETGROUPGETUSERS | WaitType::PREEMPTIVE_OS_NETLOCALGROUPGETMEMBERS | WaitType::PREEMPTIVE_OS_NETUSERGETGROUPS | WaitType::PREEMPTIVE_OS_NETUSERGETLOCALGROUPS | WaitType::PREEMPTIVE_OS_NETUSERMODALSGET | WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICY | WaitType::PREEMPTIVE_OS_NETVALIDATEPASSWORDPOLICYFREE | WaitType::PREEMPTIVE_OS_OPENDIRECTORY | WaitType::PREEMPTIVE_OS_PIPEOPS | WaitType::PREEMPTIVE_OS_PROCESSOPS | WaitType::PREEMPTIVE_OS_QUERYREGISTRY | WaitType::PREEMPTIVE_OS_QUERYSECURITYCONTEXTTOKEN | WaitType::PREEMPTIVE_OS_REMOVEDIRECTORY | WaitType::PREEMPTIVE_OS_REPORTEVENT | WaitType::PREEMPTIVE_OS_REVERTTOSELF | WaitType::PREEMPTIVE_OS_RSFXDEVICEOPS | WaitType::PREEMPTIVE_OS_SECURITYOPS | WaitType::PREEMPTIVE_OS_SERVICEOPS | WaitType::PREEMPTIVE_OS_SETENDOFFILE | WaitType::PREEMPTIVE_OS_SETFILEPOINTER | WaitType::PREEMPTIVE_OS_SETFILEVALIDDATA | WaitType::PREEMPTIVE_OS_SETNAMEDSECURITYINFO | WaitType::PREEMPTIVE_OS_SQLCLROPS | WaitType::PREEMPTIVE_OS_VERIFYSIGNATURE | WaitType::PREEMPTIVE_OS_VSSOPS | WaitType::PREEMPTIVE_OS_WAITFORSINGLEOBJECT | WaitType::PREEMPTIVE_OS_WINSOCKOPS | WaitType::PREEMPTIVE_OS_WRITEFILE | WaitType::PREEMPTIVE_OS_WRITEFILEGATHER | WaitType::PREEMPTIVE_OS_WSASETLASTERROR | WaitType::PREEMPTIVE_REENLIST | WaitType::PREEMPTIVE_RESIZELOG | WaitType::PREEMPTIVE_ROLLFORWARDREDO | WaitType::PREEMPTIVE_ROLLFORWARDUNDO | WaitType::PREEMPTIVE_SB_STOPENDPOINT | WaitType::PREEMPTIVE_SERVER_STARTUP | WaitType::PREEMPTIVE_SETRMINFO | WaitType::PREEMPTIVE_SHAREDMEM_GETDATA | WaitType::PREEMPTIVE_SNIOPEN | WaitType::PREEMPTIVE_SOSHOST | WaitType::PREEMPTIVE_STARTRM | WaitType::PREEMPTIVE_STREAMFCB_CHECKPOINT | WaitType::PREEMPTIVE_STREAMFCB_RECOVER | WaitType::PREEMPTIVE_TRANSIMPORT | WaitType::PREEMPTIVE_UNMARSHALPROPAGATIONTOKEN | WaitType::PREEMPTIVE_VSS_CREATESNAPSHOT | WaitType::PREEMPTIVE_VSS_CREATEVOLUMESNAPSHOT | WaitType::PREEMPTIVE_XE_CALLBACKEXECUTE | WaitType::PREEMPTIVE_XE_DISPATCHER | WaitType::PREEMPTIVE_XE_ENGINEINIT | WaitType::PREEMPTIVE_XE_GETTARGETSTATE | WaitType::PREEMPTIVE_XE_SESSIONCOMMIT | WaitType::PREEMPTIVE_XE_TARGETFINALIZE | WaitType::PREEMPTIVE_XE_TARGETINIT | WaitType::PREEMPTIVE_XE_TIMERRUN | WaitType::REPL_HISTORYCACHE_ACCESS | WaitType::REPL_TRANHASHTABLE_ACCESS | WaitType::RG_RECONFIG | WaitType::TRACE_EVTNOTIF | WaitType::XE_SERVICES_EVENTMANUAL | WaitType::XE_SERVICES_MUTEX | WaitType::XE_SERVICES_RWLOCK | WaitType::XE_SESSION_CREATE_SYNC | WaitType::XE_SESSION_FLUSH | WaitType::XE_SESSION_SYNC | WaitType::XE_STM_CREATE | WaitType::XE_TIMER_EVENT | WaitType::XE_TIMER_MUTEX | WaitType::XE_TIMER_TASK_DONE => "Internal use only.",
            WaitType::CXCONSUMER => "Occurs with parallel query plans when a consumer thread (parent) waits for a producer thread to send rows. CXCONSUMER waits are caused by an Exchange Iterator that runs out of rows from its producer thread. This is a normal part of parallel query execution.\n\nApplies to: SQL Server (Starting with SQL Server 2016 (13.x) Service Pack 2, SQL Server 2017 (14.x) CU 3), Azure SQL Database, Azure SQL Managed Instance",
            WaitType::CXPACKET => "Occurs with parallel query plans when waiting to synchronize the Query Processor Exchange Iterator, and when producing and consuming rows. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nNote: Starting with SQL Server 2016 (13.x) Service Pack 2 and SQL Server 2017 (14.x) CU 3, CXPACKET only refers to waiting to synchronize the Exchange Iterator and producing rows. Threads consuming rows are tracked separately in the CXCONSUMER wait type. If the consumer threads are too slow, the Exchange Iterator buffer might become full and cause CXPACKET waits.\n\nNote: In SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance, CXPACKET only refers to waiting on threads producing rows. Exchange Iterator synchronization is tracked separately in the CXSYNC_PORT and CXSYNC_CONSUMER wait types. Threads consuming rows are tracked separately in the CXCONSUMER wait type.",
            WaitType::CXROWSET_SYNC => "Occurs during a parallel range scan.",
            WaitType::CXSYNC_CONSUMER => "Occurs with parallel query plans when waiting to reach an Exchange Iterator synchronization point among all consumer threads.\n\nApplies to: SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance",
            WaitType::CXSYNC_PORT => "Occurs with parallel query plans when waiting to open, close, and synchronize Exchange Iterator ports between producer and consumer threads. For example, if a query plan has a long sort operation, CXSYNC_PORT waits might be higher because the sort must complete before the Exchange Iterator port can be synchronized.\n\nApplies to: SQL Server 2022 (16.x) and later versions, Azure SQL Database, and Azure SQL Managed Instance",
            WaitType::DAC_INIT => "Occurs while the dedicated administrator connection is initializing.",
            WaitType::DBMIRRORING_CMD => "Occurs when a task is waiting for log records to be flushed to disk. This wait state is expected to be held for long periods of time.",
            WaitType::DBMIRROR_EVENTS_QUEUE => "Occurs when database mirroring waits for events to process.",
            WaitType::DBMIRROR_SEND => "Occurs when a task is waiting for a communications backlog at the network layer to clear to be able to send messages. Indicates that the communications layer is starting to become overloaded and affect the database mirroring data throughput.",
            WaitType::DBMIRROR_WORKER_QUEUE => "Indicates that the database mirroring worker task is waiting for more work.",
            WaitType::DEADLOCK_ENUM_MUTEX => "Occurs when the deadlock monitor and sys.dm_os_waiting_tasks try to make sure that SQL Server isn't running multiple deadlock searches at the same time.",
            WaitType::DEADLOCK_TASK_SEARCH => "Large waiting time on this resource indicates that the server is executing queries on top of sys.dm_os_waiting_tasks, and these queries are blocking deadlock monitor from running deadlock search. This wait type is used by deadlock monitor only. Queries on top of sys.dm_os_waiting_tasks use DEADLOCK_ENUM_MUTEX.",
            WaitType::DEBUG => "Occurs during Transact-SQL and CLR debugging for internal synchronization.",
            WaitType::DISABLE_VERSIONING => "Occurs when SQL Server polls the version transaction manager to see whether the timestamp of the earliest active transaction is later than the timestamp of when the state started changing. If this is this case, all the snapshot transactions that were started before the ALTER DATABASE statement was run have finished. This wait state is used when SQL Server disables versioning by using the ALTER DATABASE statement.",
            WaitType::DISKIO_SUSPEND => "Occurs when a task is waiting to access a file when an external backup is active. This is reported for each waiting user process. A count larger than five per user process might indicate that the external backup is taking too much time to finish.",
            WaitType::DISPATCHER_QUEUE_SEMAPHORE => "Occurs when a thread from the dispatcher pool is waiting for more work to process. The wait time for this wait type is expected to increase when the dispatcher is idle.",
            WaitType::DLL_LOADING_MUTEX => "Occurs once while waiting for the XML parser DLL to load.",
            WaitType::DROPTEMP => "Occurs between attempts to drop a temporary object if the previous attempt failed. The wait duration grows exponentially with each failed drop attempt.",
            WaitType::DTC => "Occurs when a task is waiting on an event that is used to manage state transition. This state controls when the recovery of Microsoft Distributed Transaction Coordinator (MS DTC) transactions occurs after SQL Server receives notification that the MS DTC service has become unavailable.",
            WaitType::DTC_ABORT_REQUEST => "Occurs in an MSDTC worker session when the session is waiting to take ownership of an MSDTC transaction. After MS DTC owns the transaction, the session can roll back the transaction. Generally, the session waits for another session that is using the transaction.",
            WaitType::DTC_RESOLVE => "Occurs when a recovery task is waiting for the master database in a cross-database transaction so that the task can query the outcome of the transaction.",
            WaitType::DTC_STATE => "Occurs when a task is waiting on an event that protects changes to the internal MS DTC global state object. This state should be held for very short periods of time.",
            WaitType::DTC_TMDOWN_REQUEST => "Occurs in an MSDTC worker session when SQL Server receives notification that the MS DTC service isn't available. First, the worker waits for the MS DTC recovery process to start. Then, the worker waits to obtain the outcome of the distributed transaction that the worker is working on. This might continue until the connection with the MS DTC service has been reestablished.",
            WaitType::DTC_WAITFOR_OUTCOME => "Occurs when recovery tasks wait for MS DTC to become active to enable the resolution of prepared transactions.",
            WaitType::DUMP_LOG_COORDINATOR => "Occurs when a main task is waiting for a subtask to generate data. Ordinarily, this state doesn't occur. A long wait indicates an unexpected blockage. The subtask should be investigated.",
            WaitType::EE_PMOLOCK => "Occurs during synchronization of certain types of memory allocations during statement execution.",
            WaitType::EE_SPECPROC_MAP_INIT => "Occurs during synchronization of internal procedure hash table creation. This wait can only occur during the initial accessing of the hash table after the SQL Server instance starts.",
            WaitType::ENABLE_VERSIONING => "Occurs when SQL Server waits for all update transactions in this database to finish before declaring the database ready to transition to snapshot isolation allowed state. This state is used when SQL Server enables snapshot isolation by using the ALTER DATABASE statement.",
            WaitType::ERROR_REPORTING_MANAGER => "Occurs during synchronization of multiple concurrent error log initializations.",
            WaitType::EXCHANGE => "Occurs during synchronization in the query processor exchange iterator during parallel queries.",
            WaitType::EXECSYNC => "Occurs during parallel queries while synchronizing in query processor in areas not related to the exchange iterator. Examples of such areas are bitmaps, large binary objects (LOBs), and the spool iterator. LOBs might frequently use this wait state.",
            WaitType::EXECUTION_PIPE_EVENT_INTERNAL => "Occurs during synchronization between producer and consumer parts of batch execution that are submitted through the connection context.",
            WaitType::EXTERNAL_SCRIPT_NETWORK_IO | WaitType::FOREIGN_REDO => "Internal use only.\n\nApplies to: SQL Server 2017 (14.x) through current.",
            WaitType::FCB_REPLICA_READ => "Occurs when the reads of a snapshot (or a temporary snapshot created by DBCC) sparse file are synchronized.",
            WaitType::FCB_REPLICA_WRITE => "Occurs when the pushing or pulling of a page to a snapshot (or a temporary snapshot created by DBCC) sparse file is synchronized.",
            WaitType::FSAGENT => "Occurs when a FILESTREAM file I/O operation is waiting for a FILESTREAM agent resource that is being used by another file I/O operation.",
            WaitType::FSA_FORCE_OWN_XACT => "Occurs when a FILESTREAM file I/O operation needs to bind to the associated transaction, but the transaction is currently owned by another session.",
            WaitType::FSTR_CONFIG_MUTEX => "Occurs when there's a wait for another FILESTREAM feature reconfiguration to be completed.",
            WaitType::FSTR_CONFIG_RWLOCK => "Occurs when there's a wait to serialize access to the FILESTREAM configuration parameters.",
            WaitType::FS_FC_RWLOCK => "Occurs when there's a wait by the FILESTREAM garbage collector to do either of the following tasks:\n\n- Disable garbage collection (used by backup and restore).\n- Execute one cycle of the FILESTREAM garbage collector.",
            WaitType::FS_GARBAGE_COLLECTOR_SHUTDOWN => "Occurs when the FILESTREAM garbage collector is waiting for cleanup tasks to be completed.",
            WaitType::FS_HEADER_RWLOCK => "Occurs when there's a wait to acquire access to the FILESTREAM header of a FILESTREAM data container to either read or update contents in the FILESTREAM header file (Filestream.hdr).",
            WaitType::FS_LOGTRUNC_RWLOCK => "Occurs when there's a wait to acquire access to FILESTREAM log truncation to do either of the following tasks:\n\n- Temporarily disable FILESTREAM log (FSLOG) truncation (used by backup and restore).\n- Execute one cycle of FSLOG truncation.",
            WaitType::FT_COMPROWSET_RWLOCK => "Full-text is waiting on fragment metadata operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_IFTSHC_MUTEX => "Full-text is waiting on an FDHost control operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_IFTSISM_MUTEX => "Full-text is waiting on communication operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_IFTS_ASYNC_WRITE_PIPE | WaitType::FT_IFTS_BLOB_HASH | WaitType::FT_IFTS_CATEALOG_SOURCE | WaitType::FT_IFTS_CHUNK_BUFFER_CLIENT_MANAGER | WaitType::FT_IFTS_CHUNK_BUFFER_PROTO_WORD_LIST | WaitType::FT_IFTS_COMP_DESC_MANAGER | WaitType::FT_IFTS_CONSUMER_PLUGIN | WaitType::FT_IFTS_CRAWL_BATCH_LIST | WaitType::FT_IFTS_CRAWL_CHILDREN | WaitType::FT_IFTS_DOCID_INTERFACE_LIST | WaitType::FT_IFTS_DOCID_LIST | WaitType::FT_IFTS_FP_INFO_LIST | WaitType::FT_IFTS_HOST_CONTROLLER | WaitType::FT_IFTS_MASTER_MERGE_TASK_LIST | WaitType::FT_IFTS_MEMREGPOOL | WaitType::FT_IFTS_MERGE_FRAGMENT_SYNC | WaitType::FT_IFTS_NOISE_WORDS_COLLECTION_CACHE | WaitType::FT_IFTS_NOISE_WORDS_RESOURCE | WaitType::FT_IFTS_OCCURRENCE_BUFFER_POOL | WaitType::FT_IFTS_PIPELINE | WaitType::FT_IFTS_PIPELINE_LIST | WaitType::FT_IFTS_PIPELINE_MANAGER | WaitType::FT_IFTS_PROJECT_FD_INFO_MAP | WaitType::FT_IFTS_SCHEDULER | WaitType::FT_IFTS_SHARED_MEMORY | WaitType::FT_IFTS_SHUTDOWN_PIPE | WaitType::FT_IFTS_SRCH_FD_MANAGER | WaitType::FT_IFTS_SRCH_FD_SERVICE | WaitType::FT_IFTS_STOPLIST_CACHE_MANAGER | WaitType::FT_IFTS_THESAURUS | WaitType::FT_IFTS_VERSION_MANAGER | WaitType::FT_IFTS_WORK_QUEUE => "Internal use only.\n\nApplies to: SQL Server 2022 (16.x) CU 1 and later versions.",
            WaitType::FT_IFTS_RWLOCK => "Full-text is waiting on internal synchronization. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_IFTS_SCHEDULER_IDLE_WAIT => "Full-text scheduler sleep wait type. The scheduler is idle.",
            WaitType::FT_MASTER_MERGE => "Full-text is waiting on master merge operation. Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_METADATA_MUTEX => "Documented for informational purposes only. Not supported. Future compatibility isn't guaranteed.",
            WaitType::FT_RESTART_CRAWL => "Occurs when a full-text crawl needs to restart from a last known good point to recover from a transient failure. The wait lets the worker tasks currently working on that population to complete or exit the current step.",
            WaitType::FULLTEXT_GATHERER => "Occurs during synchronization of full-text operations.",
            WaitType::GHOSTCLEANUP_UPDATE_STATS | WaitType::GLOBAL_QUERY_CANCEL | WaitType::GLOBAL_QUERY_CONSUMER | WaitType::GLOBAL_QUERY_PRODUCER | WaitType::HADR_THROTTLE_LOG_RATE_LOG_SIZE | WaitType::HADR_THROTTLE_LOG_RATE_SEEDING | WaitType::HADR_THROTTLE_LOG_RATE_SEND_RECV_QUEUE_SIZE | WaitType::MEMORY_GRANT_UPDATE | WaitType::MIGRATIONBUFFER | WaitType::PWAIT_PREEMPTIVE_APP_USAGE_TIMER | WaitType::PWAIT_SBS_FILE_OPERATION | WaitType::QDS_HOST_INIT | WaitType::RBIO_WAIT_VLF | WaitType::REMOTE_BLOCK_IO | WaitType::SBS_DISPATCH | WaitType::SBS_RECEIVE_TRANSPORT | WaitType::SBS_TRANSPORT | WaitType::SECURITY_CNG_PROVIDER_MUTEX | WaitType::SNI_WRITE_ASYNC | WaitType::TEMPORAL_BACKGROUND_PROCEED_CLEANUP | WaitType::WAIT_XLOGREAD_SIGNAL | WaitType::WAIT_XTP_COMPILE_WAIT | WaitType::WAIT_XTP_SERIAL_RECOVERY | WaitType::XIO_EDS_MGR_RWLOCK | WaitType::XIO_EDS_RWLOCK | WaitType::XIO_IOSTATS_BLOBLIST_RWLOCK | WaitType::XIO_IOSTATS_FCBLIST_RWLOCK => "Internal use only.\n\nApplies to: SQL Server 2017 (14.x) and later versions.",
            WaitType::HADR_AG_MUTEX => "Occurs when an availability group DDL statement or Windows Server Failover Clustering command is waiting for exclusive read/write access to the configuration of an availability group.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_ARCONTROLLER_NOTIFICATIONS_SUBSCRIBER_LIST => "The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the list of event subscribers. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_AR_CRITICAL_SECTION_ENTRY => "Occurs when an availability group DDL statement or Windows Server Failover Clustering command is waiting for exclusive read/write access to the runtime state of the local replica of the associated availability group.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_AR_MANAGER_MUTEX => "Occurs when an availability replica shutdown is waiting for startup to complete or an availability replica startup is waiting for shutdown to complete. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_BACKUP_BULK_LOCK => "The availability group primary database received a backup request from a secondary database and is waiting for the background thread to finish processing the request on acquiring or releasing the BulkOp lock.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_BACKUP_QUEUE => "The backup background thread of the availability group primary database is waiting for a new work request from the secondary database. (Typically, this occurs when the primary database is holding the BulkOp log and is waiting for the secondary database to indicate that the primary database can release the lock).\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_CLUSAPI_CALL => "A SQL Server thread is waiting to switch from non-preemptive mode (scheduled by SQL Server) to preemptive mode (scheduled by the operating system) in order to invoke Windows Server Failover Clustering APIs.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_COMPRESSED_CACHE_SYNC => "Waiting for access to the cache of compressed log blocks that is used to avoid redundant compression of the log blocks sent to multiple secondary databases.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DATABASE_FLOW_CONTROL => "Waiting for messages to be sent to the partner when the maximum number of queued messages has been reached. Indicates that the log scans are running faster than the network sends. This is an issue only if network sends are slower than expected.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DATABASE_VERSIONING_STATE => "Occurs on the versioning state change of an availability group secondary database. This wait is for internal data structures and usually is very short with no direct effect on data access.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DATABASE_WAIT_FOR_RESTART => "Waiting for the database to restart under availability group control. Under normal conditions, this isn't a customer issue because waits are expected here.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DATABASE_WAIT_FOR_TRANSITION_TO_VERSIONING => "A query on objects in a readable secondary database of an availability group is blocked on row versioning while waiting for commit or rollback of all transactions that were in-flight when the secondary replica was enabled for read workloads. This wait type guarantees that row versions are available before execution of a query under snapshot isolation.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DBR_SUBSCRIBER => "The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the runtime state of an event subscriber that corresponds to an availability database. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DBR_SUBSCRIBER_FILTER_LIST => "The publisher for an availability replica event (such as a state change or configuration change) is waiting for exclusive read/write access to the list of event subscribers that correspond to availability databases. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DBSTATECHANGE_SYNC => "Concurrency control wait for updating the internal state of the database replica.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DB_COMMAND | WaitType::HADR_DB_OP_COMPLETION_SYNC => "Waiting for responses to conversational messages (which require an explicit response from the other side, using the availability group conversational message infrastructure). Many different message types use this wait type.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_DB_OP_START_SYNC => "An availability group DDL statement or a Windows Server Failover Clustering command is waiting for serialized access to an availability database and its runtime state.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_BLOCK_FLUSH => "The FILESTREAM Always On transport manager is waiting until processing of a log block is finished.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_FILE_CLOSE => "The FILESTREAM Always On transport manager is waiting until the next FILESTREAM file gets processed and its handle gets closed.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_FILE_REQUEST => "An Always On secondary replica is waiting for the primary replica to send all requested FILESTREAM files during UNDO.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_IOMGR => "The FILESTREAM Always On transport manager is waiting for R/W lock that protects the FILESTREAM Always On I/O manager during startup or shutdown.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_IOMGR_IOCOMPLETION => "The FILESTREAM Always On I/O manager is waiting for I/O completion.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_FILESTREAM_MANAGER => "The FILESTREAM Always On transport manager is waiting for the R/W lock that protects the FILESTREAM Always On transport manager during startup or shutdown.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_GROUP_COMMIT => "Transaction commit processing is waiting to allow a group commit so that multiple commit log records can be put into a single log block. This wait is an expected condition that optimizes the log I/O, capture, and send operations.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_LOGCAPTURE_SYNC => "Concurrency control around the log capture or apply object when creating or destroying scans. This is an expected wait when partners change state or connection status.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_LOGCAPTURE_WAIT => "Waiting for log records to become available. Can occur either when waiting for new log records to be generated by connections or for I/O completion when reading log not in the cache. This is an expected wait if the log scan is caught up to the end of log or is reading from disk.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_LOGPROGRESS_SYNC => "Concurrency control wait when updating the log progress status of database replicas.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_NOTIFICATION_DEQUEUE => "A background task that processes Windows Server Failover Clustering notifications is waiting for the next notification. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_NOTIFICATION_WORKER_EXCLUSIVE_ACCESS => "The availability replica manager is waiting for serialized access to the runtime state of a background task that processes Windows Server Failover Clustering notifications. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_NOTIFICATION_WORKER_STARTUP_SYNC => "A background task is waiting for the completion of the startup of a background task that processes Windows Server Failover Clustering notifications. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_NOTIFICATION_WORKER_TERMINATION_SYNC => "A background task is waiting for the termination of a background task that processes Windows Server Failover Clustering notifications. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_PARTNER_SYNC => "Concurrency control wait on the partner list.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_READ_ALL_NETWORKS => "Waiting to get read or write access to the list of WSFC networks. Internal use only. Note: The engine keeps a list of WSFC networks that is used in DMVs (such as sys.dm_hadr_cluster_networks) or to validate Always On Transact-SQL statements that reference WSFC network information. This list is updated upon engine startup, WSFC related notifications, and internal Always On restart (for example, losing and regaining of WSFC quorum). Tasks are usually blocked when an update in that list is in progress.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_RECOVERY_WAIT_FOR_CONNECTION => "Waiting for the secondary database to connect to the primary database before running recovery. This is an expected wait, which can lengthen if the connection to the primary is slow to establish.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_RECOVERY_WAIT_FOR_UNDO => "Database recovery is waiting for the secondary database to finish the reverting and initializing phase to bring it back to the common log point with the primary database. This is an expected wait after failovers. Undo progress can be tracked through the Windows System Monitor (perfmon.exe) and DMVs.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_REPLICAINFO_SYNC => "Waiting for concurrency control to update the current replica state.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_SYNCHRONIZING_THROTTLE => "Waiting for transaction commit processing to allow a synchronizing secondary database to catch up to the primary end of the log, in order to transition to the synchronized state. This is an expected wait when a secondary database is catching up.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_SYNC_COMMIT => "Waiting for a transaction commit processing on the synchronized secondary databases to harden the log. This wait is also reflected by the Transaction Delay performance counter. This wait type is expected for synchronous-commit availability groups, and indicates the time to send, write, and acknowledge log commit to the secondary databases.\nFor detailed information and troubleshooting HADR_SYNC_COMMIT, refer to this blog post\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_TDS_LISTENER_SYNC => "Either the internal Always On system, or the WSFC cluster, requests that listeners are started or stopped. The processing of this request is always asynchronous, and there's a mechanism to remove redundant requests. There are also moments that this process is suspended because of configuration changes. All waits related with this listener synchronization mechanism use this wait type. Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_TDS_LISTENER_SYNC_PROCESSING => "Used at the end of an Always On Transact-SQL statement that requires starting and/or stopping an availability group listener. Since the start/stop operation is done asynchronously, the user thread blocks using this wait type until the situation of the listener is known.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_THROTTLE_LOG_RATE_MISMATCHED_SLO => "Occurs when a geo-replication secondary is configured with lower compute size (lower SLO) than the primary. A primary database is throttled due to delayed log consumption by the secondary. This is caused by the secondary database having insufficient compute capacity to keep up with the primary database's rate of change.\n\nApplies to: Azure SQL Database",
            WaitType::HADR_TIMER_TASK => "Waiting to get the lock on the timer task object and is also used for the actual waits between times that work is being performed. For example, for a task that runs every 10 seconds, after one execution, availability groups waits about 10 seconds to reschedule the task, and the wait is included here.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_TRANSPORT_DBRLIST => "Waiting for access to the transport layer's database replica list. Used for the spinlock that grants access to it.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_TRANSPORT_FLOW_CONTROL => "Waiting when the number of outstanding unacknowledged Always On messages is over the out flow control threshold. This is on an availability replica-to-replica basis (not on a database-to-database basis).\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_TRANSPORT_SESSION => "Availability groups are waiting while changing or accessing the underlying transport state.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_WORK_POOL => "Concurrency control wait on the availability group background work task object.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_WORK_QUEUE => "Availability group background worker thread waiting for new work to be assigned. This is an expected wait when there are ready workers waiting for new work, which is the normal state.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HADR_XRF_STACK_ACCESS => "Accessing (look up, add, and delete) the extended recovery fork stack for an availability database.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HTBUILD => "Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2012 (11.x) and later versions, but not Azure SQL Database, Azure SQL Managed Instance with the always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTBUILD_AGG => "Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTBUILD_JOIN => "Occurs with parallel batch-mode plans when synchronizing the building of the hash table on the input side of a hash join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTDELETE => "Occurs with parallel batch-mode plans when synchronizing at the end of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions, but not Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTDELETE_AGG => "Occurs with parallel batch-mode plans when synchronizing at the end of a hash aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTDELETE_JOIN => "Occurs with parallel batch-mode plans when synchronizing at the end of a hash join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: Azure SQL Database, Azure SQL Managed Instance with the SQL Server 2025 or Always-up-to-date update policy, and Azure Synapse Analytics.",
            WaitType::HTMEMO => "Occurs with parallel batch-mode plans when synchronizing before scanning hash table to output matches / non-matches in hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::HTREINIT => "Occurs with parallel batch-mode plans when synchronizing before resetting a hash join/aggregation for the next partial join. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::HTREPARTITION => "Occurs with parallel batch-mode plans when synchronizing the repartitioning of the hash table on the input side of a hash join/aggregation. If waiting is excessive and can't be reduced by tuning the query (such as adding indexes), consider adjusting the cost threshold for parallelism, or lowering the degree of parallelism.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::HTTP_ENUMERATION => "Occurs at startup to enumerate the HTTP endpoints to start HTTP.",
            WaitType::HTTP_START => "Occurs when a connection is waiting for HTTP to complete initialization.",
            WaitType::IMPPROV_IOWAIT => "Occurs when SQL Server waits for a bulkload I/O to finish.",
            WaitType::IO_AUDIT_MUTEX => "Occurs during synchronization of trace event buffers.",
            WaitType::IO_COMPLETION => "Occurs while waiting for I/O operations to complete. This wait type generally represents non-data page I/Os. Data page I/O completion waits appear as PAGEIOLATCH_* waits.",
            WaitType::IO_RETRY => "Occurs when an I/O operation such as a read or a write to disk fails because of insufficient resources, and is then retried.",
            WaitType::KSOURCE_WAKEUP => "Used by the service control task while waiting for requests from the Service Control Manager. Long waits are expected and don't indicate a problem.",
            WaitType::LATCH_DT => "Occurs when waiting for a DT (destroy) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.",
            WaitType::LATCH_EX => "Occurs when waiting for an EX (exclusive) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.",
            WaitType::LATCH_KP => "Occurs when waiting for a KP (keep) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.",
            WaitType::LATCH_SH => "Occurs when waiting for an SH (share) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.",
            WaitType::LATCH_UP => "Occurs when waiting for an UP (update) latch. This doesn't include buffer latches or transaction mark latches. A listing of LATCH_* waits is available in sys.dm_os_latch_stats. sys.dm_os_latch_stats groups LATCH_NL, LATCH_SH, LATCH_UP, LATCH_EX, and LATCH_DT waits together.",
            WaitType::LAZYWRITER_SLEEP => "Occurs when lazy writer tasks are suspended. This is a measure of the time spent by background tasks that are waiting. Don't consider this state when you're looking for user stalls.",
            WaitType::LCK_M_BU => "Occurs when a task is waiting to acquire a Bulk Update (BU) lock. For more information, see Bulk Update Locks.",
            WaitType::LCK_M_BU_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Bulk Update (BU) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Bulk Update Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_BU_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Bulk Update (BU) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Bulk Update Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IS => "Occurs when a task is waiting to acquire an Intent Shared (IS) lock. For more information, see Intent Locks.",
            WaitType::LCK_M_IS_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Intent Shared (IS) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IS_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Intent Shared (IS) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IU => "Occurs when a task is waiting to acquire an Intent Update (IU) lock. For more information, see Intent Locks.",
            WaitType::LCK_M_IU_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Intent Update (IU) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IU_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Intent Update (IU) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IX => "Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock. For more information, see Intent Locks.",
            WaitType::LCK_M_IX_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_IX_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Intent Exclusive (IX) lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_NL => "Occurs when a task is waiting to acquire a NULL lock on the current key value, and an Insert Range lock between the current and previous key. A NULL lock on the key is an instant release lock.",
            WaitType::LCK_M_RIn_NL_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a NULL lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. A NULL lock on the key is an instant release lock. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_NL_LOW_PRIORITY => "Occurs when a task is waiting to acquire a NULL lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. A NULL lock on the key is an instant release lock. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_S => "Occurs when a task is waiting to acquire a shared lock on the current key value, and an Insert Range lock between the current and previous key.",
            WaitType::LCK_M_RIn_S_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a shared lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_S_LOW_PRIORITY => "Occurs when a task is waiting to acquire a shared lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_U => "Task is waiting to acquire an Update lock on the current key value, and an Insert Range lock between the current and previous key.",
            WaitType::LCK_M_RIn_U_ABORT_BLOCKERS => "Task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_U_LOW_PRIORITY => "Task is waiting to acquire an Update lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_X => "Occurs when a task is waiting to acquire an Exclusive lock on the current key value, and an Insert Range lock between the current and previous key.",
            WaitType::LCK_M_RIn_X_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers on the current key value, and an Insert Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RIn_X_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Exclusive lock with Low Priority on the current key value, and an Insert Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RS_S => "Occurs when a task is waiting to acquire a Shared lock on the current key value, and a Shared Range lock between the current and previous key.",
            WaitType::LCK_M_RS_S_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Shared lock with Abort Blockers on the current key value, and a Shared Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RS_S_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Shared lock with Low Priority on the current key value, and a Shared Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RS_U => "Occurs when a task is waiting to acquire an Update lock on the current key value, and an Update Range lock between the current and previous key.",
            WaitType::LCK_M_RS_U_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Update Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RS_U_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Update lock with Low Priority on the current key value, and an Update Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_S => "Occurs when a task is waiting to acquire a Shared lock on the current key value, and an Exclusive Range lock between the current and previous key.",
            WaitType::LCK_M_RX_S_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Shared lock with Abort Blockers on the current key value, and an Exclusive Range with Abort Blockers lock between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_S_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Shared lock with Low Priority on the current key value, and an Exclusive Range with Low Priority lock between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_U => "Occurs when a task is waiting to acquire an Update lock on the current key value, and an Exclusive range lock between the current and previous key.",
            WaitType::LCK_M_RX_U_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Update lock with Abort Blockers on the current key value, and an Exclusive range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_U_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Update lock with Low Priority on the current key value, and an Exclusive range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_X => "Occurs when a task is waiting to acquire an Exclusive lock on the current key value, and an Exclusive Range lock between the current and previous key.",
            WaitType::LCK_M_RX_X_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers on the current key value, and an Exclusive Range lock with Abort Blockers between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_RX_X_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Exclusive lock with Low Priority on the current key value, and an Exclusive Range lock with Low Priority between the current and previous key. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.)\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_S => "Occurs when a task is waiting to acquire a Shared lock. For more information, see Shared Locks.",
            WaitType::LCK_M_SCH_M => "Occurs when a task is waiting to acquire a Schema Modify lock. For more information, see Schema Locks.",
            WaitType::LCK_M_SCH_M_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Schema Modify lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SCH_M_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Schema Modify lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SCH_S => "Occurs when a task is waiting to acquire a Schema Share lock. For more information, see Schema Locks.",
            WaitType::LCK_M_SCH_S_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Schema Share lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SCH_S_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Schema Share lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Schema Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SIU => "Occurs when a task is waiting to acquire a Shared With Intent Update lock. For more information, see Intent Locks.",
            WaitType::LCK_M_SIU_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Shared With Intent Update lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SIU_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Shared With Intent Update lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SIX => "Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock. For more information, see Intent Locks.",
            WaitType::LCK_M_SIX_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_SIX_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Shared With Intent Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_S_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire a Shared lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Shared Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_S_LOW_PRIORITY => "Occurs when a task is waiting to acquire a Shared lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Shared Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_S_XACT => "Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction) wait_resource type, where the read or modify intent can't be inferred.",
            WaitType::LCK_M_S_XACT_MODIFY => "Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction) wait_resource type, with an intent to modify.",
            WaitType::LCK_M_S_XACT_READ => "Occurs when optimized locking is enabled and a task is waiting for a shared lock on an XACT (transaction)wait_resource type, with an intent to read.",
            WaitType::LCK_M_U => "Occurs when a task is waiting to acquire an Update lock. For more information, see Update Locks.",
            WaitType::LCK_M_UIX => "Occurs when a task is waiting to acquire an Update With Intent Exclusive lock. For more information, see Intent Locks.",
            WaitType::LCK_M_UIX_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Update With Intent Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_UIX_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Update With Intent Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Intent Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_U_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Update lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Update Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_U_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Update lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Update Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_X => "Occurs when a task is waiting to acquire an Exclusive lock. For more information, see Exclusive Locks.",
            WaitType::LCK_M_X_ABORT_BLOCKERS => "Occurs when a task is waiting to acquire an Exclusive lock with Abort Blockers. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Exclusive Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LCK_M_X_LOW_PRIORITY => "Occurs when a task is waiting to acquire an Exclusive lock with Low Priority. (Related to the low priority wait option of ALTER TABLE and ALTER INDEX.) For more information, see Exclusive Locks.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::LOGBUFFER => "Occurs when a task is waiting for space in the log buffer to store a log record. Consistently high values might indicate that the log devices can't keep up with the amount of log being generated by the server.",
            WaitType::LOGMGR => "Occurs when a task is waiting for any outstanding log I/Os to finish before shutting down the log while closing the database.",
            WaitType::LOGMGR_QUEUE => "Occurs while the log writer task waits for work requests.",
            WaitType::LOGMGR_RESERVE_APPEND => "Occurs when a task is waiting to see whether log truncation frees up log space to enable the task to write a new log record. Consider increasing the size of the log files for the affected database to reduce this wait.",
            WaitType::LOWFAIL_MEMMGR_QUEUE => "Occurs while waiting for memory to be available for use.",
            WaitType::MEMORY_ALLOCATION_EXT => "Occurs while allocating memory from either the internal SQL Server memory pool or the operation system.\n\nApplies to: SQL Server 2016 (13.x) and later versions.",
            WaitType::METADATA_LAZYCACHE_RWLOCK | WaitType::SQLTRACE_LOCK => "Internal use only.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::MSQL_DQ => "Occurs when a task is waiting for a distributed query operation to finish. This is used to detect potential Multiple Active Result Set (MARS) application deadlocks. The wait ends when the distributed query call finishes.",
            WaitType::MSQL_XACT_MGR_MUTEX => "Occurs when a task is waiting to obtain ownership of the session transaction manager to perform a session level transaction operation.",
            WaitType::MSQL_XACT_MUTEX => "Occurs during synchronization of transaction usage. A request must acquire the mutex before it can use the transaction.",
            WaitType::MSQL_XP => "Occurs when a task is waiting for an extended stored procedure to end. SQL Server uses this wait state to detect potential MARS application deadlocks. The wait stops when the extended stored procedure call ends.",
            WaitType::MSSEARCH => "Occurs during Full-Text Search calls. This wait ends when the full-text operation completes. It doesn't indicate contention, but rather the duration of full-text operations.",
            WaitType::NET_WAITFOR_PACKET => "Occurs when a connection is waiting for a network packet during a network read.",
            WaitType::OLEDB => "Occurs when SQL Server calls the SNAC OLE DB Provider (SQLNCLI) or the Microsoft OLE DB Driver for SQL Server (MSOLEDBSQL). This wait type isn't used for synchronization. Instead, it indicates the duration of calls to the OLE DB provider.",
            WaitType::ONDEMAND_TASK_QUEUE => "Occurs while a background task waits for high priority system task requests. Long wait times indicate that there have been no high priority requests to process, and shouldn't cause concern.",
            WaitType::PAGEIOLATCH_DT => "Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Destroy mode. Long waits might indicate problems with the disk subsystem.",
            WaitType::PAGEIOLATCH_EX => "Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Exclusive mode - a mode used when the buffer is being written to disk. Long waits might indicate problems with the disk subsystem.\n\nFor more information, see Slow I/O - SQL Server and disk I/O performance.",
            WaitType::PAGEIOLATCH_KP => "Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Keep mode. Long waits might indicate problems with the disk subsystem.",
            WaitType::PAGEIOLATCH_SH => "Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Shared mode - a mode used when the buffer is being read from disk. Long waits might indicate problems with the disk subsystem.\n\nFor more information, see Slow I/O - SQL Server and disk I/O performance.",
            WaitType::PAGEIOLATCH_UP => "Occurs when a task is waiting on a latch for a buffer that is in an I/O request. The latch request is in Update mode. Long waits might indicate problems with the disk subsystem.\n\nFor more information, see Slow I/O - SQL Server and disk I/O performance.",
            WaitType::PAGELATCH_DT => "Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Destroy mode. Destroy mode must be acquired before deleting contents of a page. For more information, see Latch Modes.",
            WaitType::PAGELATCH_EX => "Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Exclusive mode - it blocks other threads from writing to or reading from the page (buffer).\n\nA common scenario that leads to this latch is the \"last-page insert\" buffer latch contention. To understand and resolve this, use Resolve last-page insert PAGELATCH_EX contention and Diagnose and resolve last-page-insert latch contention on SQL Server. Another scenario is Latch contention on small tables with a non-clustered index and random inserts (queue table).",
            WaitType::PAGELATCH_KP => "Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Keep mode, which prevents the page from being destroyed by another thread. For more information, see Latch Modes.",
            WaitType::PAGELATCH_SH => "Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Shared mode, which allows multiple threads to read, but not modify, a buffer (page). For more information, see Latch Modes.",
            WaitType::PAGELATCH_UP => "Occurs when a task is waiting on a latch for a buffer that isn't in an I/O request. The latch request is in Update mode. Commonly this wait type might be observed when a system page (buffer) like PFS, GAM, SGAM is latched. For more information, see Latch Modes.\n\nFor troubleshooting a common scenario with this latch, refer to Reduce Allocation Contention in SQL Server tempdb database.",
            WaitType::PARALLEL_BACKUP_QUEUE => "Occurs when serializing output produced by RESTORE HEADERONLY, RESTORE FILELISTONLY, or RESTORE LABELONLY.",
            WaitType::PREEMPTIVE_AUDIT_ACCESS_EVENTLOG => "Occurs when the SQL Server Operating System (SQLOS) scheduler switches to preemptive mode to write an audit event to the Windows event log.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::PREEMPTIVE_AUDIT_ACCESS_SECLOG => "Occurs when the SQLOS scheduler switches to preemptive mode to write an audit event to the Windows Security log.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::PREEMPTIVE_CLOSEBACKUPMEDIA => "Occurs when the SQLOS scheduler switches to preemptive mode to close backup media.",
            WaitType::PREEMPTIVE_CLOSEBACKUPTAPE => "Occurs when the SQLOS scheduler switches to preemptive mode to close a tape backup device.",
            WaitType::PREEMPTIVE_CLOSEBACKUPVDIDEVICE => "Occurs when the SQLOS scheduler switches to preemptive mode to close a virtual backup device.",
            WaitType::PREEMPTIVE_CLUSAPI_CLUSTERRESOURCECONTROL => "Occurs when the SQLOS scheduler switches to preemptive mode to perform Windows Server failover cluster operations.",
            WaitType::PREEMPTIVE_COM_COCREATEINSTANCE => "Occurs when the SQLOS scheduler switches to preemptive mode to create a COM object.",
            WaitType::PREEMPTIVE_HADR_LEASE_MECHANISM => "Availability group lease manager scheduling for Microsoft Support diagnostics.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PREEMPTIVE_OS_SQMLAUNCH => "Internal use only.\n\nApplies to: SQL Server 2008 R2 (10.50.x) through SQL Server 2016 (13.x).",
            WaitType::PRINT_ROLLBACK_PROGRESS => "Used to wait while user processes are ended in a database that has been transitioned by using the ALTER DATABASE termination clause. For more information, see ALTER DATABASE (Transact-SQL).",
            WaitType::PVS_CLEANUP_LOCK => "Occurs when the persistent version store (PVS) cleanup process is waiting for a lock required to start the cleanup. Might occur when an active transaction is preventing PVS cleanup initiated internally or using the sys.sp_persistent_version_cleanup system stored procedure. For more information, see Monitor and troubleshoot accelerated database recovery.\n\nApplies to: SQL Server 2019 (15.x) and later versions.",
            WaitType::PWAIT_HADR_CHANGE_NOTIFIER_TERMINATION_SYNC => "Occurs when a background task is waiting for the termination of the background task that receives (via polling) Windows Server Failover Clustering notifications.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_HADR_CLUSTER_INTEGRATION => "An append, replace, and/or remove operation is waiting to grab a write lock on an Always On internal list (such as a list of networks, network addresses, or availability group listeners). Internal use only.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_HADR_OFFLINE_COMPLETED => "A drop availability group operation is waiting for the target availability group to go offline before destroying Windows Server Failover Clustering objects.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_HADR_ONLINE_COMPLETED => "A create or failover availability group operation is waiting for the target availability group to come online.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_HADR_POST_ONLINE_COMPLETED => "A drop availability group operation is waiting for the termination of any background task that was scheduled as part of a previous command. For example, there might be a background task that is transitioning availability databases to the primary role. The DROP AVAILABILITY GROUP DDL must wait for this background task to terminate in order to avoid race conditions.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_HADR_WORKITEM_COMPLETED => "Internal wait by a thread waiting for an async work task to complete. This is an expected wait and is for CSS use.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_MD_LOGIN_STATS => "Occurs during internal synchronization in metadata on login stats.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_MD_RELATION_CACHE => "Occurs during internal synchronization in metadata on table or index.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_MD_SERVER_CACHE => "Occurs during internal synchronization in metadata on linked servers.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::PWAIT_MD_UPGRADE_CONFIG => "Occurs during internal synchronization in upgrading server wide configurations.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::QPJOB_KILL => "Indicates that an asynchronous automatic statistics update was canceled by a call to KILL as the update was starting to run. The terminating thread is suspended, waiting for it to start listening for KILL commands. A good value is less than one second.",
            WaitType::QPJOB_WAITFOR_ABORT => "Indicates that an asynchronous automatic statistics update was canceled by a call to KILL when it was running. The update has now completed but is suspended until the terminating thread message coordination is complete. This is an ordinary but rare state, and should be very short. A good value is less than one second.",
            WaitType::QRY_MEM_GRANT_INFO_MUTEX => "Occurs when Query Execution memory management tries to control access to static grant information list. This state lists information about the current granted and waiting memory requests. This state is a simple access control state. There should never be a long wait on this state. If this mutex isn't released, all new memory-using queries stop responding.",
            WaitType::QUERY_ERRHDL_SERVICE_DONE | WaitType::QUERY_WAIT_ERRHDL_SERVICE | WaitType::XE_PACKAGE_LOCK_BACKOFF => "Identified for informational purposes only. Not supported.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::QUERY_EXECUTION_INDEX_SORT_EVENT_OPEN => "Occurs in certain cases when offline create index build is run in parallel, and the different worker threads that are sorting synchronize access to the sort files.",
            WaitType::QUERY_NOTIFICATION_MGR_MUTEX => "Occurs during synchronization of the garbage collection queue in the Query Notification Manager.",
            WaitType::QUERY_NOTIFICATION_SUBSCRIPTION_MUTEX => "Occurs during state synchronization for transactions in Query Notifications.",
            WaitType::QUERY_NOTIFICATION_TABLE_MGR_MUTEX => "Occurs during internal synchronization within the Query Notification Manager.",
            WaitType::QUERY_OPTIMIZER_PRINT_MUTEX => "Occurs during synchronization of query optimizer diagnostic output production. This wait type only occurs if diagnostic settings have been enabled under direction of Microsoft Product Support.",
            WaitType::RBIO_RG_DESTAGE => "Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the long term log storage.\n\nApplies to: Azure SQL Database Hyperscale.",
            WaitType::RBIO_RG_LOCALDESTAGE => "Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the log service.\n\nApplies to: Azure SQL Database Hyperscale.",
            WaitType::RBIO_RG_REPLICA => "Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption by the readable secondary replica nodes.\n\nApplies to: Azure SQL Database Hyperscale.",
            WaitType::RBIO_RG_STORAGE => "Occurs when a Hyperscale database compute node is being throttled due to delayed log consumption at the page servers.\n\nApplies to: Azure SQL Database Hyperscale.",
            WaitType::RECOVER_CHANGEDB => "Occurs during synchronization of database status in warm standby database.",
            WaitType::REPLICA_WRITES => "Occurs while a task waits for completion of page writes to database snapshots or DBCC replicas.",
            WaitType::REPL_CACHE_ACCESS => "Occurs during synchronization on a replication article cache. During these waits, the replication log reader stalls, and data definition language (DDL) statements on a published table are blocked.",
            WaitType::REPL_SCHEMA_ACCESS => "Occurs during synchronization of replication schema version information. This state exists when DDL statements are executed on the replicated object, and when the log reader builds or consumes versioned schema based on DDL occurrence. Contention can be seen on this wait type if you have many published databases on a single publisher with transactional replication and the published databases are very active.",
            WaitType::REQUEST_DISPENSER_PAUSE => "Occurs when a task is waiting for all outstanding I/O to complete, so that I/O to a file can be frozen for snapshot backup.",
            WaitType::REQUEST_FOR_DEADLOCK_SEARCH => "Occurs while the deadlock monitor waits to start the next deadlock search. This wait is expected between deadlock detections, and lengthy total waiting time on this resource doesn't indicate a problem.",
            WaitType::RESMGR_THROTTLED => "Occurs when a new request comes in and is throttled based on the GROUP_MAX_REQUESTS setting.",
            WaitType::RESOURCE_QUEUE => "Occurs during synchronization of various internal resource queues.",
            WaitType::RESOURCE_SEMAPHORE => "Occurs when a query memory request during query execution can't be granted immediately due to other concurrent queries. High waits and wait times might indicate excessive number of concurrent queries, or excessive memory request amounts. Excessive waits of this type might raise SQL error 8645, \"A time out occurred while waiting for memory resources to execute the query. Rerun the query.\"\n\nFor detailed information and troubleshooting ideas on memory grant waits, see Troubleshoot slow performance or low memory issues caused by memory grants in SQL Server.",
            WaitType::RESOURCE_SEMAPHORE_MUTEX => "Occurs while a query waits for its request for a thread reservation to be fulfilled. It also occurs when synchronizing query compile and memory grant requests.",
            WaitType::RESOURCE_SEMAPHORE_QUERY_COMPILE => "Occurs when the number of concurrent query compilations reaches a throttling limit. High waits and wait times might indicate excessive compilations, recompiles, or uncacheable plans.",
            WaitType::RESOURCE_SEMAPHORE_SMALL_QUERY => "Occurs when memory request by a small query can't be granted immediately due to other concurrent queries. Wait time shouldn't exceed more than a few seconds, because the server transfers the request to the main query memory pool if it fails to grant the requested memory within a few seconds. High waits might indicate an excessive number of concurrent small queries while the main memory pool is blocked by waiting queries.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::SECURITY_MUTEX => "Occurs when there's a wait for mutexes that control access to the global list of Extensible Key Management (EKM) cryptographic providers and the session-scoped list of EKM sessions.",
            WaitType::SEC_DROP_TEMP_KEY => "Occurs after a failed attempt to drop a temporary security key before a retry attempt.",
            WaitType::SEQUENTIAL_GUID => "Occurs while a new sequential GUID is being obtained.",
            WaitType::SERVER_IDLE_CHECK => "Occurs during synchronization of SQL Server instance idle status when a resource monitor is attempting to declare a SQL Server instance as idle or trying to wake up.",
            WaitType::SHUTDOWN => "Occurs while a shutdown statement waits for active connections to exit.",
            WaitType::SLEEP_BPOOL_FLUSH => "Occurs when a checkpoint is throttling the issuance of new I/Os in order to avoid flooding the disk subsystem.",
            WaitType::SLEEP_DBSTARTUP => "Occurs during database startup while waiting for all databases to recover.",
            WaitType::SLEEP_DCOMSTARTUP => "Occurs once at most during SQL Server instance startup while waiting for DCOM initialization to complete.",
            WaitType::SLEEP_MSDBSTARTUP => "Occurs when SQL Trace waits for the msdb database to complete startup.",
            WaitType::SLEEP_SYSTEMTASK => "Occurs during the start of a background task while waiting for tempdb to complete startup.",
            WaitType::SLEEP_TASK => "Occurs when a task sleeps while waiting for a generic event to occur.",
            WaitType::SLEEP_TEMPDBSTARTUP => "Occurs while a task waits for tempdb to complete startup.",
            WaitType::SNI_CRITICAL_SECTION => "Occurs during internal synchronization within SQL Server networking components.",
            WaitType::SNI_HTTP_WAITFOR_0_DISCON => "Occurs during SQL Server shutdown, while waiting for outstanding HTTP connections to exit.",
            WaitType::SNI_LISTENER_ACCESS => "Occurs while waiting for non-uniform memory access (NUMA) nodes to update state change. Access to state change is serialized.",
            WaitType::SNI_TASK_COMPLETION => "Occurs when there's a wait for all tasks to finish during a NUMA node state change.",
            WaitType::SOAP_READ => "Occurs while waiting for an HTTP network read to complete.",
            WaitType::SOAP_WRITE => "Occurs while waiting for an HTTP network write to complete.",
            WaitType::SOSHOST_EVENT => "Occurs when a hosted component, such as CLR, waits on a SQL Server event synchronization object.",
            WaitType::SOSHOST_INTERNAL => "Occurs during synchronization of memory manager callbacks used by hosted components, such as CLR.",
            WaitType::SOSHOST_MUTEX => "Occurs when a hosted component, such as CLR, waits on a SQL Server mutex synchronization object.",
            WaitType::SOSHOST_RWLOCK => "Occurs when a hosted component, such as CLR, waits on a SQL Server reader-writer synchronization object.",
            WaitType::SOSHOST_SEMAPHORE => "Occurs when a hosted component, such as CLR, waits on a SQL Server semaphore synchronization object.",
            WaitType::SOSHOST_SLEEP => "Occurs when a hosted task sleeps while waiting for a generic event to occur. Hosted tasks are used by hosted components such as CLR.",
            WaitType::SOSHOST_TRACELOCK => "Occurs during synchronization of access to trace streams.",
            WaitType::SOSHOST_WAITFORDONE => "Occurs when a hosted component, such as CLR, waits for a task to complete.",
            WaitType::SOS_CALLBACK_REMOVAL => "Occurs while performing synchronization on a callback list in order to remove a callback. It isn't expected for this counter to change after server initialization is completed.",
            WaitType::SOS_DISPATCHER_MUTEX => "Occurs during internal synchronization of the dispatcher pool. This includes when the pool is being adjusted.",
            WaitType::SOS_LOCALALLOCATORLIST => "Occurs during internal synchronization in the SQL Server memory manager.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::SOS_MEMORY_USAGE_ADJUSTMENT => "Occurs when memory usage is being adjusted among pools.",
            WaitType::SOS_OBJECT_STORE_DESTROY_MUTEX => "Occurs during internal synchronization in memory pools when destroying objects from the pool.",
            WaitType::SOS_PHYS_PAGE_CACHE => "Accounts for the time a thread waits to acquire the mutex it must acquire before it allocates physical pages or before it returns those pages to the operating system. Waits on this type only appear if the instance of SQL Server uses AWE memory.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::SOS_PROCESS_AFFINITY_MUTEX => "Occurs during synchronizing of access to process affinity settings.",
            WaitType::SOS_RESERVEDMEMBLOCKLIST => "Occurs during internal synchronization in the SQL Server Memory Manager.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::SOS_SCHEDULER_YIELD => "Occurs when a task voluntarily yields the scheduler for other tasks to execute. During this wait, the task is waiting in a runnable queue for its quantum to be renewed, that is, waiting to be scheduled to run on the CPU again. Prolonged waits on this wait type most frequently indicate opportunities to optimize queries that perform index or table scans. Focus on plan regression, missing index, stats updates, and query rewrites. Optimizing runtimes reduces the need for tasks to be yielding multiple times. If query times for such CPU-consuming tasks are acceptable, then this wait type is expected and can be ignored.",
            WaitType::SOS_SMALL_PAGE_ALLOC => "Occurs during the allocation and freeing of memory that is managed by some memory objects.",
            WaitType::SOS_STACKSTORE_INIT_MUTEX => "Occurs during synchronization of internal store initialization.",
            WaitType::SOS_SYNC_TASK_ENQUEUE_EVENT => "Occurs when a task is started in a synchronous manner. Most tasks in SQL Server are started in an asynchronous manner, in which control returns to the starter immediately after the task request has been placed on the work queue.",
            WaitType::SOS_VIRTUALMEMORY_LOW => "Occurs when a memory allocation waits for a Resource Manager to free up virtual memory.",
            WaitType::SOS_WORK_DISPATCHER => "Internal use only.\n\nApplies to: SQL Server 2019 (15.x) and later versions.",
            WaitType::SPINLOCK_EXT => "Occurs when a thread is waiting to acquire a spinlock. Includes both the spinning and the sleeping time. High values might indicate spinlock contention.\n\nBecause of a possibility of a minor performance impact with high throughput and high concurrency workloads, the SPINLOCK_EXT waits are tracked only if trace flag 8134 is enabled.\n\nApplies to: SQL Server 2025 (17.x) and later versions.",
            WaitType::SQLCLR_APPDOMAIN => "Occurs while CLR waits for an application domain to complete startup.",
            WaitType::SQLCLR_ASSEMBLY => "Occurs while waiting for access to the loaded assembly list in the appdomain.",
            WaitType::SQLCLR_DEADLOCK_DETECTION => "Occurs while CLR waits for deadlock detection to complete.",
            WaitType::SQLCLR_QUANTUM_PUNISHMENT => "Occurs when a CLR task is throttled because it has exceeded its execution quantum. This throttling is done in order to reduce the effect of this resource-intensive task on other tasks.",
            WaitType::SQLSORT_NORMMUTEX | WaitType::SQLSORT_SORTMUTEX => "Occurs during internal synchronization, while initializing internal sorting structures.",
            WaitType::SQLTRACE_BUFFER_FLUSH => "Occurs when a task is waiting for a background task to flush trace buffers to disk every four seconds.\n\nApplies to: SQL Server 2008 R2 (10.50.x) only.",
            WaitType::SQLTRACE_FILE_BUFFER => "Occurs during synchronization on trace buffers during a file trace.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::SQLTRACE_SHUTDOWN => "Occurs while trace shutdown waits for outstanding trace events to complete.",
            WaitType::SQLTRACE_WAIT_ENTRIES => "Occurs while a SQL Trace event queue waits for packets to arrive on the queue.",
            WaitType::SRVPROC_SHUTDOWN => "Occurs while the shutdown process waits for internal resources to be released to shut down cleanly.",
            WaitType::TEMPOBJ => "Occurs when temporary object drops are synchronized. This wait is rare, and only occurs if a task has requested exclusive access for temp table drops.",
            WaitType::THREADPOOL => "Occurs when a task (query or login/logout) is waiting for a worker thread to execute it. This can indicate that the maximum worker thread setting is misconfigured, or, most commonly, that batch executions are taking unusually long, thus reducing the number of worker threads available to satisfy other batches. Examine the performance of batches (queries) and reduce query duration by either reducing bottlenecks (blocking, parallelism, I/O, latch waits), or providing proper indexing or query design.",
            WaitType::TIMEPRIV_TIMEPERIOD => "Occurs during internal synchronization of the Extended Events timer.",
            WaitType::TRACEWRITE => "Occurs when the SQL Trace rowset trace provider waits for either a free buffer or a buffer with events to process.",
            WaitType::TRANSACTION_MUTEX => "Occurs during synchronization of access to a transaction by multiple batches.",
            WaitType::TRAN_MARKLATCH_DT => "Occurs when waiting for a destroy mode latch on a transaction mark latch. Transaction mark latches are used for synchronization of commits with marked transactions.",
            WaitType::TRAN_MARKLATCH_EX => "Occurs when waiting for an exclusive mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.",
            WaitType::TRAN_MARKLATCH_KP => "Occurs when waiting for a keep mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.",
            WaitType::TRAN_MARKLATCH_SH => "Occurs when waiting for a shared mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.",
            WaitType::TRAN_MARKLATCH_UP => "Occurs when waiting for an update mode latch on a marked transaction. Transaction mark latches are used for synchronization of commits with marked transactions.",
            WaitType::UTIL_PAGE_ALLOC => "Occurs when transaction log scans wait for memory to be available during memory pressure.",
            WaitType::VIA_ACCEPT => "Occurs when a Virtual Interface Adapter (VIA) provider connection is completed during startup.",
            WaitType::VIEW_DEFINITION_MUTEX => "Occurs during synchronization on access to cached view definitions.",
            WaitType::WAITFOR => "Occurs as a result of a WAITFOR Transact-SQL statement. The duration of the wait is determined by the parameters to the statement. This is a user-initiated wait.",
            WaitType::WAITSTAT_MUTEX => "Occurs during synchronization of access to the collection of statistics used to populate sys.dm_os_wait_stats.",
            WaitType::WAIT_FOR_RESULTS => "Occurs when waiting for a query notification to be triggered.",
            WaitType::WAIT_ON_SYNC_STATISTICS_REFRESH => "Occurs when waiting for synchronous statistics update to complete before query compilation and execution can resume.\n\nApplies to: Starting with SQL Server 2019 (15.x)",
            WaitType::WAIT_XTP_CKPT_CLOSE => "Occurs when waiting for a checkpoint to complete.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_CKPT_ENABLED => "Occurs when checkpointing is disabled, and waiting for checkpointing to be enabled.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_CKPT_STATE_LOCK => "Occurs when synchronizing checking of checkpoint state.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_GUEST => "Occurs when the database memory allocator needs to stop receiving low-memory notifications.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::WAIT_XTP_HOST_WAIT => "Occurs when waits are triggered by the database engine and implemented by the host.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_OFFLINE_CKPT_LOG_IO => "Occurs when offline checkpoint is waiting for a log read IO to complete.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_OFFLINE_CKPT_NEW_LOG => "Occurs when offline checkpoint is waiting for new log records to scan.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_PROCEDURE_ENTRY => "Occurs when a drop procedure is waiting for all current executions of that procedure to complete.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_RECOVERY => "Occurs when database recovery is waiting for recovery of memory-optimized objects to finish.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WAIT_XTP_TASK_SHUTDOWN => "Occurs when waiting for an In-Memory OLTP thread to complete.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::WAIT_XTP_TRAN_DEPENDENCY => "Occurs when waiting for transaction dependencies.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::WORKTBL_DROP => "Occurs while pausing before retrying, after a failed worktable drop.",
            WaitType::WRITELOG => "Occurs while waiting for a log flush to complete. Common operations that cause log flushes are transaction commits and checkpoints. Common reasons for long waits on WRITELOG are: disk latency (where transaction log files reside), the inability for I/O to keep up with transactions, or, a large number of transaction log operations and flushes (commits, rollback)",
            WaitType::WRITE_COMPLETION => "Occurs when a write operation is in progress.",
            WaitType::XACTLOCKINFO => "Occurs during synchronization of access to the list of locks for a transaction. In addition to the transaction itself, the list of locks is accessed by operations such as deadlock detection and lock migration during page splits.",
            WaitType::XACTWORKSPACE_MUTEX => "Occurs during synchronization of defections from a transaction, as well as the number of database locks between enlist members of a transaction.",
            WaitType::XACT_OWN_TRANSACTION => "Occurs while waiting to acquire ownership of a transaction.",
            WaitType::XACT_RECLAIM_SESSION => "Occurs while waiting for the current owner of a session to release ownership of the session.",
            WaitType::XE_BUFFERMGR_ALLPROCESSED_EVENT => "Occurs when Extended Events session buffers are flushed to targets. This wait occurs on a background thread.",
            WaitType::XE_BUFFERMGR_FREEBUF_EVENT => "Occurs when either of the following conditions is true:\n\n- An Extended Events session is configured for no event loss, and all buffers in the session are currently full. This can indicate that the buffers for an Extended Events session are too small or should be partitioned.\n- Audits experience a delay. This can indicate a disk bottleneck on the drive where the audits are written.",
            WaitType::XE_DISPATCHER_CONFIG_SESSION_LIST => "Occurs when an Extended Events session that is using asynchronous targets is started or stopped. This wait indicates either of the following conditions:\n\n- An Extended Events session is registering with a background thread pool.\n- The background thread pool is calculating the required number of threads based on current load.",
            WaitType::XE_DISPATCHER_JOIN => "Occurs when a background thread that is used for Extended Events sessions is terminating.",
            WaitType::XE_DISPATCHER_WAIT => "Occurs when a background thread that is used for Extended Events sessions is waiting for event buffers to process.",
            WaitType::XTPPROC_CACHE_ACCESS => "Occurs when for accessing all natively compiled stored procedure cache objects.\n\nApplies to: SQL Server 2014 (12.x) and later versions.",
            WaitType::XTPPROC_PARTITIONED_STACK_CREATE => "Occurs when allocating per-NUMA node natively compiled stored procedure cache structures (must be done single threaded) for a given procedure.\n\nApplies to: SQL Server 2012 (11.x) and later versions.",
            WaitType::Unknown(_) => "Not present in the generated wait-type table.",
        }
    }
}
